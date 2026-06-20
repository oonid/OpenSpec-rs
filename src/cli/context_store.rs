use crate::cli::args::ContextStoreCommands;
use crate::core::context_store::{
    self, DoctorResult, ListResult, MutationResult, CleanupResult,
};

pub fn run(cmd: ContextStoreCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ContextStoreCommands::Setup {
            id,
            path,
            init_git,
            allow_inside_git_repository,
            json,
        } => {
            run_setup(
                id.as_deref(),
                path.as_deref(),
                init_git,
                allow_inside_git_repository,
                json,
            )
        }
        ContextStoreCommands::Register { path, id, json } => {
            run_register(path.as_deref(), id.as_deref(), json)
        }
        ContextStoreCommands::Unregister { id, json } => {
            run_unregister(&id, json)
        }
        ContextStoreCommands::Remove { id, yes, json } => {
            run_remove(&id, yes, json)
        }
        ContextStoreCommands::List { json } => {
            run_list(json)
        }
        ContextStoreCommands::Doctor { id, json } => {
            run_doctor(id.as_deref(), json)
        }
    }
}

fn run_setup(
    id: Option<&str>,
    path: Option<&str>,
    init_git: bool,
    allow_inside_git_repository: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = context_store::setup_context_store(
        id,
        path,
        init_git,
        allow_inside_git_repository,
        None,
    )
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_mutation_human("Context store ready", &result);
    }

    Ok(())
}

fn run_register(
    path: Option<&str>,
    id: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = context_store::register_existing_context_store(path, id, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_mutation_human("Context store registered", &result);
    }

    Ok(())
}

fn run_unregister(id: &str, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = context_store::unregister_context_store(id, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_cleanup_human("Unregistered context store", &result);
    }

    Ok(())
}

fn run_remove(id: &str, yes: bool, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Deleting the local folder is destructive, so require explicit confirmation. main() prints
    // the returned error once, so return a single guidance message (no extra eprintln here).
    if !yes {
        return Err(format!(
            "refusing to delete context store '{id}' without confirmation. Re-run with --yes: openspec context-store remove {id} --yes"
        )
        .into());
    }

    let result = context_store::remove_context_store(id, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_cleanup_human("Removed context store", &result);
    }

    Ok(())
}

fn run_list(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = context_store::list_context_stores(None);

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_list_human(&result);
    }

    Ok(())
}

fn run_doctor(id: Option<&str>, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = context_store::doctor_context_stores(id, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_doctor_human(&result);
    }

    Ok(())
}

// Human-readable output formatting functions

fn print_mutation_human(title: &str, result: &MutationResult) {
    println!("{}: {}", title, result.store.id);
    println!("Location: {}", result.store.root);
    println!();
    println!("Next: ask your agent to create an initiative in {}.", result.store.id);
}

fn print_cleanup_human(title: &str, result: &CleanupResult) {
    println!("{}: {}", title, result.store.id);

    if let Some(deleted_path) = &result.deleted_path {
        println!("Deleted: {}", deleted_path);
    } else if let Some(left_on_disk) = &result.left_on_disk {
        println!("Files kept at: {}", left_on_disk);
    } else if !result.deleted {
        println!("Files were already missing: {}", result.store.root);
    }
}

fn print_list_human(result: &ListResult) {
    if result.stores.is_empty() {
        println!("No context stores registered.");
        println!();
        println!("Next:");
        println!("  openspec context-store setup team-context");
        println!("  openspec context-store register /path/to/context-store");
        return;
    }

    println!("OpenSpec context stores ({})", result.stores.len());
    println!();
    println!("{:<16}Location", "ID");
    for store in &result.stores {
        println!("{:<16}{}", store.id, store.root);
    }
}

fn print_doctor_human(result: &DoctorResult) {
    if result.stores.is_empty() {
        println!("No context stores registered.");
        return;
    }

    println!("Context store doctor");
    for store in &result.stores {
        println!();
        println!("{}", store.id);
        println!("  Location: {}", store.root);
        println!(
            "  Metadata: {}",
            if store.metadata_valid {
                "ok"
            } else if store.metadata_present {
                "invalid"
            } else {
                "missing"
            }
        );
        println!(
            "  Git: {}",
            if store.is_git_repository {
                "repository detected"
            } else {
                "not detected"
            }
        );
        println!("  Issues: none");
    }
}
