# initiatives Specification

## Purpose
TBD - created by archiving change sync-upstream-v1-4-1. Update Purpose after archive.
## Requirements
### Requirement: Initiative Change Linking

The system SHALL allow linking a repo-local change to an initiative.

#### Scenario: Link a change to an initiative
- **WHEN** a change is created with `--initiative <id>` (optionally `--store <id>` or `--store-path <path>`)
- **THEN** system records the link between the repo-local change and the initiative

### Requirement: Initiative Management

The system SHALL provide an `openspec initiative` command group to create and list coordinated initiatives that live inside a context store and group related changes.

#### Scenario: Create an initiative
- **WHEN** user runs `openspec initiative create [id] --title <title> --summary <summary>`
- **THEN** system creates an initiative in the resolved context store
- **AND** system supports selecting the store via `--store <id>` or `--store-path <path>`

#### Scenario: Show an initiative
- **WHEN** user runs `openspec initiative show <id>`
- **THEN** system reports where the initiative lives and how to read it

#### Scenario: List initiatives
- **WHEN** user runs `openspec initiative list`
- **THEN** system lists initiatives across registered context stores
- **AND** system supports `--json` output