## Purpose

Markdown spec parser for OpenSpec. Parses delta specs with ADDED/MODIFIED/REMOVED/RENAMED sections and merges them into main specs.

## Requirements

### Requirement: Delta Spec Parsing

The system SHALL parse markdown spec files with ADDED, MODIFIED, REMOVED, and RENAMED sections.

#### Scenario: Parse ADDED requirements
- **WHEN** system parses a spec file with `## ADDED Requirements` section
- **THEN** system extracts all requirements under that section
- **AND** each requirement has a name and description

#### Scenario: Parse MODIFIED requirements
- **WHEN** system parses a spec file with `## MODIFIED Requirements` section
- **THEN** system extracts requirements with their updated content

#### Scenario: Parse REMOVED requirements
- **WHEN** system parses a spec file with `## REMOVED Requirements` section
- **THEN** system extracts requirements with Reason and Migration fields

#### Scenario: Parse RENAMED requirements
- **WHEN** system parses a spec file with `## RENAMED Requirements` section
- **THEN** system extracts FROM and TO mappings

### Requirement: Requirement Extraction

The system SHALL extract requirements with their scenarios from markdown.

#### Scenario: Extract requirement with scenarios
- **WHEN** system parses `### Requirement: <name>` followed by `#### Scenario: <name>` blocks
- **THEN** system extracts requirement name, description, and all scenarios
- **AND** each scenario has WHEN/THEN conditions

#### Scenario: Handle malformed requirement
- **WHEN** a requirement is missing scenarios
- **THEN** system still extracts the requirement
- **AND** system may warn about missing scenarios

### Requirement: Spec Merging

The system SHALL merge delta specs into main specs during archive.

#### Scenario: Merge ADDED requirements
- **WHEN** archiving a change with ADDED requirements
- **THEN** system appends new requirements to the main spec file

#### Scenario: Merge MODIFIED requirements
- **WHEN** archiving a change with MODIFIED requirements
- **THEN** system replaces the existing requirement in the main spec

#### Scenario: Merge REMOVED requirements
- **WHEN** archiving a change with REMOVED requirements
- **THEN** system removes the requirement from the main spec

### Requirement: Spec File Discovery

The system SHALL discover spec files matching glob patterns.

#### Scenario: Discover specs in change directory
- **WHEN** system looks for specs in `openspec/changes/<name>/specs/`
- **THEN** system finds all `spec.md` files in subdirectories

#### Scenario: Discover main specs
- **WHEN** system looks for specs in `openspec/specs/`
- **THEN** system finds all `spec.md` files organized by capability
