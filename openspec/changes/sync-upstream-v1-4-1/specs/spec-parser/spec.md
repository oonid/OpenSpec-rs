## MODIFIED Requirements

### Requirement: Requirement Extraction

The system SHALL extract requirements with their scenarios from markdown. Requirement headers SHALL be matched case-insensitively, and requirements nested inside fenced code blocks SHALL still be detected for validation purposes.

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

#### Scenario: Detect requirement hidden in a code block
- **WHEN** a main spec contains a `### Requirement:` line inside a fenced code block
- **THEN** system still detects that requirement during validation so it is not silently ignored
