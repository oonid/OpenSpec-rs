## Why

OpenSpec currently requires Node.js 20.19.0+ and npm, creating friction for users who don't have Node.js installed or prefer not to manage JavaScript toolchains. A single Rust binary eliminates the runtime dependency, provides faster startup and execution, and simplifies distribution across all platforms.

## What Changes

- **BREAKING**: Replace Node.js/TypeScript implementation with Rust
- Distribute as single static binary per platform (Linux, macOS, Windows)
- Maintain full CLI command compatibility
- Preserve all existing workflows and schemas
- Keep artifact structure unchanged (proposal.md, design.md, specs/, tasks.md)

## Capabilities

### New Capabilities

- `cli-commands`: Core CLI commands (init, update, list, show, validate, archive, status, instructions, new change)
- `workflow-engine`: Artifact graph resolution, dependency tracking, status computation
- `spec-parser`: Markdown parsing for specs with ADDED/MODIFIED/REMOVED sections
- `ai-tool-integration`: Generate skills/commands for supported AI tools (opencode, claude, cursor, etc.)
- `shell-completion`: Bash, Zsh, Fish completion generation and installation
- `config-management`: Project-level config.yaml and global config handling
- `telemetry`: Anonymous usage stats (optional, opt-out support)

### Modified Capabilities

- None (fresh implementation maintaining compatibility)

## Impact

- **Users**: No Node.js required, faster CLI, simpler installation
- **Distribution**: GitHub releases with binaries for all platforms
- **Development**: Rust codebase instead of TypeScript
- **Compatibility**: Must maintain CLI interface parity with v1.2.0
