use crate::cli::args::InitiativeCommands;
use crate::core::collections::initiatives::{
    self, find_initiative_across_stores, registered_stores, resolve_selected_store,
    CreateInitiativeInput, InitiativeState,
};
use serde::Serialize;

pub fn run(cmd: InitiativeCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        InitiativeCommands::Create {
            id,
            title,
            summary,
            store,
            store_path,
            json,
        } => run_create(
            id.as_deref(),
            title.as_deref(),
            summary.as_deref(),
            store.as_deref(),
            store_path.as_deref(),
            json,
        ),
        InitiativeCommands::Show {
            id,
            store,
            store_path,
            json,
        } => run_show(&id, store.as_deref(), store_path.as_deref(), json),
        InitiativeCommands::List {
            store,
            store_path,
            json,
        } => run_list(store.as_deref(), store_path.as_deref(), json),
    }
}

#[derive(Serialize)]
struct CreateOutput {
    store: StoreInfo,
    initiative: InitiativeState,
}

#[derive(Serialize)]
struct StoreInfo {
    id: String,
    root: String,
}

fn run_create(
    id: Option<&str>,
    title: Option<&str>,
    summary: Option<&str>,
    store: Option<&str>,
    store_path: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate required fields
    let id = id.ok_or_else(|| "Pass an initiative id.".to_string())?;
    let title =
        title.ok_or_else(|| "Initiative title is required. Pass --title <title>.".to_string())?;
    let summary = summary
        .ok_or_else(|| "Initiative summary is required. Pass --summary <summary>.".to_string())?;

    // Resolve the store
    let selected = resolve_selected_store(store, store_path, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Create the initiative
    let initiative = initiatives::create_initiative(
        &selected.root,
        CreateInitiativeInput {
            id: id.to_string(),
            title: title.to_string(),
            summary: summary.to_string(),
            ..Default::default()
        },
    )
    .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        let output = CreateOutput {
            store: StoreInfo {
                id: selected.id.clone(),
                root: selected.root.to_string_lossy().to_string(),
            },
            initiative,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        let initiative_dir = selected.root.join("initiatives").join(&initiative.id);
        println!("Initiative created: {}", initiative.id);
        println!("Store: {}", selected.id);
        println!("Location: {}", initiative_dir.display());
        println!();
        println!("Next: read {}/requirements.md", initiative_dir.display());
    }

    Ok(())
}

#[derive(Serialize)]
struct ShowOutput {
    store: StoreInfo,
    initiative: InitiativeState,
    root: String,
    #[serde(rename = "metadataPath")]
    metadata_path: String,
}

fn run_show(
    id: &str,
    store: Option<&str>,
    store_path: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let initiative = if store.is_some() || store_path.is_some() {
        // Specific store requested
        let selected = resolve_selected_store(store, store_path, None)
            .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

        let found = initiatives::read_initiative(&selected.root, id)
            .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?
            .ok_or_else(|| {
                format!(
                    "Initiative '{}' was not found in context store '{}'.",
                    id, selected.id
                )
            })?;

        (selected, found)
    } else {
        // Search across all stores
        let matches = find_initiative_across_stores(id, None)
            .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

        match matches.len() {
            0 => {
                return Err(format!(
                    "Initiative '{}' was not found in registered context stores.",
                    id
                )
                .into());
            }
            1 => matches.into_iter().next().unwrap(),
            _ => {
                return Err(format!(
                    "Initiative '{}' exists in multiple context stores. Use --store <store>.",
                    id
                )
                .into());
            }
        }
    };

    let (selected, found) = initiative;
    let initiative_dir = selected.root.join("initiatives").join(&found.id);
    let metadata_path = initiative_dir.join("initiative.yaml");

    if json {
        let output = ShowOutput {
            store: StoreInfo {
                id: selected.id,
                root: selected.root.to_string_lossy().to_string(),
            },
            initiative: found,
            root: initiative_dir.to_string_lossy().to_string(),
            metadata_path: metadata_path.to_string_lossy().to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Initiative: {}", found.id);
        println!("Store: {}", selected.id);
        println!("Location: {}", initiative_dir.display());
        println!("Metadata: {}", metadata_path.display());
    }

    Ok(())
}

#[derive(Serialize)]
struct ListStoreGroup {
    store: StoreInfo,
    initiatives: Vec<InitiativeState>,
}

#[derive(Serialize)]
struct ListOutput {
    stores: Vec<ListStoreGroup>,
}

fn run_list(
    store: Option<&str>,
    store_path: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let stores_to_query = if store.is_some() || store_path.is_some() {
        // Single store requested
        let selected = resolve_selected_store(store, store_path, None)
            .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;
        vec![selected]
    } else {
        // All registered stores
        registered_stores(None)
    };

    if stores_to_query.is_empty() {
        if json {
            let output = ListOutput { stores: vec![] };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No context stores registered.");
            println!();
            println!("Next:");
            println!("  openspec context-store setup team");
            println!("  openspec initiative create <id> --title <title> --summary <summary> --store team");
        }
        return Ok(());
    }

    let mut groups = Vec::new();
    let mut total_initiatives = 0;

    for store in stores_to_query {
        let initiatives = initiatives::list_initiatives(&store.root)
            .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;
        total_initiatives += initiatives.len();
        groups.push(ListStoreGroup {
            store: StoreInfo {
                id: store.id,
                root: store.root.to_string_lossy().to_string(),
            },
            initiatives,
        });
    }

    if json {
        let output = ListOutput { stores: groups };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if total_initiatives == 0 {
            println!("No initiatives found.");
            println!();
            println!("Next:");
            println!("  openspec initiative create <id> --title <title> --summary <summary> --store <store>");
            return Ok(());
        }

        for group in groups {
            println!("Store: {}", group.store.id);
            println!("Location: {}", group.store.root);
            for initiative in group.initiatives {
                println!(
                    "  {:<30} {:<12} {}",
                    initiative.id,
                    format!("{:?}", initiative.status).to_lowercase(),
                    initiative.title
                );
            }
            println!();
        }
    }

    Ok(())
}
