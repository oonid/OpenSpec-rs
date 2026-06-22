## ADDED Requirements

### Requirement: Workspace Setup and Linking

The system SHALL provide a beta `openspec workspace` command group that sets up coordination workspaces and links existing repos or folders to them.

#### Scenario: Set up a workspace
- **WHEN** user runs `openspec workspace setup`
- **THEN** system creates a workspace and records its configuration
- **AND** system can link existing repos or folders during setup

#### Scenario: Link an existing repo
- **WHEN** user runs `openspec workspace link [nameOrPath] [path]`
- **THEN** system links the repo or folder to the workspace under a link name

#### Scenario: Relink a moved repo
- **WHEN** user runs `openspec workspace relink <name> <path>`
- **THEN** system updates the local path for the existing workspace link

### Requirement: Workspace Inspection

The system SHALL allow listing and diagnosing workspaces.

#### Scenario: List workspaces
- **WHEN** user runs `openspec workspace list` (or `ls`)
- **THEN** system lists known OpenSpec workspaces

#### Scenario: Diagnose workspace resolution
- **WHEN** user runs `openspec workspace doctor`
- **THEN** system reports what the workspace can resolve on this machine

### Requirement: Workspace Open and View State

The system SHALL open a workspace in an agent or editor, and SHALL persist beta workspace view state under `.openspec-workspace/view.yaml`.

#### Scenario: Open a workspace
- **WHEN** user runs `openspec workspace open [name]`
- **THEN** system opens the workspace in a selected agent or VS Code editor

#### Scenario: Persist view state
- **WHEN** workspace view state changes
- **THEN** system stores it under `.openspec-workspace/view.yaml`

### Requirement: Workspace Update Isolation

The system SHALL keep workspace updates separate from project updates.

#### Scenario: Refresh workspace guidance
- **WHEN** user runs `openspec workspace update [name]`
- **THEN** system refreshes workspace-local OpenSpec guidance and agent skills

#### Scenario: Top-level update stays project-scoped
- **WHEN** user runs top-level `openspec update` outside a workspace root
- **THEN** system does not route into workspace updates
