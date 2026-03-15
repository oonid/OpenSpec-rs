# OpenSpec-rs

A Rust port of [OpenSpec](https://github.com/Fission-AI/OpenSpec) - an AI-native system for spec-driven development.

## Why This Project?

This project exists for two reasons:

1. **Dogfooding OpenSpec** - The author wanted to try OpenSpec, so what better way than using OpenSpec to manage the development of porting OpenSpec itself to Rust? This project is a real-world use case demonstrating how OpenSpec works.

2. **Single Binary Bonus** - As a bonus, we get a standalone Rust binary that eliminates the npm/Node.js dependency. No more `npm install -g @fission-ai/openspec` - just download and run.

## Overview

OpenSpec-rs is a complete Rust port of the OpenSpec CLI. It provides the same functionality with better performance and simpler deployment - a single ~4MB binary with no runtime dependencies.

## Features

- **Single Binary** - No runtime dependencies, just download and run
- **Cross-Platform** - Linux (x64/ARM), macOS (x64/ARM), Windows (x64)
- **Fast** - Native Rust performance
- **Small Size** - ~4MB binary with LTO optimization
- **Full CLI** - All OpenSpec commands supported
- **AI Agent Skills** - Embedded skills for /opsx: commands

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

## Project Structure

```
OpenSpec-rs/
├── src/
│   ├── cli/           # CLI command implementations
│   ├── core/          # Core functionality (schema, artifact graph, etc.)
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

OpenSpec-rs generates configuration files for various AI tools:

- **OpenCode** - `.opencode/` directory with skills and commands
- **Claude** - `.claude/` directory with CLAUDE.md and commands
- **Cursor** - `.cursor/` directory with rules and commands
- **Windsurf** - `.windsurf/` directory with rules and commands

Use `openspec init --tools <tool>` to configure specific tools.

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
