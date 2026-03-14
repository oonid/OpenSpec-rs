use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::cli::args::Cli;
use crate::core::config::OPENSPEC_DIR_NAME;

pub fn run_completion_generate(shell: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let shell = shell.unwrap_or("bash");

    let target_shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "elvish" => Shell::Elvish,
        _ => {
            eprintln!(
                "Unknown shell: {}. Supported: bash, zsh, fish, elvish",
                shell
            );
            std::process::exit(1);
        }
    };

    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    generate(target_shell, &mut cmd, name, &mut std::io::stdout());

    Ok(())
}

pub fn run_completion_install(
    shell: Option<&str>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let shell = shell.unwrap_or("bash");

    let (target_shell, completion_dir, completion_file) = match shell.to_lowercase().as_str() {
        "bash" => {
            let dir = dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("bash-completion")
                .join("completions");
            (Shell::Bash, dir, format!("{}.bash", "openspec"))
        }
        "zsh" => {
            let dir = std::path::PathBuf::from("/usr/local/share/zsh/site-functions");
            (Shell::Zsh, dir, format!("_{}", "openspec"))
        }
        "fish" => {
            let dir = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("fish")
                .join("completions");
            (Shell::Fish, dir, format!("{}.fish", "openspec"))
        }
        "elvish" => {
            eprintln!("Elvish completion install not yet supported");
            std::process::exit(1);
        }
        _ => {
            eprintln!("Unknown shell: {}. Supported: bash, zsh, fish", shell);
            std::process::exit(1);
        }
    };

    std::fs::create_dir_all(&completion_dir)?;

    let completion_path = completion_dir.join(&completion_file);

    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    let mut buffer = Vec::new();
    generate(target_shell, &mut cmd, name.clone(), &mut buffer);

    std::fs::write(&completion_path, &buffer)?;

    if verbose {
        println!(
            "Completion script written to: {}",
            completion_path.display()
        );
        println!("\nTo activate completions:");
        match shell.to_lowercase().as_str() {
            "bash" => {
                println!("  source {}", completion_path.display());
                println!("  Or add to your ~/.bashrc:");
                println!(
                    "  [ -f {} ] && source {}",
                    completion_path.display(),
                    completion_path.display()
                );
            }
            "zsh" => {
                println!("  The completion file is installed to:");
                println!("  {}", completion_path.display());
                println!("  You may need to restart your shell or run:");
                println!("  autoload -U compinit && compinit");
            }
            "fish" => {
                println!("  Restart your fish shell or run:");
                println!("  source {}", completion_path.display());
            }
            _ => {}
        }
    } else {
        println!("Completion installed for {}", shell);
    }

    Ok(())
}

pub fn run_completion_uninstall(
    shell: Option<&str>,
    _yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let shell = shell.unwrap_or("bash");

    let completion_file = match shell.to_lowercase().as_str() {
        "bash" => dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("bash-completion")
            .join("completions")
            .join(format!("{}.bash", "openspec")),
        "zsh" => std::path::PathBuf::from("/usr/local/share/zsh/site-functions")
            .join(format!("_{}", "openspec")),
        "fish" => dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("fish")
            .join("completions")
            .join(format!("{}.fish", "openspec")),
        "elvish" => {
            eprintln!("Elvish completion uninstall not yet supported");
            std::process::exit(1);
        }
        _ => {
            eprintln!("Unknown shell: {}. Supported: bash, zsh, fish", shell);
            std::process::exit(1);
        }
    };

    if completion_file.exists() {
        std::fs::remove_file(&completion_file)?;
        println!("Completion uninstalled for {}", shell);
    } else {
        println!("No completion file found for {}", shell);
    }

    Ok(())
}

pub fn run_complete(complete_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = std::env::current_dir()?;

    match complete_type.to_lowercase().as_str() {
        "changes" => {
            let changes_dir = project_root.join(OPENSPEC_DIR_NAME).join("changes");
            if changes_dir.exists() {
                for entry in std::fs::read_dir(&changes_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name != "archive" && !name.starts_with('.') {
                            let metadata_path = entry.path().join(".openspec.yaml");
                            if metadata_path.exists() {
                                println!("{}\tactive change", name);
                            }
                        }
                    }
                }
            }
        }
        "specs" => {
            let specs_dir = project_root.join(OPENSPEC_DIR_NAME).join("specs");
            if specs_dir.exists() {
                for entry in std::fs::read_dir(&specs_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let spec_path = entry.path().join("spec.md");
                        if spec_path.exists() {
                            println!("{}\tspecification", name);
                        }
                    }
                }
            }
        }
        "archived-changes" => {
            let archive_dir = project_root
                .join(OPENSPEC_DIR_NAME)
                .join("changes")
                .join("archive");
            if archive_dir.exists() {
                for entry in std::fs::read_dir(&archive_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        println!("{}\tarchived change", name);
                    }
                }
            }
        }
        _ => {
            std::process::exit(1);
        }
    }

    Ok(())
}
