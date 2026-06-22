# OpenSpec-rs

A Rust port of [OpenSpec](https://github.com/Fission-AI/OpenSpec) - an AI-native system for spec-driven development.

## Why This Project?

This project exists for two reasons:

1. **Dogfooding OpenSpec** - The author wanted to try OpenSpec, so what better way than using OpenSpec to manage the development of porting OpenSpec itself to Rust? This project is a real-world use case demonstrating how OpenSpec works.

2. **Single Binary Bonus** - As a bonus, we get a standalone Rust binary that eliminates the npm/Node.js dependency. No more `npm install -g @fission-ai/openspec` - just download and run.

## Overview

OpenSpec-rs is a complete Rust port of the OpenSpec CLI, tracking upstream **v1.4.1**. It provides the same functionality with better performance and simpler deployment - a single ~4MB binary with no runtime dependencies.

## Features

- **Single Binary** - No runtime dependencies, just download and run
- **Cross-Platform** - Linux (x64/ARM), macOS (x64/ARM), Windows (x64)
- **Fast** - Native Rust performance
- **Small Size** - ~4MB binary with LTO optimization
- **Full CLI** - All OpenSpec commands supported
- **30 AI Tools** - Skill generation for Claude, Cursor, OpenCode, Copilot, Codex, Gemini, and many more
- **AI Agent Skills** - Embedded skills for /opsx: commands
- **Coordination (beta)** - Context stores, initiatives, and multi-repo workspaces for cross-repo planning

## Installation

### From Releases

Download the latest release for your platform from the [Releases](https://github.com/oonid/OpenSpec-rs/releases) page.

### From Source

```bash
# Clone the repository
git clone https://github.com/oonid/OpenSpec-rs.git
cd OpenSpec-rs

# Initialize submodule (optional, for reference)
git submodule update --init --recursive

# Build
cargo build --release

# Binary will be at target/release/openspec
```

**Requirements:** Rust 1.75 or later

## Quick Start

### CLI Commands

```bash
# Initialize OpenSpec in your project
openspec init .

# Create a new change
openspec new change add-dark-mode

# Check status
openspec status --change add-dark-mode

# List changes
openspec list

# Archive when done
openspec archive add-dark-mode
```

### AI Agent Commands (/opsx:)

OpenSpec works best with AI agents. After running `openspec init --tools opencode`, you can use these commands:

```
/opsx:explore          # Think through ideas before proposing
/opsx:propose <name>   # Create a change with proposal, specs, design, tasks
/opsx:apply            # Implement tasks from the current change
/opsx:verify           # Verify implementation matches artifacts
/opsx:sync             # Sync delta specs to main specs
/opsx:archive          # Finalize and archive the change
```

**Typical workflow:**
```
You: /opsx:explore add-auth
AI:  [explores the problem space with you]

You: /opsx:propose add-auth
AI:  Created openspec/changes/add-auth/
     ✓ proposal.md
     ✓ specs/
     ✓ design.md
     ✓ tasks.md

You: /opsx:apply
AI:  [implements tasks one by one]

You: /opsx:archive
AI:  Archived. Specs updated.
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `init [path]` | Initialize OpenSpec in a project |
| `new change <name>` | Create a new change directory |
| `list` | List changes (or specs with `--specs`) |
| `status` | Show artifact completion status |
| `instructions [artifact]` | Get instructions for creating artifacts |
| `schemas` | List available workflow schemas |
| `show [item]` | Show a change or spec |
| `validate [item]` | Validate changes and specs |
| `archive [change]` | Archive a completed change |
| `update [path]` | Update AI tool instruction files |
| `config` | View and modify configuration |
| `completion` | Generate shell completions |
| `context-store` | Set up and inspect local context stores (beta) |
| `initiative` | Create and inspect coordinated initiatives (beta) |
| `workspace` | Set up and inspect coordination workspaces (beta) |
| `templates` | Show resolved template paths for a schema |
| `schema` | Inspect, validate, fork, or init workflow schemas |
| `set change` | Link a repo-local change to an initiative |

## Coordination: Context Stores, Initiatives & Workspaces (beta)

For work that spans multiple repositories, OpenSpec-rs adds three coordination
primitives (on-disk formats are compatible with the upstream npm CLI):

- **Context stores** — local, git-backed stores that hold cross-repo planning context.
  ```bash
  openspec context-store setup team-context
  openspec context-store list
  ```
- **Initiatives** — durable cross-team/cross-repo intent (requirements, design, decisions, tasks) living inside a context store.
  ```bash
  openspec initiative create roadmap --title "Q3 Roadmap" --summary "..." --store team-context
  openspec initiative list
  ```
- **Workspaces** — a local working view that links repos/folders and opens them in your agent or editor.
  ```bash
  openspec workspace setup --name platform --link ../api --link ../web
  openspec workspace open platform --editor
  ```

These are beta; existing single-project workflows are unaffected.

## Project Structure

```
OpenSpec-rs/
├── src/
│   ├── cli/           # CLI command implementations
│   ├── core/          # Core functionality (schema, artifact graph, etc.)
│   │   ├── context_store/   # Context store registry + operations (beta)
│   │   ├── collections/     # Initiatives collection (beta)
│   │   └── workspace/       # Workspace data, openers, skills (beta)
│   ├── templates/     # Embedded templates for skills and commands
│   ├── ai_tools/      # AI tool configuration generators
│   ├── telemetry/     # Optional telemetry (PostHog)
│   └── utils/         # Utilities (output, errors, etc.)
├── templates/
│   └── skills/        # Skill template markdown files
├── tests/
│   └── integration_tests.rs
├── vendor/
│   └── OpenSpec/      # Git submodule - original TypeScript source
├── openspec/          # OpenSpec project files (changes, specs)
├── .opencode/         # OpenCode AI tool configuration
└── Cargo.toml
```

## Sub-modules

### vendor/OpenSpec/

This is a git submodule pointing to the original [OpenSpec TypeScript repository](https://github.com/Fission-AI/OpenSpec). It's used as a reference for porting features and ensuring compatibility.

```bash
# Initialize the submodule
git submodule update --init --recursive
```

## AI Tool Integration

OpenSpec-rs generates skill/command configuration for **30 AI tools**, including:

- **Claude Code** - `.claude/`
- **OpenCode** - `.opencode/`
- **Cursor** - `.cursor/`
- **Windsurf** - `.windsurf/`
- **GitHub Copilot** - `.github/`
- **Codex**, **Gemini CLI**, **Qwen Code**, **Kiro**, **Kimi CLI**, **Mistral Vibe**, and more

`openspec init` auto-detects the tools already present in your project; use
`openspec init --tools <all|none|comma-separated-list>` to choose explicitly.

## Shell Completions

Generate shell completions:

```bash
# Bash
openspec completion generate bash > /etc/bash_completion.d/openspec

# Zsh
openspec completion generate zsh > "${fpath[1]}/_openspec"

# Fish
openspec completion generate fish > ~/.config/fish/completions/openspec.fish
```

Or install them in one step (opt-in; not installed automatically by `init`/`update`):

```bash
openspec completion install zsh
```

## Development

```bash
# Run tests
cargo test

# Run with release optimizations
cargo run --release -- <command>

# Check code
cargo clippy
cargo fmt --check
```

## Telemetry

OpenSpec-rs collects anonymous usage telemetry to improve the tool. To opt out:

```bash
export OPENSPEC_TELEMETRY=false
# or
export DO_NOT_TRACK=1
```

Telemetry is automatically disabled in CI environments.

## Support the Project

If you find this project useful, please consider giving it a ⭐ on [GitHub](https://github.com/oonid/OpenSpec-rs/stargazers)!

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

This project is a Rust port of [OpenSpec](https://github.com/Fission-AI/OpenSpec) by Fission AI.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
