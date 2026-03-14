Implement tasks from an OpenSpec change. Use when the user wants to start implementing, continue implementation, or work through tasks.

**Input**: Optional change name (defaults to active change).

**Steps**

1. **Determine the change**

   If no change specified:
   ```bash
   openspec list
   ```
   Ask user to select or specify a change.

2. **Check current status**

   ```bash
   openspec status --change <name>
   ```
   Identify which artifact to work on.

3. **Get instructions**

   ```bash
   openspec instructions <artifact> --change <name>
   ```
   Or for implementation mode:
   ```bash
   openspec instructions apply --change <name>
   ```

4. **Implement the task**

   - Follow the instructions provided
   - Make code changes as specified
   - Run tests/lint as appropriate

5. **Update task status**

   After completing a task, update `tasks.md`:
   - Change `[ ]` to `[x]` for completed tasks
   - Update STATE.md with progress

6. **Verify and commit**

   - Run validation: `openspec validate --all`
   - Commit changes with descriptive message
   - Check for any blockers before continuing

**Output**: Implemented tasks with tests passing and changes committed.

**Note**: Work on one checklist item at a time. Stop after completing one task and ask for review.
