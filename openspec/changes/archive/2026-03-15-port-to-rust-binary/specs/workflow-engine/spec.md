## ADDED Requirements

### Requirement: Schema Resolution

The system SHALL resolve schemas in a specific priority order: project-local, user-override, then package built-in.

#### Scenario: Resolve project-local schema
- **WHEN** a schema exists at `openspec/schemas/<name>/schema.yaml`
- **THEN** system uses the project-local schema
- **AND** system ignores user and package schemas of same name

#### Scenario: Resolve user-override schema
- **WHEN** no project-local schema exists
- **AND** a schema exists at `${XDG_DATA_HOME}/openspec/schemas/<name>/schema.yaml`
- **THEN** system uses the user-override schema

#### Scenario: Resolve package built-in schema
- **WHEN** no project-local or user-override schema exists
- **THEN** system uses the embedded package schema

### Requirement: Schema Parsing

The system SHALL parse schema.yaml files into structured schema objects.

#### Scenario: Parse valid schema
- **WHEN** system parses a valid schema.yaml
- **THEN** system extracts `name`, `version`, `description`, and `artifacts`
- **AND** each artifact has `id`, `generates`, `description`, `template`, `instruction`, and `requires`

#### Scenario: Handle invalid schema
- **WHEN** system parses an invalid schema.yaml
- **THEN** system returns a SchemaLoadError with path and cause

### Requirement: Artifact Dependency Graph

The system SHALL compute artifact status based on dependency satisfaction.

#### Scenario: Compute ready status
- **WHEN** an artifact has all dependencies satisfied (files exist)
- **THEN** system marks artifact status as `ready`

#### Scenario: Compute blocked status
- **WHEN** an artifact has unsatisfied dependencies
- **THEN** system marks artifact status as `blocked`
- **AND** system lists `missingDeps`

#### Scenario: Compute done status
- **WHEN** an artifact's output file exists
- **THEN** system marks artifact status as `done`

### Requirement: Apply Requirements Resolution

The system SHALL determine which artifacts must be complete before implementation can begin.

#### Scenario: Get apply requirements
- **WHEN** system loads a schema
- **THEN** system extracts `apply.requires` to determine minimum artifacts for implementation

### Requirement: Template Resolution

The system SHALL resolve artifact templates from the schema directory.

#### Scenario: Get template content
- **WHEN** system needs a template for an artifact
- **THEN** system reads the template file from the schema directory
- **OR** system uses embedded template if no file exists
