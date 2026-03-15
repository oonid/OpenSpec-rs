# OpenSpec-rs Project State

> **Purpose**: This file tracks the current implementation state of the Rust port and its compatibility with the upstream TypeScript version.

## Goal

Port OpenSpec (TypeScript/Node.js CLI) to a single Rust binary. No npm/Node.js dependency.

## Usage

This file serves as:
1. **Implementation tracker** - Shows which features are complete vs pending
2. **Sync reference** - Documents compatibility with upstream vendor/OpenSpec/
3. **Release notes** - Tracks version history and changes

### When to Update This File

- After completing a major feature or task group
- When syncing with upstream vendor/OpenSpec/ changes
- Before creating a new release
- When updating MSRV or dependencies

## Setup

- OpenSpec TS source: `vendor/OpenSpec/` (git submodule) - for reference/sync only
  - Current version: `v1.2.0-5-gafdca0d` (commit `afdca0d5dab1aa109cfd8848b2512333ccad60c3`)
  - Last synced: 2026-03-14
- **Rust binary**: Build with `cargo build --release`, binary at `target/release/openspec`
- Initialized OpenSpec for this project: `openspec/` and `.opencode/` directories exist

### Using the Binary

```bash
# Build
cargo build --release

# Run directly
./target/release/openspec --help
./target/release/openspec status --change "port-to-rust-binary"

# Or install locally
cargo install --path .
openspec --help
```

## Vendor/OpenSpec Compatibility

The `vendor/OpenSpec/` submodule contains the upstream TypeScript implementation. This section tracks compatibility for syncing after upgrades.

### Current Sync Status

| Component | Rust Status | TS Reference | Notes |
|-----------|-------------|--------------|-------|
| CLI Commands | Complete | `src/cli/` | All commands ported |
| Schema System | Complete | `src/core/schema/` | Schemas embedded in binary |
| Spec Parser | Complete | `src/core/parser/` | pulldown-cmark based |
| Artifact Graph | Complete | `src/core/artifact/` | Status computation |
| AI Tool Integration | Complete | `src/ai_tools/` | 6 skills embedded |
| Shell Completion | Complete | `src/cli/completion/` | bash/zsh/fish |
| Telemetry | Complete | `src/telemetry/` | PostHog, opt-out |

### How to Sync After Upgrading vendor/OpenSpec/

1. **Update the submodule**:
   ```bash
   cd vendor/OpenSpec
   git fetch origin
   git checkout <new-tag-or-commit>
   cd ../..
   git add vendor/OpenSpec
   ```

2. **Check for changes**:
   ```bash
   # Compare CLI commands
   diff -r vendor/OpenSpec/src/commands src/cli/
   
   # Compare schemas
   diff vendor/OpenSpec/src/schemas/ src/core/schema/
   
   # Compare templates
   diff vendor/OpenSpec/templates/ src/templates/
   ```

3. **Update affected components**:
   - New CLI flags → Update `src/cli/*.rs`
   - Schema changes → Update `src/core/schema.rs` and embedded schemas
   - Template changes → Update `src/templates/*.rs`
   - New AI tool configs → Update `src/ai_tools/config.rs`

4. **Update version tracking**:
   - Update "Current version" in this file
   - Update "Last synced" date
   - Add entries to CHANGELOG.md

5. **Run tests**:
   ```bash
   cargo test
   cargo clippy
   ```

### Files to Watch for Changes

| Upstream File | Rust Equivalent | Priority |
|---------------|-----------------|----------|
| `src/commands/*.ts` | `src/cli/*.rs` | High |
| `src/schemas/*.yaml` | `src/templates/schema.rs` | High |
| `templates/skills/*.md` | `src/templates/skills.rs` | Medium |
| `src/core/*.ts` | `src/core/*.rs` | Medium |
| `package.json` (version) | `Cargo.toml` | Low |

## Archived Change: `port-to-rust-binary`

Location: `openspec/changes/archive/2026-03-15-port-to-rust-binary/`

**Completed and archived on 2026-03-15.** All 84 tasks completed. Main specs created at `openspec/specs/`.

## Implementation Progress

**Done: Project Setup**
- [x] 1.1 Cargo project exists (fixed edition to 2021)
- [x] 1.2 Added dependencies to Cargo.toml
- [x] 1.3 Created module structure (cli/, core/, templates/, ai_tools/, telemetry/, utils/)
- [x] 1.4 Set up CI/CD for cross-platform builds (GitHub Actions)

**Done: Core Infrastructure**
- [x] 2.1 Implement error types and Result alias
- [x] 2.2 Create config module for project and global configuration
- [x] 2.3 Implement XDG data directory resolution for global config
- [x] 2.4 Create colored output utilities (termcolor wrapper)
- [x] 2.5 Implement spinner/progress utilities (indicatif wrapper)

**Done: Schema System**
- [x] 3.1 Define SchemaYaml struct with serde
- [x] 3.2 Implement schema.yaml parser
- [x] 3.3 Implement schema validation
- [x] 3.4 Implement schema resolution (project → user → package priority)
- [x] 3.5 Embed default `spec-driven` schema in binary
- [x] 3.6 Implement `listSchemas` and `listSchemasWithInfo` functions

**Done: Artifact Graph**
- [x] 4.1 Define artifact dependency types
- [x] 4.2 Implement artifact status computation (ready/blocked/done)
- [x] 4.3 Implement `applyRequires` resolution
- [x] 4.4 Create status command output (text and JSON)

**Done: Spec Parser**
- [x] 5.1 Implement markdown spec parser with pulldown-cmark
- [x] 5.2 Parse ADDED/MODIFIED/REMOVED/RENAMED sections
- [x] 5.3 Extract requirements with scenarios
- [x] 5.4 Implement spec merging for archive
- [x] 5.5 Implement spec file discovery (glob patterns)

**Done: CLI Commands - Phase 1**
- [x] 6.1 Set up clap derive macros for CLI
- [x] 6.2 Implement `init` command with tool selection
- [x] 6.3 Implement `new change` command (kebab-case validation, directory creation, .openspec.yaml)
- [x] 6.4 Implement `status` command (--json support, artifact completion status)
- [x] 6.5 Implement `instructions` command (--json support, artifact + apply instructions)
- [x] 6.6 Implement `list` command (--specs, --json support)
- [x] 6.7 Implement `schemas` command

**Done: CLI Commands - Phase 2**
- [x] 7.1 Implement `show` command (--json, --deltas-only support)
- [x] 7.2 Implement `validate` command (--all, --changes, --specs, --strict)
- [x] 7.3 Implement `config` command for viewing/editing settings

**Done: CLI Commands - Phase 3**
- [x] 8.1 Implement `archive` command with spec merging
- [x] 8.2 Implement archive with --skip-specs option
- [x] 8.3 Implement archive with --yes confirmation skip
- [x] 8.4 Move archived changes to archive/ directory with timestamp

**Done: CLI Commands - Phase 4**
- [x] 9.1 Implement `update` command
- [x] 9.2 Implement `--version` flag (via clap derive)
- [x] 9.3 Implement `--help` for all commands (via clap derive)
- [x] 9.4 Implement `--no-color` global flag (via clap derive with NO_COLOR env)

**Done: AI Tool Integration**
- [x] 10.1 Define AI tool configurations (opencode, claude, cursor, etc.)
- [x] 10.2 Implement skill file generation
- [x] 10.3 Implement command file generation
- [x] 10.4 Create embedded templates for skills/commands
- [x] 10.5 Implement tool-specific output paths
- [x] 10.6 Add verification & sync skills (templates/skills/verify-change.md, sync-specs.md)

**Done: Shell Completion**
- [x] 11.1 Implement `completion generate` for bash
- [x] 11.2 Implement `completion generate` for zsh
- [x] 11.3 Implement `completion generate` for fish
- [x] 11.4 Implement `completion install` for each shell
- [x] 11.5 Implement `completion uninstall` for each shell
- [x] 11.6 Implement `__complete` internal command for dynamic completion

**Done: Testing** (commit 4a01616)
- [x] 12.1 Port test fixtures from vendor/OpenSpec/test/
- [x] 12.2 Unit tests for schema parsing (67 unit tests passing)
- [x] 12.3 Unit tests for spec parser
- [x] 12.4 Integration tests for init command (17 integration tests passing)
- [x] 12.5 Integration tests for archive with spec merging
- [x] 12.6 Integration tests for validation
- Fixed change discovery to look for .openspec.yaml instead of proposal.md
- Fixed show command to fallback to README.md when proposal.md doesn't exist

**Done: Telemetry (Optional)** (commit 31cdae0)
- [x] 13.1 Implement telemetry module with PostHog client
- [x] 13.2 Implement first-run notice display
- [x] 13.3 Implement environment variable opt-out (OPENSPEC_TELEMETRY, DO_NOT_TRACK)
- [x] 13.4 Implement CI auto-disable
- [x] 13.5 Implement async telemetry shutdown (flush_and_shutdown)

**Done: Build & Release** (pre-configured in .github/workflows/release.yml)
- [x] 14.1 Configure release profile with LTO and strip (binary size: 4.0MB)
- [x] 14.2 Set up cross-compilation for Linux x64/ARM
- [x] 14.3 Set up cross-compilation for macOS x64/ARM
- [x] 14.4 Set up cross-compilation for Windows x64
- [x] 14.5 Create GitHub release workflow
- [x] 14.6 Test binary size target (<10MB) ✓ (4.0MB)

**Done: Verification & Sync Skills**
- [x] 15.1 Create templates/skills/verify-change.md with verification instructions
- [x] 15.2 Create templates/skills/sync-specs.md with sync instructions
- [x] 15.3 Add SKILL_VERIFY_CHANGE and SKILL_SYNC_SPECS constants to src/templates/skills.rs
- [x] 15.4 Add skill entries to SKILL_ENTRIES array in src/templates/skills.rs

**Done: Documentation Phase**
- [x] 16.1 Create README.md with project objective, sub-module, and vendor/ folder info
- [x] 16.2 Move scripts/ contents to docs/ and document each script
- [x] 16.3 Document openspec/ folder structure to docs/
- [x] 16.4 Document .opencode/ folder structure to docs/
- [x] 16.5 Move STATE.md to docs/STATE.md with usage explanation
- [x] 16.6 Update .gitignore to track openspec/ folder (removed from gitignore)
- [x] 16.7 Create CONTRIBUTING.md with contribution guidelines
- [x] 16.8 Create CHANGELOG.md based on releases
- [x] 16.9 Create docs/ARCHITECTURE.md explaining Rust project structure
- [x] 16.10 Create docs/INSTALLATION.md with install/build instructions

## Releases

| Version | Date | Notes |
|---------|------|-------|
| v0.1.4 | 2026-03-15 | Documentation complete, port-to-rust-binary archived, main specs created |
| v0.1.3 | 2026-03-14 | Rust 1.75 MSRV, pinned dependencies, GitHub Actions updates |
| v0.1.2 | 2026-03-14 | Fix ARM cross-compilation strip |
| v0.1.1 | 2026-03-14 | Fix CI build before tests |
| v0.1.0 | 2026-03-14 | Initial release |

## Current Version: 0.1.4

- MSRV: Rust 1.75+
- Pinned dependencies for Rust 1.75 compatibility
- GitHub Actions updated to latest versions
- All 6 skills embedded: explore, propose, apply-change, archive-change, verify-change, sync-specs
- Documentation complete (README, CONTRIBUTING, CHANGELOG, ARCHITECTURE, INSTALLATION)
- Main specs created from port-to-rust-binary change

## See Also

- [README.md](../README.md) - Project overview
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [CHANGELOG.md](../CHANGELOG.md) - Version history
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Rust project structure
- [INSTALLATION.md](./INSTALLATION.md) - Build and install instructions
