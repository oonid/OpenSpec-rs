## ADDED Requirements

### Requirement: Config Subcommand Surface

The system SHALL provide an `openspec config` command group matching upstream's subcommand shape, replacing the flat `--set/--get/--list` flags. (**BREAKING**: the flat-flag form is removed in favor of subcommands.)

#### Scenario: Show config file path
- **WHEN** user runs `openspec config path`
- **THEN** system prints the path to the global config file

#### Scenario: List all settings
- **WHEN** user runs `openspec config list`
- **THEN** system prints all configuration keys and values (with `--json` support)

#### Scenario: Get a value
- **WHEN** user runs `openspec config get <key>`
- **THEN** system prints the value for that key (or indicates it is unset)

#### Scenario: Set a value
- **WHEN** user runs `openspec config set <key> <value>`
- **THEN** system persists the value to the global config

#### Scenario: Unset a value
- **WHEN** user runs `openspec config unset <key>`
- **THEN** system removes that key from the global config

#### Scenario: Reset config
- **WHEN** user runs `openspec config reset` (or `reset --all`)
- **THEN** system restores configuration to defaults

#### Scenario: Edit config
- **WHEN** user runs `openspec config edit`
- **THEN** system opens the global config file in the user's editor

#### Scenario: Manage profile
- **WHEN** user runs `openspec config profile [preset]`
- **THEN** system shows or sets the active profile (e.g. `core`, `custom`)
