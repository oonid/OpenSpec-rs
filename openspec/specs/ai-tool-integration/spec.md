## Purpose

AI tool integration for OpenSpec. Generates skill files and slash commands for AI coding assistants.

## Requirements

### Requirement: AI Tool Detection

The system SHALL support configuration for multiple AI coding tools.

#### Scenario: List supported tools
- **WHEN** system initializes
- **THEN** system supports at minimum: opencode, claude, cursor, windsurf, cline, continue, amazon-q

### Requirement: Skill Generation

The system SHALL generate skill files for supported AI tools.

#### Scenario: Generate opencode skills
- **WHEN** user initializes with `--tools opencode`
- **THEN** system creates `.opencode/skills/` directory
- **AND** system creates `SKILL.md` files for each workflow skill (propose, apply, archive, explore)

#### Scenario: Generate skill with correct format
- **WHEN** system generates a skill file
- **THEN** file includes YAML frontmatter with name, description, metadata
- **AND** file includes skill instructions in markdown

### Requirement: Command Generation

The system SHALL generate slash command files for supported AI tools.

#### Scenario: Generate opencode commands
- **WHEN** user initializes with `--tools opencode`
- **THEN** system creates `.opencode/command/` directory
- **AND** system creates command markdown files (opsx-propose, opsx-apply, opsx-archive, opsx-explore)

### Requirement: Tool-Specific Output Paths

The system SHALL generate files in tool-specific directories.

#### Scenario: Generate for multiple tools
- **WHEN** user initializes with `--tools claude,cursor`
- **THEN** system creates `.claude/` and `.cursor/` directories
- **AND** each contains tool-appropriate configuration files
