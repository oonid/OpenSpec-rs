use chrono::Utc;

use crate::core::config::OPENSPEC_DIR_NAME;
use crate::core::error::{OpenSpecError, Result};
use crate::core::spec_parser::{
    find_spec_updates, merge_delta_plan, parse_delta_spec, MergeResult, SpecUpdate,
};

pub fn run_archive(
    name: Option<&str>,
    yes: bool,
    skip_specs: bool,
    no_validate: bool,
) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let changes_dir = project_root.join(OPENSPEC_DIR_NAME).join("changes");
    let archive_dir = changes_dir.join("archive");
    let main_specs_dir = project_root.join(OPENSPEC_DIR_NAME).join("specs");

    if !changes_dir.exists() {
        return Err(OpenSpecError::Custom(
            "No OpenSpec changes directory found. Run 'openspec init' first.".to_string(),
        ));
    }

    let change_name = name.ok_or_else(|| {
        OpenSpecError::Custom(
            "Change name is required. Usage: openspec archive <change-name>".to_string(),
        )
    })?;

    let change_dir = changes_dir.join(change_name);

    if !change_dir.exists() || !change_dir.is_dir() {
        return Err(OpenSpecError::Custom(format!(
            "Change '{}' not found.",
            change_name
        )));
    }

    let progress = count_task_progress(&change_dir);
    println!("Task status: {}/{}", progress.0, progress.1);

    let incomplete_tasks = progress.1.saturating_sub(progress.0);
    if incomplete_tasks > 0 && !yes {
        eprintln!(
            "Warning: {} incomplete task(s) found. Use --yes to skip confirmation.",
            incomplete_tasks
        );
        return Err(OpenSpecError::Custom(
            "Archive cancelled due to incomplete tasks. Use --yes to force.".to_string(),
        ));
    }

    if !skip_specs {
        let spec_updates = find_spec_updates(&change_dir, &main_specs_dir);

        if !spec_updates.is_empty() {
            println!("\nSpecs to update:");
            for update in &spec_updates {
                let status = if update.target_exists {
                    "update"
                } else {
                    "create"
                };
                println!("  {}: {}", update.spec_name, status);
            }

            if !yes {
                println!("\nUse --yes to confirm spec updates and archive.");
                return Err(OpenSpecError::Custom(
                    "Archive cancelled. Use --yes to confirm.".to_string(),
                ));
            }

            let mut totals = MergeCounts::default();

            for update in &spec_updates {
                match apply_spec_update(update, change_name, no_validate) {
                    Ok(result) => {
                        totals.added += result.counts.added;
                        totals.modified += result.counts.modified;
                        totals.removed += result.counts.removed;
                        totals.renamed += result.counts.renamed;
                    }
                    Err(e) => {
                        return Err(OpenSpecError::Custom(format!(
                            "Failed to apply spec update for {}: {}",
                            update.spec_name, e
                        )));
                    }
                }
            }

            println!(
                "\nTotals: + {} ~ {} - {} → {}",
                totals.added, totals.modified, totals.removed, totals.renamed
            );
            println!("Specs updated successfully.");
        }
    } else {
        println!("Skipping spec updates (--skip-specs flag provided).");
    }

    let archive_name = format!("{}-{}", get_archive_date(), change_name);
    let archive_path = archive_dir.join(&archive_name);

    if archive_path.exists() {
        return Err(OpenSpecError::Custom(format!(
            "Archive '{}' already exists.",
            archive_name
        )));
    }

    std::fs::create_dir_all(&archive_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to create archive directory: {}", e)))?;

    move_directory(&change_dir, &archive_path)?;

    println!("Change '{}' archived as '{}'.", change_name, archive_name);

    Ok(())
}

#[derive(Default)]
struct MergeCounts {
    added: usize,
    modified: usize,
    removed: usize,
    renamed: usize,
}

fn apply_spec_update(
    update: &SpecUpdate,
    change_name: &str,
    _no_validate: bool,
) -> Result<MergeResult> {
    let change_content = std::fs::read_to_string(&update.source)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read source spec: {}", e)))?;

    let plan = parse_delta_spec(&change_content);

    let target_content = if update.target_exists {
        std::fs::read_to_string(&update.target).unwrap_or_default()
    } else {
        String::new()
    };

    let result = merge_delta_plan(&target_content, &plan, &update.spec_name, change_name)
        .map_err(OpenSpecError::Custom)?;

    let target_dir = update.target.parent().unwrap();
    std::fs::create_dir_all(target_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to create target directory: {}", e)))?;

    std::fs::write(&update.target, &result.rebuilt)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to write target spec: {}", e)))?;

    println!(
        "\nApplying changes to openspec/specs/{}/spec.md:",
        update.spec_name
    );
    if result.counts.added > 0 {
        println!("  + {} added", result.counts.added);
    }
    if result.counts.modified > 0 {
        println!("  ~ {} modified", result.counts.modified);
    }
    if result.counts.removed > 0 {
        println!("  - {} removed", result.counts.removed);
    }
    if result.counts.renamed > 0 {
        println!("  → {} renamed", result.counts.renamed);
    }

    Ok(result)
}

fn count_task_progress(change_dir: &std::path::Path) -> (usize, usize) {
    let tasks_path = change_dir.join("tasks.md");
    if !tasks_path.exists() {
        return (0, 0);
    }

    let content = std::fs::read_to_string(&tasks_path).unwrap_or_default();
    let mut total = 0;
    let mut completed = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("- [ ] ") || line.starts_with("* [ ] ") {
            total += 1;
        } else if line.starts_with("- [x] ")
            || line.starts_with("- [X] ")
            || line.starts_with("* [x] ")
            || line.starts_with("* [X] ")
        {
            total += 1;
            completed += 1;
        }
    }

    (completed, total)
}

fn get_archive_date() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

fn move_directory(src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    if let Err(e) = std::fs::rename(src, dest) {
        if e.raw_os_error() == Some(18) || e.raw_os_error() == Some(1) {
            copy_dir_recursive(src, dest)?;
            std::fs::remove_dir_all(src).map_err(|e| {
                OpenSpecError::Custom(format!("Failed to remove source directory: {}", e))
            })?;
        } else {
            return Err(OpenSpecError::Custom(format!(
                "Failed to move directory: {}",
                e
            )));
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dest)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to create directory: {}", e)))?;

    let entries = std::fs::read_dir(src)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read directory: {}", e)))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| OpenSpecError::Custom(format!("Failed to read entry: {}", e)))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)
                .map_err(|e| OpenSpecError::Custom(format!("Failed to copy file: {}", e)))?;
        }
    }

    Ok(())
}
