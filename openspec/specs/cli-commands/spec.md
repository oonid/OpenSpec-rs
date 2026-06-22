## Purpose

CLI commands for the OpenSpec Rust binary. Provides all commands for initializing projects, managing changes, viewing status, and archiving completed work.
## Requirements
### Requirement: CLI Init Command

The system SHALL provide an `openspec init [path]` command that initializes OpenSpec in a project directory.

#### Scenario: Initialize in current directory
- **WHEN** user runs `openspec init .`
- **THEN** system creates `openspec/` directory with `specs/` and `changes/` subdirectories
- **AND** system generates AI tool configuration files for selected tools

#### Scenario: Initialize with tools flag
- **WHEN** user runs `openspec init --tools opencode`
- **THEN** system initializes OpenSpec structure
- **AND** system generates configuration for opencode tool only

#### Scenario: Initialize in non-existent directory
- **WHEN** user runs `openspec init /path/to/new/project`
- **THEN** system creates the directory if it doesn't exist
- **AND** system initializes OpenSpec structure within it

### Requirement: CLI New Change Command

The system SHALL provide an `openspec new change <name>` command that creates a new change directory.

#### Scenario: Create new change
- **WHEN** user runs `openspec new change add-dark-mode`
- **THEN** system creates `openspec/changes/add-dark-mode/` directory
- **AND** system creates `.openspec.yaml` with schema configuration

#### Scenario: Create change with schema option
- **WHEN** user runs `openspec new change my-feature --schema spec-driven`
- **THEN** system creates change directory with specified schema

### Requirement: CLI Status Command

The system SHALL provide an `openspec status --change <name>` command that displays artifact completion status.

#### Scenario: Show status as JSON
- **WHEN** user runs `openspec status --change add-dark-mode --json`
- **THEN** system outputs JSON with `applyRequires` and `artifacts` array
- **AND** each artifact has `id`, `outputPath`, and `status` fields

#### Scenario: Show status with blocked artifacts
- **WHEN** user runs `openspec status --change incomplete-change`
- **THEN** system displays artifacts with `blocked` status and their `missingDeps`

### Requirement: CLI Instructions Command

The system SHALL provide an `openspec instructions <artifact> --change <name>` command that outputs artifact creation guidance.

#### Scenario: Get artifact instructions as JSON
- **WHEN** user runs `openspec instructions proposal --change my-change --json`
- **THEN** system outputs JSON with `template`, `instruction`, `outputPath`, and `dependencies`

### Requirement: CLI List Command

The system SHALL provide an `openspec list` command that lists changes or specs.

#### Scenario: List changes
- **WHEN** user runs `openspec list`
- **THEN** system displays all active changes in `openspec/changes/`

#### Scenario: List specs
- **WHEN** user runs `openspec list --specs`
- **THEN** system displays all specs in `openspec/specs/`

#### Scenario: List as JSON
- **WHEN** user runs `openspec list --json`
- **THEN** system outputs JSON array of changes

### Requirement: CLI Show Command

The system SHALL provide an `openspec show [item-name]` command that displays change or spec details.

#### Scenario: Show change as JSON
- **WHEN** user runs `openspec show add-dark-mode --json`
- **THEN** system outputs JSON with change details including artifacts

### Requirement: CLI Validate Command

The system SHALL provide an `openspec validate [item-name]` command that validates changes and specs, and SHALL emit actionable hints for common authoring mistakes.

#### Scenario: Validate all changes
- **WHEN** user runs `openspec validate --all`
- **THEN** system validates all changes and specs
- **AND** system reports validation errors if any

#### Scenario: Hint when SHALL/MUST only in header
- **WHEN** a requirement has SHALL/MUST only in its `### Requirement:` header line and not in the requirement body
- **THEN** system reports a hint to move the keyword onto the requirement body line
- **AND** the hint is more specific than the generic missing-normative-keyword error

### Requirement: CLI Archive Command

The system SHALL provide an `openspec archive [change-name]` command that archives a completed change.

#### Scenario: Archive change
- **WHEN** user runs `openspec archive add-dark-mode -y`
- **THEN** system merges delta specs into main specs
- **AND** system moves change to `openspec/changes/archive/`

### Requirement: CLI Update Command

The system SHALL provide an `openspec update [path]` command that updates AI tool instruction files.

#### Scenario: Update tool configurations
- **WHEN** user runs `openspec update .`
- **THEN** system regenerates skills and commands in tool-specific directories

### Requirement: CLI Config Command

The system SHALL provide an `openspec config` command for managing configuration.

#### Scenario: Show current config
- **WHEN** user runs `openspec config`
- **THEN** system displays current configuration settings

### Requirement: CLI Schemas Command

The system SHALL provide an `openspec schemas` command that lists available workflow schemas.

#### Scenario: List schemas
- **WHEN** user runs `openspec schemas`
- **THEN** system displays available schemas with descriptions

### Requirement: Version and Help

The system SHALL provide `--version` and `--help` flags for all commands.

#### Scenario: Show version
- **WHEN** user runs `openspec --version`
- **THEN** system outputs the version number

#### Scenario: Show help
- **WHEN** user runs `openspec --help`
- **THEN** system displays available commands and options

### Requirement: JSON Output Stream Separation

The system SHALL keep `--json` output parseable by AI agents that combine stdout and stderr.

#### Scenario: No spinner leakage with --json
- **WHEN** a command is run with `--json`
- **THEN** spinner/progress text is suppressed and does not leak into stderr
- **AND** the JSON payload remains the only structured output

### Requirement: CLI Initiative Command

The system SHALL provide an `openspec initiative` command group for managing initiatives that group related changes.

#### Scenario: Initiative command available
- **WHEN** user runs `openspec initiative --help`
- **THEN** system lists initiative subcommands

### Requirement: CLI Workspace Command

The system SHALL provide an `openspec workspace` command group for beta multi-project workspace planning.

#### Scenario: Workspace command available
- **WHEN** user runs `openspec workspace --help`
- **THEN** system lists workspace subcommands (e.g. register, open, view)

#### Scenario: Update does not route into workspace
- **WHEN** user runs top-level `openspec update` in a project that is not a workspace root
- **THEN** system performs a normal project update and does NOT route into workspace updates
- **AND** a foreign root `workspace.yaml` (e.g. from another tool) does not block normal updates