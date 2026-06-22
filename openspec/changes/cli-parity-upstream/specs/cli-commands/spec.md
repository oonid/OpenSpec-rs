## MODIFIED Requirements

### Requirement: CLI New Change Command

The system SHALL provide an `openspec new change <name>` command that creates a new change directory, supporting JSON output and optional initiative linking.

#### Scenario: Create new change
- **WHEN** user runs `openspec new change add-dark-mode`
- **THEN** system creates `openspec/changes/add-dark-mode/` directory
- **AND** system creates `.openspec.yaml` with schema configuration

#### Scenario: Create change with schema option
- **WHEN** user runs `openspec new change my-feature --schema spec-driven`
- **THEN** system creates change directory with specified schema

#### Scenario: Create change with JSON output
- **WHEN** user runs `openspec new change my-feature --json`
- **THEN** system outputs the created change as JSON (id, path)

#### Scenario: Create change linked to an initiative
- **WHEN** user runs `openspec new change my-feature --initiative <id>` (optionally `--store <id>` or `--store-path <path>`)
- **THEN** system records the initiative link in the change's `.openspec.yaml` metadata
- **AND** system supports `--goal` and `--affected-areas` metadata fields

### Requirement: CLI Show Command

The system SHALL provide an `openspec show [item-name]` command that displays change or spec details, with spec-filtering flags for JSON output.

#### Scenario: Show change as JSON
- **WHEN** user runs `openspec show add-dark-mode --json`
- **THEN** system outputs JSON with change details including deltas

#### Scenario: Show only requirements of a spec
- **WHEN** user runs `openspec show <spec> --type spec --json --requirements`
- **THEN** system outputs the spec's requirements without scenario content

#### Scenario: Show a single requirement
- **WHEN** user runs `openspec show <spec> --type spec --json --requirement <n>`
- **THEN** system outputs only the nth (1-based) requirement

#### Scenario: Exclude scenarios
- **WHEN** user runs `openspec show <spec> --type spec --json --no-scenarios`
- **THEN** system outputs requirements with scenario content excluded

### Requirement: CLI Validate Command

The system SHALL provide an `openspec validate [item-name]` command that validates changes and specs, emits actionable hints for common authoring mistakes, and supports bounded parallelism.

#### Scenario: Validate all changes
- **WHEN** user runs `openspec validate --all`
- **THEN** system validates all changes and specs
- **AND** system reports validation errors if any

#### Scenario: Hint when SHALL/MUST only in header
- **WHEN** a requirement has SHALL/MUST only in its `### Requirement:` header line and not in the requirement body
- **THEN** system reports a hint to move the keyword onto the requirement body line
- **AND** the hint is more specific than the generic missing-normative-keyword error

#### Scenario: Bounded concurrency
- **WHEN** user runs `openspec validate --all --concurrency <n>`
- **THEN** system validates items with at most `n` concurrent operations

## ADDED Requirements

### Requirement: CLI Feedback Command

The system SHALL provide an `openspec feedback <message>` command to submit anonymous feedback, respecting the telemetry opt-out settings.

#### Scenario: Submit feedback
- **WHEN** user runs `openspec feedback "the new workspace flow is great"`
- **THEN** system submits the message anonymously and reports success

#### Scenario: Feedback respects opt-out
- **WHEN** telemetry is disabled (`OPENSPEC_TELEMETRY=0` or `DO_NOT_TRACK=1`)
- **THEN** `openspec feedback` does not transmit and informs the user it is disabled

### Requirement: Non-Interactive Flag Acceptance

The system SHALL accept the `--no-interactive` flag wherever upstream defines it, treating it as a no-op (the Rust CLI is already non-interactive), so scripts passing it do not error.

#### Scenario: --no-interactive accepted
- **WHEN** user passes `--no-interactive` to a command that upstream documents it on (e.g. `show`, `validate`, `workspace setup`)
- **THEN** system accepts the flag without error and behaves identically

### Requirement: Installed Package Schema Resolution

The system SHALL resolve the built-in (package) schema from the embedded schema for installed binaries, not from a development-only `vendor/OpenSpec/schemas` path.

#### Scenario: Package schema resolves when run outside the dev repo
- **WHEN** an installed binary runs `openspec schema which spec-driven` or `openspec templates` from any directory
- **THEN** system resolves the built-in schema from the embedded definition (source: package)
- **AND** system does not depend on a `vendor/OpenSpec/schemas` directory existing
