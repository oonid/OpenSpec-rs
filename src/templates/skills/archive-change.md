Archive a completed change in the experimental workflow. Use when the user wants to finalize and archive a change after implementation is complete.

**Input**: Optionally specify a change name. If omitted, check for the current active change.

**Steps**

1. **Verify change is complete**

   ```bash
   openspec status --change "<name>" --json
   ```
   
   Ensure all artifacts are marked as done before proceeding.

2. **Confirm archive action**

   Ask the user to confirm they want to archive the change. Show:
   - Change name and summary
   - Spec files that will be merged
   - Location of archived change

3. **Execute archive**

   ```bash
   openspec archive "<name>" --yes
   ```

4. **Report results**

   Show:
   - Which specs were updated
   - Where the change was archived
   - Any issues encountered

**Output Format**

```
## Archive Complete: <change-name>

### Merged Specs
- specs/file1.md - <changes made>
- specs/file2.md - <changes made>

### Archived Location
openspec/changes/archive/<timestamp>-<change-name>/

### Next Steps
- Review the merged spec changes
- Start a new change with /opsx:propose or /opsx:new
```

**Guardrails**
- Always verify change is complete before archiving
- Warn if there are uncommitted git changes
- Preserve the full change history in archive
- Handle spec merge conflicts gracefully
