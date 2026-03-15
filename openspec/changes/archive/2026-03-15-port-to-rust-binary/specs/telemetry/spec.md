## ADDED Requirements

### Requirement: Anonymous Usage Tracking

The system MAY collect anonymous usage statistics when telemetry is enabled.

#### Scenario: Track command execution
- **WHEN** user runs any openspec command
- **THEN** system MAY send command name and version to analytics
- **AND** system MUST NOT send arguments, paths, or PII

### Requirement: Telemetry Opt-Out

The system SHALL respect user telemetry preferences.

#### Scenario: Disable via environment variable
- **WHEN** `OPENSPEC_TELEMETRY=0` or `DO_NOT_TRACK=1` is set
- **THEN** system disables all telemetry

#### Scenario: Disable in CI environment
- **WHEN** `CI=true` environment variable is set
- **THEN** system automatically disables telemetry

### Requirement: First-Run Notice

The system SHALL display telemetry notice on first run.

#### Scenario: Show telemetry notice
- **WHEN** user runs openspec for the first time
- **THEN** system displays notice about anonymous usage stats
- **AND** system shows how to opt out

#### Scenario: Suppress notice after first run
- **WHEN** user has run openspec before
- **THEN** system does not display telemetry notice again

### Requirement: Async Telemetry

The system SHALL send telemetry asynchronously to avoid blocking CLI operations.

#### Scenario: Non-blocking telemetry
- **WHEN** system sends telemetry
- **THEN** command execution is not delayed
- **AND** telemetry failures do not affect command results
