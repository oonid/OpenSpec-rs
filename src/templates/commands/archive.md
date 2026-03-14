Archive a completed change in the experimental workflow. Use when the user wants to finalize and archive a change after implementation is complete.

**Input**: Optional change name to archive.

**Steps**

1. **Determine the change**

   If no change specified:
   ```bash
   openspec list
   ```
   Ask user to select or specify a change.

2. **Verify completion**

   ```bash
   openspec status --change <name>
   ```
   Check that all artifacts are complete.

3. **Validate before archiving**

   ```bash
   openspec validate <name>
   ```
   Ensure specs are valid.

4. **Archive the change**

   ```bash
   openspec archive <name> --yes
   ```
   This will:
   - Merge delta specs into main specs
   - Move change to `archive/` directory with timestamp
   - Update spec files with changes

5. **Confirm success**

   Verify:
   - Change moved to `openspec/changes/archive/YYYY-MM-DD-<name>/`
   - Specs updated in `openspec/specs/`

**Output**: Archived change with updated main specs.

**Note**: Use `--skip-specs` if you don't want to update main specs during archive.
