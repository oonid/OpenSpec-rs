use clap::{Parser, Subcommand};

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    #[command(about = "Show resolved template paths for all artifacts in a schema")]
    Templates {
        #[arg(long, help = "Schema to use (default: spec-driven)")]
        schema: Option<String>,

        #[arg(long, help = "Output as JSON mapping artifact IDs to template paths")]
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

    #[command(subcommand, about = "Set up and inspect local context stores")]
    ContextStore(ContextStoreCommands),

    #[command(subcommand, about = "Create and inspect coordinated initiatives")]
    Initiative(InitiativeCommands),

    #[command(subcommand, about = "Set up and inspect coordination workspaces")]
    Workspace(WorkspaceCommands),

    #[command(subcommand, about = "Manage workflow schemas [experimental]")]
    Schema(SchemaCommands),

    #[command(subcommand, about = "Set checked-in OpenSpec metadata")]
    Set(SetCommands),
}

#[derive(Subcommand, Debug)]
pub enum SetCommands {
    #[command(about = "Link a change to an initiative")]
    Change {
        #[arg(help = "Change name")]
        name: Option<String>,

        #[arg(long, help = "Initiative id to link")]
        initiative: Option<String>,

        #[arg(long, help = "Context store id from the local registry")]
        store: Option<String>,

        #[arg(long = "store-path", help = "Existing local context store root")]
        store_path: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum SchemaCommands {
    #[command(about = "Show where a schema resolves from")]
    Which {
        #[arg(help = "Schema name")]
        name: Option<String>,

        #[arg(long, help = "List all schemas with their resolution sources")]
        all: bool,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Validate a schema structure and templates")]
    Validate {
        #[arg(help = "Schema name (default: spec-driven)")]
        name: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Copy an existing schema to project for customization")]
    Fork {
        #[arg(help = "Source schema to copy")]
        source: String,

        #[arg(help = "Destination schema name (defaults to <source>-custom)")]
        name: Option<String>,

        #[arg(long, help = "Overwrite existing destination")]
        force: bool,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Create a new project-local schema")]
    Init {
        #[arg(help = "Schema name")]
        name: String,

        #[arg(long, help = "Overwrite existing schema")]
        force: bool,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
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

#[derive(Subcommand, Debug)]
pub enum ContextStoreCommands {
    #[command(about = "Create and register a local context store")]
    Setup {
        #[arg(help = "Context store id")]
        id: Option<String>,
        #[arg(
            long,
            help = "Context store folder path; defaults to OpenSpec managed local data"
        )]
        path: Option<String>,
        #[arg(
            long = "init-git",
            help = "Initialize a Git repository in the context store"
        )]
        init_git: bool,
        #[arg(
            long = "allow-inside-git-repository",
            help = "Allow the context store path to be inside an existing Git repository"
        )]
        allow_inside_git_repository: bool,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Register an existing local context store")]
    Register {
        #[arg(help = "Context store folder path")]
        path: Option<String>,
        #[arg(long, help = "Context store id; defaults to metadata or folder name")]
        id: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Forget a local context-store registration without deleting files")]
    Unregister {
        #[arg(help = "Context store id")]
        id: String,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Forget a local context-store registration and delete its local folder")]
    Remove {
        #[arg(help = "Context store id")]
        id: String,
        #[arg(long, help = "Confirm local context-store folder deletion")]
        yes: bool,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "List locally registered context stores")]
    List {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Check local context-store registration and metadata")]
    Doctor {
        #[arg(help = "Context store id")]
        id: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum InitiativeCommands {
    #[command(about = "Create an initiative in a context store")]
    Create {
        #[arg(help = "Initiative id")]
        id: Option<String>,
        #[arg(long, help = "Initiative title")]
        title: Option<String>,
        #[arg(long, help = "Initiative summary")]
        summary: Option<String>,
        #[arg(long, help = "Context store id from the local registry")]
        store: Option<String>,
        #[arg(long = "store-path", help = "Existing local context store root")]
        store_path: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Show where an initiative lives and how to read it")]
    Show {
        #[arg(help = "Initiative id")]
        id: String,
        #[arg(long)]
        store: Option<String>,
        #[arg(long = "store-path")]
        store_path: Option<String>,
        #[arg(long)]
        json: bool,
    },
    #[command(
        visible_alias = "ls",
        about = "List initiatives across registered context stores"
    )]
    List {
        #[arg(long)]
        store: Option<String>,
        #[arg(long = "store-path")]
        store_path: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceCommands {
    #[command(visible_alias = "ls", about = "List known OpenSpec workspaces")]
    List {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Link an existing repo or folder to a workspace")]
    Link {
        #[arg(help = "Link name or path")]
        name_or_path: Option<String>,
        #[arg(help = "Path (when first arg is a name)")]
        path: Option<String>,
        #[arg(long, help = "Workspace name from known local workspace views")]
        workspace: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Update the local path for an existing workspace link")]
    Relink {
        #[arg(help = "Link name")]
        name: String,
        #[arg(help = "New path")]
        path: String,
        #[arg(long, help = "Workspace name from known local workspace views")]
        workspace: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Check what a workspace can resolve on this machine")]
    Doctor {
        #[arg(long, help = "Workspace name from known local workspace views")]
        workspace: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    #[command(about = "Set up a workspace and link existing repos or folders")]
    Setup {
        #[arg(long)]
        name: Option<String>,
        #[arg(
            long = "link",
            help = "Repo/folder link: <path> or <name>=<path> (repeatable)"
        )]
        links: Vec<String>,
        #[arg(long)]
        opener: Option<String>,
        #[arg(long)]
        tools: Option<String>,
        #[arg(long)]
        json: bool,
    },
    #[command(about = "Refresh workspace-local OpenSpec guidance and agent skills")]
    Update {
        #[arg(help = "Workspace name")]
        name: Option<String>,
        #[arg(long)]
        workspace: Option<String>,
        #[arg(long)]
        tools: Option<String>,
        #[arg(long)]
        json: bool,
    },
    #[command(about = "Open a workspace in an agent or VS Code editor")]
    Open {
        #[arg(help = "Workspace name")]
        name: Option<String>,
        #[arg(long)]
        workspace: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        editor: bool,
        #[arg(long)]
        json: bool,
    },
}

fn get_command_path(cli: &Cli) -> String {
    match &cli.command {
        Commands::Init { .. } => "init".to_string(),
        Commands::Update { .. } => "update".to_string(),
        Commands::List { .. } => "list".to_string(),
        Commands::Status { .. } => "status".to_string(),
        Commands::Instructions { .. } => "instructions".to_string(),
        Commands::Schemas { .. } => "schemas".to_string(),
        Commands::Templates { .. } => "templates".to_string(),
        Commands::Show { .. } => "show".to_string(),
        Commands::Validate { .. } => "validate".to_string(),
        Commands::Archive { .. } => "archive".to_string(),
        Commands::Config { .. } => "config".to_string(),
        Commands::New(NewCommands::Change { .. }) => "new:change".to_string(),
        Commands::Completion(cmd) => match cmd {
            CompletionCommands::Generate { .. } => "completion:generate".to_string(),
            CompletionCommands::Install { .. } => "completion:install".to_string(),
            CompletionCommands::Uninstall { .. } => "completion:uninstall".to_string(),
        },
        Commands::ContextStore(cmd) => match cmd {
            ContextStoreCommands::Setup { .. } => "context-store:setup".to_string(),
            ContextStoreCommands::Register { .. } => "context-store:register".to_string(),
            ContextStoreCommands::Unregister { .. } => "context-store:unregister".to_string(),
            ContextStoreCommands::Remove { .. } => "context-store:remove".to_string(),
            ContextStoreCommands::List { .. } => "context-store:list".to_string(),
            ContextStoreCommands::Doctor { .. } => "context-store:doctor".to_string(),
        },
        Commands::Initiative(cmd) => match cmd {
            InitiativeCommands::Create { .. } => "initiative:create".to_string(),
            InitiativeCommands::Show { .. } => "initiative:show".to_string(),
            InitiativeCommands::List { .. } => "initiative:list".to_string(),
        },
        Commands::Workspace(cmd) => match cmd {
            WorkspaceCommands::List { .. } => "workspace:list".to_string(),
            WorkspaceCommands::Link { .. } => "workspace:link".to_string(),
            WorkspaceCommands::Relink { .. } => "workspace:relink".to_string(),
            WorkspaceCommands::Doctor { .. } => "workspace:doctor".to_string(),
            WorkspaceCommands::Setup { .. } => "workspace:setup".to_string(),
            WorkspaceCommands::Update { .. } => "workspace:update".to_string(),
            WorkspaceCommands::Open { .. } => "workspace:open".to_string(),
        },
        Commands::Schema(cmd) => match cmd {
            SchemaCommands::Which { .. } => "schema:which".to_string(),
            SchemaCommands::Validate { .. } => "schema:validate".to_string(),
            SchemaCommands::Fork { .. } => "schema:fork".to_string(),
            SchemaCommands::Init { .. } => "schema:init".to_string(),
        },
        Commands::Set(cmd) => match cmd {
            SetCommands::Change { .. } => "set:change".to_string(),
        },
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Handle __complete command before clap parsing (to avoid clap_complete issues)
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "__complete" {
        crate::cli::completion::run_complete(&args[2])?;
        return Ok(());
    }

    let cli = Cli::parse();

    if cli.no_color {
        unsafe { std::env::set_var("NO_COLOR", "1") };
    }

    // One-time best-effort migration of legacy macOS global config to the upstream-aligned
    // location (no-op on other platforms). Must run before telemetry reads the config.
    crate::core::config::migrate_legacy_macos_global_config();

    // Telemetry: show first-run notice and track command
    crate::telemetry::maybe_show_telemetry_notice();
    let command_path = get_command_path(&cli);
    crate::telemetry::track_command(&command_path, VERSION);

    match cli.command {
        Commands::Init {
            path,
            tools,
            force,
            profile,
        } => {
            crate::cli::init::run_init(&path, tools.as_deref(), force, profile.as_deref())?;
        }
        Commands::Update { path: _path, force } => {
            crate::cli::update::run_update(force)?;
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
        Commands::Templates { schema, json } => {
            crate::cli::templates::run(schema.as_deref(), json)?;
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
                crate::cli::completion::run_completion_generate(shell.as_deref())?;
            }
            CompletionCommands::Install { shell, verbose } => {
                crate::cli::completion::run_completion_install(shell.as_deref(), verbose)?;
            }
            CompletionCommands::Uninstall { shell, yes: _yes } => {
                crate::cli::completion::run_completion_uninstall(shell.as_deref(), _yes)?;
            }
        },
        Commands::ContextStore(cmd) => {
            crate::cli::context_store::run(cmd)?;
        }
        Commands::Initiative(cmd) => {
            crate::cli::initiative::run(cmd)?;
        }
        Commands::Workspace(cmd) => {
            crate::cli::workspace::run(cmd)?;
        }
        Commands::Schema(cmd) => {
            crate::cli::schema::run(cmd)?;
        }
        Commands::Set(cmd) => {
            crate::cli::set::run(cmd)?;
        }
    }

    // Telemetry: flush pending events before exit
    crate::telemetry::flush_and_shutdown();

    Ok(())
}
