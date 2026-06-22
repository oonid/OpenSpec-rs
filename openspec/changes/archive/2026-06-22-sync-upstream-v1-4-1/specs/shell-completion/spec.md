## MODIFIED Requirements

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
