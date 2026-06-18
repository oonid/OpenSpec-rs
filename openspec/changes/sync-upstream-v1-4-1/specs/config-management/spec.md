## ADDED Requirements

### Requirement: Default Core Profile Sync Workflow

The system SHALL include the `sync` workflow in the default `core` profile so new installations generate `/opsx:sync` skills and commands by default.

#### Scenario: Core profile includes sync
- **WHEN** user initializes a project with the default `core` profile
- **THEN** system generates the `sync` workflow skill and command alongside the other core workflows

### Requirement: Foreign Workspace File Isolation

The system SHALL ignore foreign root `workspace.yaml` files that do not belong to OpenSpec, so unrelated projects continue updating normally.

#### Scenario: Ignore non-OpenSpec workspace.yaml
- **WHEN** a project root contains a `workspace.yaml` written by another tool
- **THEN** `openspec update` treats the project as a normal project and ignores that file
