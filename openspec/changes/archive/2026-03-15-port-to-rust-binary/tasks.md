## 1. Project Setup

- [x] 1.1 Initialize Cargo project with `cargo init`
- [x] 1.2 Add dependencies to Cargo.toml (clap, serde, serde_yaml, tokio, pulldown-cmark, walkdir, glob, termcolor, indicatif, inquire)
- [x] 1.3 Create module structure (cli/, core/, templates/, ai_tools/, telemetry/, utils/)
- [x] 1.4 Set up CI/CD for cross-platform builds (GitHub Actions)

## 2. Core Infrastructure

- [x] 2.1 Implement error types and Result alias
- [x] 2.2 Create config module for project and global configuration
- [x] 2.3 Implement XDG data directory resolution for global config
- [x] 2.4 Create colored output utilities (termcolor wrapper)
- [x] 2.5 Implement spinner/progress utilities (indicatif wrapper)

## 3. Schema System

- [x] 3.1 Define SchemaYaml struct with serde
- [x] 3.2 Implement schema.yaml parser
- [x] 3.3 Implement schema validation
- [x] 3.4 Implement schema resolution (project → user → package priority)
- [x] 3.5 Embed default `spec-driven` schema in binary
- [x] 3.6 Implement `listSchemas` and `listSchemasWithInfo` functions

## 4. Artifact Graph

- [x] 4.1 Define artifact dependency types
- [x] 4.2 Implement artifact status computation (ready/blocked/done)
- [x] 4.3 Implement `applyRequires` resolution
- [x] 4.4 Create status command output (text and JSON)

## 5. Spec Parser

- [x] 5.1 Implement markdown spec parser with pulldown-cmark
- [x] 5.2 Parse ADDED/MODIFIED/REMOVED/RENAMED sections
- [x] 5.3 Extract requirements with scenarios
- [x] 5.4 Implement spec merging for archive
- [x] 5.5 Implement spec file discovery (glob patterns)

## 6. CLI Commands - Phase 1

- [x] 6.1 Set up clap derive macros for CLI
- [x] 6.2 Implement `init` command with tool selection
- [x] 6.3 Implement `new change` command (kebab-case validation, directory creation, .openspec.yaml)
- [x] 6.4 Implement `status` command (--json support, artifact completion status)
- [x] 6.5 Implement `instructions` command (--json support, artifact + apply instructions)
- [x] 6.6 Implement `list` command (--specs, --json support)
- [x] 6.7 Implement `schemas` command

## 7. CLI Commands - Phase 2

- [x] 7.1 Implement `show` command (--json, --deltas-only support)
- [x] 7.2 Implement `validate` command (--all, --changes, --specs, --strict)
- [x] 7.3 Implement `config` command for viewing/editing settings

## 8. CLI Commands - Phase 3

- [x] 8.1 Implement `archive` command with spec merging
- [x] 8.2 Implement archive with --skip-specs option
- [x] 8.3 Implement archive with --yes confirmation skip
- [x] 8.4 Move archived changes to archive/ directory with timestamp

## 9. CLI Commands - Phase 4

- [x] 9.1 Implement `update` command
- [x] 9.2 Implement `--version` flag
- [x] 9.3 Implement `--help` for all commands
- [x] 9.4 Implement `--no-color` global flag

## 10. AI Tool Integration

- [x] 10.1 Define AI tool configurations (opencode, claude, cursor, etc.)
- [x] 10.2 Implement skill file generation
- [x] 10.3 Implement command file generation
- [x] 10.4 Create embedded templates for skills/commands
- [x] 10.5 Implement tool-specific output paths

## 11. Shell Completion

- [x] 11.1 Implement `completion generate` for bash
- [x] 11.2 Implement `completion generate` for zsh
- [x] 11.3 Implement `completion generate` for fish
- [x] 11.4 Implement `completion install` for each shell
- [x] 11.5 Implement `completion uninstall` for each shell
- [x] 11.6 Implement `__complete` internal command for dynamic completion

## 12. Testing

- [x] 12.1 Port test fixtures from vendor/OpenSpec/test/
- [x] 12.2 Unit tests for schema parsing
- [x] 12.3 Unit tests for spec parser
- [x] 12.4 Integration tests for init command
- [x] 12.5 Integration tests for archive with spec merging
- [x] 12.6 Integration tests for validation

## 13. Telemetry (Optional)

- [x] 13.1 Implement telemetry module with PostHog client
- [x] 13.2 Implement first-run notice display
- [x] 13.3 Implement environment variable opt-out (OPENSPEC_TELEMETRY, DO_NOT_TRACK)
- [x] 13.4 Implement CI auto-disable
- [x] 13.5 Implement async telemetry shutdown

## 14. Build & Release

- [x] 14.1 Configure release profile with LTO and strip
- [x] 14.2 Set up cross-compilation for Linux x64/ARM
- [x] 14.3 Set up cross-compilation for macOS x64/ARM
- [x] 14.4 Set up cross-compilation for Windows x64
- [x] 14.5 Create GitHub release workflow
- [x] 14.6 Test binary size target (<10MB)

## 15. Verification & Sync Skills

- [x] 15.1 Create templates/skills/verify-change.md with verification instructions
- [x] 15.2 Create templates/skills/sync-specs.md with sync instructions
- [x] 15.3 Add SKILL_VERIFY_CHANGE and SKILL_SYNC_SPECS constants to src/templates/skills.rs
- [x] 15.4 Add skill entries to SKILL_ENTRIES array in src/templates/skills.rs

## 16. Documentation Phase

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
