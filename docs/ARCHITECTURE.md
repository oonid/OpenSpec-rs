# OpenSpec-rs Architecture

This document describes the internal architecture of OpenSpec-rs, a Rust port of OpenSpec.

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                           │
│  clap-based argument parsing, command routing               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        Core Layer                           │
│  Schema, Artifact Graph, Spec Parser, Config, Error         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Template Layer                         │
│  Embedded skills, commands, schemas                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       AI Tools Layer                        │
│  Generator for OpenCode, Claude, Cursor, etc.               │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs              # Entry point - thin wrapper around lib
├── lib.rs               # Library root - exports all modules
│
├── cli/                 # CLI Layer - Command implementations
│   ├── mod.rs           # Module exports
│   ├── args.rs          # clap CLI definitions and main run()
│   ├── init.rs          # `init` command
│   ├── update.rs        # `update` command
│   ├── list.rs          # `list` command
│   ├── status.rs        # `status` command
│   ├── instructions.rs  # `instructions` command
│   ├── schemas.rs       # `schemas` command
│   ├── show.rs          # `show` command
│   ├── validate.rs      # `validate` command
│   ├── archive.rs       # `archive` command
│   ├── config.rs        # `config` command
│   ├── new_change.rs    # `new change` command
│   └── completion.rs    # Shell completion commands
│
├── core/                # Core Layer - Business logic
│   ├── mod.rs           # Module exports + Result type
│   ├── error.rs         # Error types (OpenSpecError)
│   ├── config.rs        # Configuration management
│   ├── schema.rs        # Schema parsing, validation, resolution
│   ├── artifact.rs      # Artifact graph and status computation
│   └── spec_parser.rs   # Markdown spec parser (ADDED/MODIFIED/etc.)
│
├── templates/           # Template Layer - Embedded content
│   ├── mod.rs           # Template generation functions
│   ├── skills.rs        # Skill templates (explore, propose, etc.)
│   └── commands.rs      # Command templates and adapters
│
├── ai_tools/            # AI Tools Layer - Tool-specific output
│   ├── mod.rs
│   └── generator.rs     # Generate .opencode/, .claude/, etc.
│
├── telemetry/           # Optional telemetry
│   ├── mod.rs
│   └── client.rs        # PostHog client
│
└── utils/               # Shared utilities
    ├── mod.rs
    ├── output.rs        # Colored terminal output
    └── progress.rs      # Progress spinners/indicators
```

## Key Components

### 1. CLI Layer (`src/cli/`)

The CLI layer uses `clap` with derive macros for argument parsing.

**Entry Point**: `cli/args.rs::run()`

```rust
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { .. } => init::run_init(...),
        Commands::Status { .. } => status::run_status(...),
        // ...
    }
}
```

**Adding a New Command**:

1. Add variant to `Commands` enum in `args.rs`
2. Create new file in `cli/` (e.g., `mycommand.rs`)
3. Add module to `cli/mod.rs`
4. Add match arm in `run()`

### 2. Core Layer (`src/core/`)

The core layer contains business logic independent of CLI.

#### Schema System (`core/schema.rs`)

Handles workflow schema parsing and resolution:

```rust
pub struct SchemaYaml {
    pub name: String,
    pub artifacts: Vec<ArtifactDef>,
    // ...
}

pub fn load_schema(project_dir: &Path) -> Result<SchemaYaml>
pub fn resolve_schema(project_dir: &Path) -> Result<SchemaYaml>
```

**Schema Resolution Priority**:
1. Project: `openspec/schema.yaml`
2. User: `~/.config/openspec/schemas/*.yaml`
3. Package: Embedded `spec-driven` schema

#### Artifact Graph (`core/artifact.rs`)

Computes artifact status and dependencies:

```rust
pub struct ArtifactStatus {
    pub name: String,
    pub status: ArtifactState,  // Ready, Blocked, Done
    pub exists: bool,
    pub missing_dependencies: Vec<String>,
}

pub fn compute_status(schema: &SchemaYaml, change_dir: &Path) -> Vec<ArtifactStatus>
```

#### Spec Parser (`core/spec_parser.rs`)

Parses markdown specs with change sections:

```rust
pub struct SpecSection {
    pub section_type: SectionType,  // Added, Modified, Removed, Renamed
    pub requirements: Vec<Requirement>,
}

pub fn parse_spec(content: &str) -> Result<SpecDocument>
pub fn merge_specs(base: &SpecDocument, delta: &SpecDocument) -> SpecDocument
```

#### Configuration (`core/config.rs`)

Manages project and global configuration:

```rust
pub struct ProjectConfig {
    pub schema: Option<String>,
    pub tools: Vec<String>,
}

pub fn load_project_config(dir: &Path) -> Result<ProjectConfig>
pub fn get_global_config_dir() -> PathBuf  // XDG resolution
```

### 3. Template Layer (`src/templates/`)

Templates are embedded in the binary using `include_str!`.

**Skills** (`templates/skills.rs`):

```rust
pub const SKILL_EXPLORE: &str = include_str!("../../templates/skills/explore.md");
pub const SKILL_PROPOSE: &str = include_str!("../../templates/skills/propose.md");
// ...

pub fn generate_skill_content(name: &str, version: &str) -> String
```

**Commands** (`templates/commands.rs`):

```rust
pub fn generate_command(skill_name: &str, format: CommandFormat) -> GeneratedCommand
```

### 4. AI Tools Layer (`src/ai_tools/`)

Generates tool-specific configuration files:

```rust
pub struct ToolConfig {
    pub name: String,
    pub output_dir: String,
    pub skill_path: String,
    pub command_path: String,
}

pub fn generate_tool_files(tool: &ToolConfig, output_dir: &Path) -> Result<()>
```

**Supported Tools**:
- OpenCode (`.opencode/`)
- Claude (`.claude/`)
- Cursor (`.cursor/`)
- Windsurf (`.windsurf/`)
- Codex (`.codex/`)

### 5. Telemetry (`src/telemetry/`)

Optional anonymous usage analytics:

```rust
pub fn maybe_show_telemetry_notice()  // First-run notice
pub fn track_command(command: &str, version: &str)  // Track usage
pub fn flush_and_shutdown()  // Flush before exit
```

**Opt-out**:
- `OPENSPEC_TELEMETRY=false`
- `DO_NOT_TRACK=1`
- CI environment (auto-detected)

### 6. Utilities (`src/utils/`)

Shared utilities:

```rust
// output.rs - Colored terminal output
pub fn print_success(msg: &str)
pub fn print_error(msg: &str)
pub fn print_info(msg: &str)

// progress.rs - Progress indicators
pub fn create_spinner(msg: &str) -> ProgressBar
```

## Data Flow

### Init Command Flow

```
openspec init --tools opencode
        │
        ▼
┌───────────────────┐
│ Parse arguments   │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Create openspec/  │
│ directory         │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Write .openspec   │
│ _project.yaml     │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Generate AI tool  │
│ files (.opencode/)│
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Track telemetry   │
└───────────────────┘
```

### Status Command Flow

```
openspec status --change my-change --json
        │
        ▼
┌───────────────────┐
│ Find change dir   │
│ (openspec/changes/│
│  my-change/)      │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Load schema       │
│ (resolve priority)│
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Compute artifact  │
│ status            │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Output JSON or    │
│ formatted text    │
└───────────────────┘
```

### Archive Command Flow

```
openspec archive my-change
        │
        ▼
┌───────────────────┐
│ Validate change   │
│ (all artifacts    │
│  complete)        │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Parse delta specs │
│ (ADDED/MODIFIED/  │
│  REMOVED/RENAMED) │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Merge with main   │
│ specs             │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Move change to    │
│ openspec/archive/ │
└───────────────────┘
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `clap_complete` | Shell completion generation |
| `serde` | Serialization/deserialization |
| `serde_json` | JSON support |
| `serde_yaml` | YAML support |
| `tokio` | Async runtime (telemetry) |
| `pulldown-cmark` | Markdown parsing |
| `walkdir` | Directory traversal |
| `glob` | Glob pattern matching |
| `termcolor` | Colored output |
| `indicatif` | Progress spinners |
| `inquire` | Interactive prompts |
| `anyhow` | Error handling |
| `thiserror` | Custom error types |
| `dirs` | XDG directory resolution |
| `regex` | Regular expressions |
| `chrono` | Date/time |
| `ureq` | HTTP client (telemetry) |
| `uuid` | UUID generation |

## Error Handling

All errors go through `core/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum OpenSpecError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Change not found: {0}")]
    ChangeNotFound(String),
    
    // ...
}

pub type Result<T> = std::result::Result<T, OpenSpecError>;
```

## Testing

### Unit Tests

Located in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_parsing() {
        // ...
    }
}
```

### Integration Tests

Located in `tests/integration_tests.rs`:

```rust
#[test]
fn test_init_creates_openspec_directory() {
    let temp = tempfile::tempdir().unwrap();
    run_init(temp.path(), None, false, None).unwrap();
    assert!(temp.path().join("openspec").exists());
}
```

## Build Configuration

`Cargo.toml` release profile:

```toml
[profile.release]
lto = true        # Link-time optimization
strip = true      # Strip symbols
opt-level = "z"   # Optimize for size
```

Result: ~4MB binary.

## See Also

- [STATE.md](./STATE.md) - Implementation progress and sync status
- [CONTRIBUTING.md](../CONTRIBUTING.md) - How to contribute
- [INSTALLATION.md](./INSTALLATION.md) - Build and install instructions
