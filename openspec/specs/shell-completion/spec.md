## Purpose

Shell completion support for OpenSpec. Generates and installs completion scripts for Bash, Zsh, and Fish shells.
## Requirements
### Requirement: Shell Completion Generation

The system SHALL generate shell completion scripts for Bash, Zsh, and Fish.

#### Scenario: Generate bash completion
- **WHEN** user runs `openspec completion generate bash`
- **THEN** system outputs bash completion script to stdout

#### Scenario: Generate zsh completion
- **WHEN** user runs `openspec completion generate zsh`
- **THEN** system outputs zsh completion script to stdout

#### Scenario: Generate fish completion
- **WHEN** user runs `openspec completion generate fish`
- **THEN** system outputs fish completion script to stdout

### Requirement: Shell Completion Installation

The system SHALL install completion scripts to appropriate shell directories. Installation SHALL be opt-in (explicitly requested), and SHALL write scripts using encodings/setup compatible with each target shell.

#### Scenario: Install bash completion
- **WHEN** user runs `openspec completion install bash`
- **THEN** system installs completion to appropriate bash completion directory

#### Scenario: Install zsh completion
- **WHEN** user runs `openspec completion install zsh`
- **THEN** system installs completion to zsh fpath directory
- **AND** the installed script works under oh-my-zsh's `compinit`

#### Scenario: Completion install is opt-in
- **WHEN** user runs `openspec init` or `openspec update`
- **THEN** system does NOT install shell completion automatically
- **AND** completion is installed only when the user runs `openspec completion install <shell>`

#### Scenario: PowerShell completion encoding
- **WHEN** user installs PowerShell completion
- **THEN** the generated profile content is written without encoding corruption

### Requirement: Shell Completion Uninstallation

The system SHALL remove installed completion scripts.

#### Scenario: Uninstall completion
- **WHEN** user runs `openspec completion uninstall bash -y`
- **THEN** system removes the installed completion script

### Requirement: Dynamic Completion Data

The system SHALL provide completion data for changes and specs.

#### Scenario: Complete change names
- **WHEN** shell requests completion for change names
- **THEN** system outputs list of available changes