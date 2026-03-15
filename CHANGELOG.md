# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2026-03-15

### Added
- docs/INSTALLATION.md - comprehensive installation guide with pre-built binaries, cargo install, and build from source instructions
- Main specs created at `openspec/specs/` with 7 capability specifications:
  - cli-commands (11 requirements)
  - workflow-engine (5 requirements)
  - spec-parser (4 requirements)
  - ai-tool-integration (4 requirements)
  - shell-completion (4 requirements)
  - config-management (4 requirements)
  - telemetry (4 requirements)
- "Support the Project" section in README.md with GitHub stargazers link

### Changed
- Archived `port-to-rust-binary` change after completing all 84 tasks
- Updated docs/STATE.md to reflect completed documentation phase and archived change

## [0.1.3] - 2026-03-14

### Changed
- Set Rust 1.75 as Minimum Supported Rust Version (MSRV)
- Pinned dependencies for Rust 1.75 compatibility
- Updated GitHub Actions to latest versions (actions/checkout@v6, etc.)

### Fixed
- ARM cross-compilation now uses `aarch64-linux-gnu-strip` for proper binary stripping

## [0.1.2] - 2026-03-14

### Fixed
- ARM cross-compilation strip command now targets correct architecture

## [0.1.1] - 2026-03-14

### Fixed
- CI now builds binary before running integration tests
- Test fixture loading handles missing vendor directory gracefully
- CI triggers on both `main` and `master` branches

### Changed
- Updated GitHub Actions to use Node.js 24 compatible versions

## [0.1.0] - 2026-03-14

### Added
- Initial release of OpenSpec-rs, a Rust port of OpenSpec

#### CLI Commands
- `init` - Initialize OpenSpec in a project with AI tool selection
- `new change` - Create a new change directory with .openspec.yaml
- `list` - List changes or specs with `--specs` and `--json` support
- `status` - Show artifact completion status with `--json` support
- `instructions` - Get instructions for creating artifacts with `--json` support
- `show` - Display a change or spec with `--json` and `--deltas-only` support
- `validate` - Validate changes and specs with `--all`, `--changes`, `--specs`, `--strict`
- `archive` - Archive completed changes with spec merging and `--skip-specs`, `--yes` options
- `update` - Update AI tool instruction files
- `config` - View and modify configuration settings
- `completion` - Generate shell completions for bash, zsh, fish

#### Core Features
- Schema system with YAML parsing and validation
- Schema resolution (project → user → package priority)
- Embedded `spec-driven` schema in binary
- Artifact dependency graph with status computation
- Markdown spec parser with ADDED/MODIFIED/REMOVED/RENAMED sections
- Spec merging for archive operation
- XDG data directory resolution for global config

#### AI Tool Integration
- Embedded skill templates for OpenSpec workflow:
  - `openspec-explore` - Thinking partner for exploring ideas
  - `openspec-propose` - Create change with proposal, specs, design, tasks
  - `openspec-apply-change` - Implement tasks from a change
  - `openspec-verify-change` - Verify implementation matches artifacts
  - `openspec-sync-specs` - Sync delta specs to main specs
  - `openspec-archive-change` - Archive completed change
- Support for multiple AI tools: OpenCode, Claude, Cursor, Windsurf, Codex
- Slash command generation (`/opsx:*` format)

#### Shell Completion
- Bash completion generation and installation
- Zsh completion generation and installation
- Fish completion generation and installation
- Dynamic completion via `__complete` internal command

#### Telemetry (Optional)
- Anonymous usage analytics via PostHog
- First-run notice display
- Environment variable opt-out (`OPENSPEC_TELEMETRY`, `DO_NOT_TRACK`)
- Automatic disable in CI environments

#### Build & Release
- Cross-platform builds: Linux (x64/ARM), macOS (x64/ARM), Windows (x64)
- Optimized release profile with LTO and binary stripping (~4MB binary)
- GitHub Actions release workflow

#### Testing
- 67 unit tests for schema parsing
- 17 integration tests for CLI commands
- Integration tests for archive with spec merging
- Integration tests for validation

---

## Version History Summary

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.4 | 2026-03-15 | Documentation complete, main specs created, change archived |
| 0.1.3 | 2026-03-14 | Rust 1.75 MSRV, pinned dependencies |
| 0.1.2 | 2026-03-14 | ARM cross-compilation fix |
| 0.1.1 | 2026-03-14 | CI build fixes |
| 0.1.0 | 2026-03-14 | Initial release |

[0.1.4]: https://github.com/oonid/OpenSpec-rs/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/oonid/OpenSpec-rs/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/oonid/OpenSpec-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/oonid/OpenSpec-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/oonid/OpenSpec-rs/releases/tag/v0.1.0
