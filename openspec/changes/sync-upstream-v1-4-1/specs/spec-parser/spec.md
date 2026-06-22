## MODIFIED Requirements

### Requirement: Requirement Extraction

The system SHALL extract requirements with their scenarios from markdown. Requirement headers SHALL be matched case-insensitively, and header-looking lines nested inside fenced code blocks SHALL NOT be parsed as requirements or scenarios.

#### Scenario: Extract requirement with scenarios
- **WHEN** system parses `### Requirement: <name>` followed by `#### Scenario: <name>` blocks
- **THEN** system extracts requirement name, description, and all scenarios
- **AND** each scenario has WHEN/THEN conditions

#### Scenario: Handle malformed requirement
- **WHEN** a requirement is missing scenarios
- **THEN** system still extracts the requirement
- **AND** system may warn about missing scenarios

#### Scenario: Case-insensitive requirement header
- **WHEN** a spec uses a header such as `### requirement: <name>` or `### REQUIREMENT: <name>`
- **THEN** system parses it as a requirement regardless of capitalization

#### Scenario: Ignore requirement inside a fenced code block
- **WHEN** a spec contains a `### Requirement:` or `#### Scenario:` line inside a fenced code block (```` ``` ```` or `~~~`)
- **THEN** system does NOT parse the fenced line as a requirement or scenario
- **AND** only real, non-fenced requirements are counted
