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

### Requirement: Global Directory Resolution Parity

The system SHALL resolve the global config and data directories to the same locations as the upstream TypeScript CLI, so a context store / global config is shared between the two implementations.

#### Scenario: XDG override honored on all platforms
- **WHEN** `XDG_CONFIG_HOME` or `XDG_DATA_HOME` is set
- **THEN** the global config/data dir is `$XDG_*_HOME/openspec` regardless of platform

#### Scenario: macOS uses dotfile paths, not Library
- **WHEN** no XDG override is set on macOS
- **THEN** the global config dir is `~/.config/openspec` and the data dir is `~/.local/share/openspec` (matching upstream, not `~/Library/Application Support`)
- **AND** an existing legacy macOS global config is migrated to the new location on a best-effort basis

### Requirement: Foreign Workspace File Isolation

The system SHALL ignore foreign root `workspace.yaml` files that do not belong to OpenSpec, so unrelated projects continue updating normally.

#### Scenario: Ignore non-OpenSpec workspace.yaml
- **WHEN** a project root contains a `workspace.yaml` written by another tool
- **THEN** `openspec update` treats the project as a normal project and ignores that file

### Requirement: Default Core Profile Sync Workflow

The system SHALL include the `sync` workflow in the default `core` profile so new installations generate `/opsx:sync` skills and commands by default.

#### Scenario: Core profile includes sync
- **WHEN** user initializes a project with the default `core` profile
- **THEN** system generates the `sync` workflow skill and command alongside the other core workflows