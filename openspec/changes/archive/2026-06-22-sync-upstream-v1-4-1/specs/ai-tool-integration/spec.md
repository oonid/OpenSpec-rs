## MODIFIED Requirements

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
