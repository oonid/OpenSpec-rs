use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "openspec")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Initialize OpenSpec in your project")]
    Init {
        #[arg(default_value = ".")]
        path: String,

        #[arg(
            long,
            help = "Configure AI tools non-interactively (all, none, or comma-separated list)"
        )]
        tools: Option<String>,

        #[arg(long, help = "Auto-cleanup legacy files without prompting")]
        force: bool,

        #[arg(long, help = "Override global config profile")]
        profile: Option<String>,
    },

    #[command(about = "Update OpenSpec instruction files")]
    Update {
        #[arg(default_value = ".")]
        path: String,

        #[arg(long, help = "Force update even when tools are up to date")]
        force: bool,
    },

    #[command(about = "List items (changes by default). Use --specs to list specs.")]
    List {
        #[arg(long, help = "List specs instead of changes")]
        specs: bool,

        #[arg(long, help = "List changes explicitly (default)")]
        changes: bool,

        #[arg(long, default_value = "recent", help = "Sort order: recent or name")]
        sort: String,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Display artifact completion status for a change")]
    Status {
        #[arg(long, help = "Change name to show status for")]
        change: Option<String>,

        #[arg(long, help = "Schema override")]
        schema: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Output enriched instructions for creating an artifact")]
    Instructions {
        #[arg(help = "Artifact ID (e.g., proposal, specs, design, tasks) or 'apply'")]
        artifact: Option<String>,

        #[arg(long, help = "Change name")]
        change: Option<String>,

        #[arg(long, help = "Schema override")]
        schema: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "List available workflow schemas with descriptions")]
    Schemas {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Show a change or spec")]
    Show {
        #[arg(help = "Item name to show")]
        name: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,

        #[arg(long, help = "Specify item type: change or spec")]
        r#type: Option<String>,

        #[arg(long, help = "Show only deltas (JSON only)")]
        deltas_only: bool,
    },

    #[command(about = "Validate changes and specs")]
    Validate {
        #[arg(help = "Item name to validate")]
        name: Option<String>,

        #[arg(long, help = "Validate all changes and specs")]
        all: bool,

        #[arg(long, help = "Validate all changes")]
        changes: bool,

        #[arg(long, help = "Validate all specs")]
        specs: bool,

        #[arg(long, help = "Specify item type: change or spec")]
        r#type: Option<String>,

        #[arg(long, help = "Enable strict validation mode")]
        strict: bool,

        #[arg(long, help = "Output validation results as JSON")]
        json: bool,
    },

    #[command(about = "Archive a completed change and update main specs")]
    Archive {
        #[arg(help = "Change name to archive")]
        name: Option<String>,

        #[arg(short, long, help = "Skip confirmation prompts")]
        yes: bool,

        #[arg(long, help = "Skip spec update operations")]
        skip_specs: bool,

        #[arg(long, help = "Skip validation")]
        no_validate: bool,
    },

    #[command(about = "View and modify global OpenSpec configuration")]
    Config {
        #[arg(long, help = "Set a configuration value (key=value)")]
        set: Option<String>,

        #[arg(long, help = "Get a configuration value")]
        get: Option<String>,

        #[arg(long, help = "List all configuration values")]
        list: bool,
    },

    #[command(subcommand, about = "Create new items")]
    New(NewCommands),

    #[command(subcommand, about = "Manage shell completions")]
    Completion(CompletionCommands),
}

#[derive(Subcommand, Debug)]
pub enum NewCommands {
    #[command(about = "Create a new change directory")]
    Change {
        #[arg(help = "Change name")]
        name: String,

        #[arg(long, help = "Description to add to README.md")]
        description: Option<String>,

        #[arg(long, help = "Workflow schema to use")]
        schema: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum CompletionCommands {
    #[command(about = "Generate completion script for a shell")]
    Generate {
        #[arg(help = "Shell type: bash, zsh, fish, or elvish")]
        shell: Option<String>,
    },

    #[command(about = "Install completion script for a shell")]
    Install {
        #[arg(help = "Shell type: bash, zsh, fish, or elvish")]
        shell: Option<String>,

        #[arg(long, help = "Show detailed installation output")]
        verbose: bool,
    },

    #[command(about = "Uninstall completion script for a shell")]
    Uninstall {
        #[arg(help = "Shell type: bash, zsh, fish, or elvish")]
        shell: Option<String>,

        #[arg(short, long, help = "Skip confirmation prompts")]
        yes: bool,
    },
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.no_color {
        unsafe { std::env::set_var("NO_COLOR", "1") };
    }

    match cli.command {
        Commands::Init {
            path,
            tools,
            force,
            profile,
        } => {
            crate::cli::init::run_init(&path, tools.as_deref(), force, profile.as_deref())?;
        }
        Commands::Update { path, force } => {
            println!("update: path={}, force={}", path, force);
        }
        Commands::List {
            specs,
            changes,
            sort,
            json,
        } => {
            crate::cli::list::run_list(specs, changes, &sort, json)?;
        }
        Commands::Status {
            change,
            schema,
            json,
        } => {
            crate::cli::status::run_status(change.as_deref(), schema.as_deref(), json)?;
        }
        Commands::Instructions {
            artifact,
            change,
            schema,
            json,
        } => {
            crate::cli::instructions::run_instructions(
                artifact.as_deref(),
                change.as_deref(),
                schema.as_deref(),
                json,
            )?;
        }
        Commands::Schemas { json } => {
            crate::cli::schemas::run_schemas(json)?;
        }
        Commands::Show {
            name,
            json,
            r#type,
            deltas_only,
        } => {
            let name = name.as_deref().ok_or_else(|| {
                crate::core::error::OpenSpecError::Custom(
                    "Missing required argument <name>".to_string(),
                )
            })?;
            crate::cli::show::run_show(name, r#type.as_deref(), json, deltas_only)?;
        }
        Commands::Validate {
            name,
            all,
            changes,
            specs,
            r#type,
            strict,
            json,
        } => {
            crate::cli::validate::run_validate(
                name.as_deref(),
                all,
                changes,
                specs,
                r#type.as_deref(),
                strict,
                json,
            )?;
        }
        Commands::Archive {
            name,
            yes,
            skip_specs,
            no_validate,
        } => {
            crate::cli::archive::run_archive(name.as_deref(), yes, skip_specs, no_validate)?;
        }
        Commands::Config { set, get, list } => {
            crate::cli::config::run_config(set.as_deref(), get.as_deref(), list)?;
        }
        Commands::New(NewCommands::Change {
            name,
            description,
            schema,
        }) => {
            crate::cli::new_change::run_new_change(
                &name,
                description.as_deref(),
                schema.as_deref(),
            )?;
        }
        Commands::Completion(cmd) => match cmd {
            CompletionCommands::Generate { shell } => {
                println!("completion generate: shell={:?}", shell);
            }
            CompletionCommands::Install { shell, verbose } => {
                println!("completion install: shell={:?}, verbose={}", shell, verbose);
            }
            CompletionCommands::Uninstall { shell, yes } => {
                println!("completion uninstall: shell={:?}, yes={}", shell, yes);
            }
        },
    }

    Ok(())
}
