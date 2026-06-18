## ADDED Requirements

### Requirement: Context Store Setup and Registration

The system SHALL provide an `openspec context-store` command group to set up and register local context stores that back workspace and initiative data.

#### Scenario: Set up a context store
- **WHEN** user runs `openspec context-store setup [id]`
- **THEN** system creates and registers a local context store
- **AND** system supports `--path <path>` (defaults to OpenSpec-managed local data) and optional `--init-git`

#### Scenario: Register an existing store
- **WHEN** user runs `openspec context-store register [path] --id <id>`
- **THEN** system registers the existing local context store, defaulting the id from metadata or folder name

### Requirement: Context Store Lifecycle

The system SHALL allow listing, diagnosing, and removing context-store registrations.

#### Scenario: List registered stores
- **WHEN** user runs `openspec context-store list`
- **THEN** system lists locally registered context stores (with `--json` support)

#### Scenario: Diagnose a store
- **WHEN** user runs `openspec context-store doctor [id]`
- **THEN** system checks the registration and metadata of the store

#### Scenario: Unregister without deleting
- **WHEN** user runs `openspec context-store unregister <id>`
- **THEN** system forgets the registration without deleting files

#### Scenario: Remove and delete
- **WHEN** user runs `openspec context-store remove <id> --yes`
- **THEN** system forgets the registration and deletes its local folder
