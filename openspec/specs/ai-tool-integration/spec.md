## Purpose

AI tool integration for OpenSpec. Generates skill files and slash commands for AI coding assistants.
## Requirements
### Requirement: AI Tool Detection

The system SHALL support configuration for multiple AI coding tools, and SHALL auto-detect tools present in a project. Detection MAY use explicit detection paths (files or directories) instead of the tool's default skills directory.

#### Scenario: List supported tools
- **WHEN** system initializes
- **THEN** system supports at minimum: opencode, claude, cursor, windsurf, cline, continue, amazon-q
- **AND** system supports the additional tools: bob, forgecode, junie, kimi, lingma, vibe

#### Scenario: Detect tool by skills directory
- **WHEN** a project contains a tool's skills directory (e.g. `.claude/`)
- **THEN** system marks that tool as detected

#### Scenario: Detect GitHub Copilot via specific paths
- **WHEN** a project contains only a bare `.github/` directory with none of Copilot's detection paths (e.g. `.github/copilot-instructions.md`, `.github/prompts`, `.github/agents`)
- **THEN** system does NOT falsely detect GitHub Copilot
- **AND** system detects GitHub Copilot only when at least one configured detection path exists

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