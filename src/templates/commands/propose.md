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
   - `specs/` - Delta specs for each capability
   - `design.md` - Technical approach
   - `tasks.md` - Implementation checklist

5. **Review with user**

   Present the generated artifacts and ask for feedback before proceeding.

**Output**: A complete change proposal ready for review and implementation.
