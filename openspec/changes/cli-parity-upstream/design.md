## Context

The Rust port reached behavioral + on-disk parity with upstream v1.4.1 in v0.2.0. The remaining differences are CLI-surface only: missing commands/flags identified by the parity audit. This change closes them. The port is intentionally non-interactive and flag-driven; upstream interactive flows become flags or are no-ops.

## Goals / Non-Goals

**Goals:**
- Full CLI command/flag parity with upstream v1.4.1 for the in-scope items.
- Keep on-disk formats unchanged (config.json already camelCase; `.openspec.yaml` initiative link already defined).

**Non-Goals:**
- The `view` interactive TUI dashboard (separate effort if ever wanted).
- Re-adding deprecated noun command groups (`spec`/`change`) or the `experimental` alias — upstream deprecates them and the verb-first equivalents exist.

## Decisions

### 1. `config` → subcommands (BREAKING)
Replace the flat `Config { set, get, list }` with a `Config(ConfigCommands)` group: `Path | List {json} | Get {key} | Set {key,value} | Unset {key} | Reset {all} | Edit | Profile {preset}`. Rationale: match upstream exactly; the flat form has no upstream analogue and confuses users switching tools. Document the break in CHANGELOG. `edit` shells out to `$EDITOR`/`$VISUAL` (fallback to a sensible default).

### 2. `feedback` reuses the telemetry transport
`feedback <message>` posts via the existing telemetry client (PostHog) under the same opt-out gating (`OPENSPEC_TELEMETRY`/`DO_NOT_TRACK`/CI). No new network stack. If disabled, it no-ops with a clear message. *Alternative:* a dedicated endpoint — rejected (reuse existing, respect existing consent).

### 3. `--no-interactive` as a global no-op clap flag
Add a global `--no-interactive` (like `--no-color`) so any command accepts it without error. Simplest faithful adaptation of upstream's per-command flag. *Alternative:* per-command flags — more surface, no benefit here.

### 4. `new change` flags reuse existing infra
`--initiative`/`--store`/`--store-path` reuse the initiatives resolution + the `set change` linking logic (write `initiative` into `.openspec.yaml`). `--goal`/`--affected-areas` add the optional ChangeMetadata fields upstream defines. `--json` prints `{id, path}`.

### 5. `show` spec filters operate on parsed JSON
`--requirements` / `--requirement <n>` / `--no-scenarios` filter the already-parsed spec model before serializing (mirror upstream `filterSpec`). JSON-only (ignored for change/human output, with a warning like upstream).

### 6. `validate --concurrency <n>` 
Bound parallel validation with a worker pool (default mirrors upstream). Determinism of aggregated output preserved (sort results).

### 7. Package-schema resolution fix
`get_package_schemas_dir` currently returns the hardcoded relative `vendor/OpenSpec/schemas`. Change resolution so the built-in schema comes from the embedded definition (already in the binary) rather than a path that only exists in the dev checkout; keep project/user schema dirs. This makes `schema which`/`templates` correct for installed users.

## Risks / Trade-offs

- **config BREAKING change** → mitigation: clear CHANGELOG note + matching upstream reduces long-term confusion; pre-1.0 so acceptable.
- **`config edit` shelling to an editor** → not unit-testable end-to-end; factor editor-command resolution into a testable helper, gate the actual spawn.
- **Schema-resolution change** → ensure `schemas`/`schema which`/`templates`/`validate` still work in-repo and installed; cover with tests using an isolated dir.

## Migration Plan
Land per command group (config, feedback, show flags, new-change flags, validate concurrency, no-interactive, schema fix), each with the full CI gate (fmt + clippy --all-targets -D warnings + test) and `validate --all` 8/8+. Update CHANGELOG (note the config break) and README command table.

## Open Questions
- `config edit` default editor on each platform when `$EDITOR`/`$VISUAL` unset (vi vs notepad) — match upstream's choice.
- Exact `feedback` payload/event name upstream uses (confirm against `cli/index.ts` feedback action during implementation).
