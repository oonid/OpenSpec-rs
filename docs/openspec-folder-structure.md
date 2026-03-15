# OpenSpec Folder Structure

This document describes the `openspec/` folder structure used by OpenSpec for spec-driven development.

> **Note:** This documents the current state during active development. The structure evolves as changes are proposed, implemented, and archived.

## Overview

The `openspec/` directory is the heart of OpenSpec's spec-driven workflow. It contains:
- **Active changes** - Work in progress
- **Archived changes** - Completed work (moved to `changes/archive/`)
- **Main specs** - Consolidated requirements from archived changes

## Directory Structure

```
openspec/
├── changes/                    # Active and archived changes
│   ├── archive/               # Completed changes (dated)
│   │   └── YYYY-MM-DD-<name>/ # Archived with timestamp
│   └── <change-name>/         # Active change directory
│       ├── .openspec.yaml     # Change metadata (schema, created date)
│       ├── proposal.md        # Why we're doing this, scope, goals
│       ├── design.md          # Technical approach and decisions
│       ├── tasks.md           # Implementation checklist
│       └── specs/             # Delta specs (requirements for this change)
│           └── <capability>/
│               └── spec.md    # Requirements and scenarios
└── specs/                      # Main specs (consolidated from archives)
    └── <capability>/
        └── spec.md            # Canonical requirements
```

## Current State (port-to-rust-binary)

```
openspec/
├── changes/
│   └── port-to-rust-binary/           # Active change
│       ├── .openspec.yaml             # schema: spec-driven
│       ├── proposal.md                # Port OpenSpec to Rust
│       ├── design.md                  # Tech decisions
│       ├── tasks.md                   # 84 tasks (76 complete)
│       └── specs/                     # 7 capability specs
│           ├── ai-tool-integration/
│           │   └── spec.md
│           ├── cli-commands/
│           │   └── spec.md
│           ├── config-management/
│           │   └── spec.md
│           ├── shell-completion/
│           │   └── spec.md
│           ├── spec-parser/
│           │   └── spec.md
│           ├── telemetry/
│           │   └── spec.md
│           └── workflow-engine/
│               └── spec.md
└── specs/                              # Empty (no archived changes yet)
```

## Key Files

### `.openspec.yaml`

Change metadata file:

```yaml
schema: spec-driven    # Workflow schema being used
created: 2026-03-14    # Creation date
```

### `proposal.md`

Explains:
- **Why** we're making this change
- **What** will be different
- **Scope** and boundaries
- Success criteria

### `design.md`

Documents:
- Technical approach
- Key decisions and rationale
- Architecture choices
- Dependencies and tools

### `tasks.md`

Implementation checklist:
```markdown
## 1. Project Setup

- [x] 1.1 Initialize Cargo project
- [x] 1.2 Add dependencies
- [ ] 2.1 Next task...
```

### `specs/<capability>/spec.md`

Delta specs with requirements:
```markdown
## ADDED Requirements

### Requirement: Feature Name
The system SHALL do something.

#### Scenario: Basic case
- **WHEN** condition
- **THEN** expected result
```

## Lifecycle

1. **Create** - `openspec new change <name>` creates the directory
2. **Propose** - `/opsx:propose` or manually create proposal, design, specs, tasks
3. **Implement** - `/opsx:apply` works through tasks
4. **Archive** - `/opsx:archive` moves to `changes/archive/YYYY-MM-DD-<name>/` and merges specs

## Git Tracking

The `openspec/` folder is **tracked in git** (not gitignored). This follows the upstream OpenSpec practice.

**Why track it?**
- **Collaboration**: Team members can see active changes
- **History**: All proposals, designs, and specs are versioned
- **Transparency**: Work-in-progress is visible to all contributors

**Current `.gitignore`:**
```
/target
/vendor
/.opencode
/STATE.md
/scripts
/.idea
```

Note: `openspec` is NOT in `.gitignore`, so all changes and specs are tracked.

**Tracking workflow:**
```bash
# Initialize OpenSpec
openspec init .
git add .openspec.yaml openspec/config.yaml

# Create a change (automatically tracked)
openspec new change add-dark-mode

# Work on the change (proposal, specs, etc.)
# Files are automatically tracked

# After archiving
openspec archive add-dark-mode
git add openspec/changes/archive/ openspec/specs/
git commit -m "Archive add-dark-mode change"
```

- [OpenSpec Workflow](./openspec-workflow.md) - How /opsx: commands work
- [Architecture](./ARCHITECTURE.md) - Rust project structure
