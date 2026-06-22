## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: JSON Output Stream Separation

The system SHALL keep `--json` output parseable by AI agents that combine stdout and stderr.

#### Scenario: No spinner leakage with --json
- **WHEN** a command is run with `--json`
- **THEN** spinner/progress text is suppressed and does not leak into stderr
- **AND** the JSON payload remains the only structured output

### Requirement: CLI Workspace Command

The system SHALL provide an `openspec workspace` command group for beta multi-project workspace planning.

#### Scenario: Workspace command available
- **WHEN** user runs `openspec workspace --help`
- **THEN** system lists workspace subcommands (e.g. register, open, view)

#### Scenario: Update does not route into workspace
- **WHEN** user runs top-level `openspec update` in a project that is not a workspace root
- **THEN** system performs a normal project update and does NOT route into workspace updates
- **AND** a foreign root `workspace.yaml` (e.g. from another tool) does not block normal updates

### Requirement: CLI Initiative Command

The system SHALL provide an `openspec initiative` command group for managing initiatives that group related changes.

#### Scenario: Initiative command available
- **WHEN** user runs `openspec initiative --help`
- **THEN** system lists initiative subcommands
