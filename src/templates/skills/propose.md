Propose a new change with all artifacts generated in one step. Use when the user wants to quickly describe what they want to build and get a complete proposal with design, specs, and tasks ready for implementation.

**Input**: A description of what the user wants to build or change.

**Steps**

1. **Gather requirements**

   If the description is vague, ask clarifying questions:
   - What is the main goal?
   - Who are the users/stakeholders?
   - What are the constraints?
   - Are there existing patterns to follow?

2. **Check for existing changes**

   ```bash
   openspec list --json
   ```
   Ensure we're not duplicating work.

3. **Create the change directory**

   ```bash
   openspec new change <change-name>
   ```

4. **Generate artifacts**

   Create in the change directory:
   - `proposal.md` - Why and what
   - `design.md` - Technical decisions
   - `specs/` - Requirement specifications
   - `tasks.md` - Implementation checklist

5. **Review with user**

   Present a summary and ask for confirmation or adjustments.

**Output Format**

```
## Proposed Change: <change-name>

### Summary
<1-2 sentence summary>

### Artifacts Created
- proposal.md - <brief description>
- design.md - <brief description>
- specs/*.md - <count> spec files
- tasks.md - <count> tasks

### Next Steps
1. Review the generated artifacts
2. Run `/opsx:apply` to start implementation
```

**Guardrails**
- Keep proposals focused and scoped
- Break large changes into smaller ones
- Follow existing project patterns
- Ensure tasks are actionable and testable
