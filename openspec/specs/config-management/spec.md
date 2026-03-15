## Purpose

Configuration management for OpenSpec. Handles project-level and global configuration with XDG compliance.

## Requirements

### Requirement: Project Configuration

The system SHALL support project-level configuration via `openspec/config.yaml`.

#### Scenario: Read project config
- **WHEN** `openspec/config.yaml` exists
- **THEN** system loads schema name, profile, and other settings

#### Scenario: Create default config
- **WHEN** user initializes a project
- **THEN** system may create `config.yaml` with default settings

### Requirement: Global Configuration

The system SHALL support user-level configuration in XDG data directory.

#### Scenario: Use global data directory
- **WHEN** system needs global config or schemas
- **THEN** system uses `$XDG_DATA_HOME/openspec/` or `~/.local/share/openspec/`

#### Scenario: Store telemetry preference
- **WHEN** user opts out of telemetry
- **THEN** system stores preference in global config

### Requirement: Configuration Validation

The system SHALL validate configuration files against a schema.

#### Scenario: Validate config
- **WHEN** system loads a config file
- **THEN** system validates required fields and types
- **AND** system reports errors for invalid configuration

### Requirement: Environment Variable Override

The system SHALL support environment variable overrides for configuration.

#### Scenario: Override telemetry via env
- **WHEN** `OPENSPEC_TELEMETRY=0` is set
- **THEN** system disables telemetry regardless of config file
