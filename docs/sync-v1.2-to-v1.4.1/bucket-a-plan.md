# Bucket A port plan — workspace / initiatives / context-store

Planning artifact for the `sync-upstream-v1-4-1` change (Opus planning phase).
Dependency order: **context-store → initiatives → workspace** (initiatives live inside a
store; workspace consumes both + view state).

Upstream sizes (TS, v1.4.1): context-store ≈ 2,091 LOC, initiatives ≈ 1,600 LOC,
workspace ≈ 3,000+ LOC. This is a multi-sub-cycle effort; each subsystem is its own
plan → snapshot → Haiku-implement → Opus-review cycle.

Division of labor: **Opus defines the interop contract** (exact serde structs, file paths,
id validation, atomic-write semantics) so on-disk files are byte-compatible with the TS CLI;
**Haiku does the mechanical translation** against that contract; **Opus reviews**.

---

## Sub-cycle 1 — context-store (foundation)

### On-disk layout (MUST match upstream exactly for interop)
- Global registry: `<global-data-dir>/context-stores/registry.yaml`
  - Rust global data dir = `crate::core::config::xdg_data_dir()` → `<XDG_DATA_HOME|data_local_dir>/openspec`. Verify this equals upstream `getGlobalDataDir()` during implementation.
- Default store root: `<global-data-dir>/context-stores/<id>`
- Per-store metadata: `<storeRoot>/.openspec-store/store.yaml`

Constants: `.openspec-store` (dir), `store.yaml`, `context-stores` (dir), `registry.yaml`.

### registry.yaml schema
```yaml
version: 1
stores:
  <id>:
    backend:
      type: git
      local_path: <abs path>
      remote: <optional>
      branch: <optional>
```

### store.yaml schema
```yaml
version: 1
id: <id>
```

### Rust serde structs (interop contract)
```rust
#[derive(Serialize, Deserialize)]
struct RegistryState {            // registry.yaml
    version: u8,                  // always 1
    #[serde(default)]
    stores: BTreeMap<String, RegistryEntryState>,
}
#[derive(Serialize, Deserialize)]
struct RegistryEntryState { backend: BackendConfig }
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum BackendConfig {              // only `git` today
    Git {
        local_path: String,
        #[serde(skip_serializing_if = "Option::is_none")] remote: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")] branch: Option<String>,
    },
}
#[derive(Serialize, Deserialize)]
struct MetadataState { version: u8, id: String }   // store.yaml
```
Note field names are snake_case in YAML (`local_path`) — match exactly. Confirm whether
upstream tags backend with `type: git` literally (it does: `type: 'git'`).

### ID validation
kebab-case `^[a-z0-9]+(?:-[a-z0-9]+)*$`; reject empty, `.`, `..`, and any `/` or `\`.
Error code `invalid_context_store_id`.

### Atomic writes
Write to a temp file then rename over the target (upstream `writeFileAtomically`). Ensure the
parent dir exists first.

### CLI: `openspec context-store <sub>` (mirror upstream flags)
- `setup [id] --path <path> --init-git/--no-init-git --json` → create + register a local store
- `register [path] --id <id> --json` → register an existing store (id defaults from metadata/folder name)
- `unregister <id> --json` → forget registration, keep files
- `remove <id> --yes --json` → forget + delete local folder (requires `--yes`)
- `list --json` → list registered stores
- `doctor [id] --json` → check registration + metadata health

### Module layout (Rust)
`src/core/context_store/{mod.rs, foundation.rs, registry.rs, operations.rs}` and
`src/cli/context_store.rs` wired into the clap command tree (`src/cli/args.rs`/`commands.rs`).
Skip upstream `binding.ts` initially unless initiatives needs it (it binds changes↔stores;
revisit in sub-cycle 2).

### Key upstream operations to port (operations.ts)
`prepareContextStoreSetup`, `setupContextStore`, `registerExistingContextStore`,
`unregisterContextStore`, `removeContextStore`, `listContextStores`, `doctorContextStores`,
`normalizeContextStorePathForComparison` (path comparison for conflict detection).
Conflict detection: `assertNoRegisteredStoreConflict` (registry.ts).

---

## Sub-cycle 2 — initiatives (TODO: detail when reached)
`src/core/collections/initiatives/*` + `commands/initiative.ts`. Initiatives live inside a
resolved context store. Commands: create, show, list (+ `--store`/`--store-path`/`--json`).
Plus `--initiative <id>` linking when creating a repo-local change.

## Sub-cycle 3 — workspace (TODO: detail when reached)
`commands/workspace/*` + `core/workspace/*`. Registry, link/relink, doctor, update, open;
view state in `.openspec-workspace/view.yaml`. Ensure top-level `openspec update` does NOT
route into workspace updates; ignore foreign root `workspace.yaml`.

---

## Blocker before Haiku execution
The reviewed **Bucket B** changes are uncommitted (held for user review). Per the cycle
("snapshot as branch to avoid deletion by Haiku"), Bucket B must be committed/snapshotted
before dispatching Haiku on Bucket A — otherwise Haiku's edits could clobber unsnapshotted
Bucket B work.
