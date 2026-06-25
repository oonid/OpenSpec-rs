## 1. config subcommands (BREAKING)

- [x] 1.1 Replace flat `Config { set, get, list }` with `Config(ConfigCommands)` in `src/cli/args.rs` (Path/List/Get/Set/Unset/Reset/Edit/Profile)
- [x] 1.2 Implement subcommands in `src/cli/config.rs` (get/set/unset/reset/list/path against the global config; `--json` for list)
- [x] 1.3 `config profile [preset]` shows/sets the active profile
- [x] 1.4 `config edit` opens `$VISUAL`/`$EDITOR` (testable editor-resolution helper; platform fallback)
- [x] 1.5 Tests for get/set/unset/reset round-trips; CHANGELOG note for the breaking change

## 2. feedback command

- [x] 2.1 Add `Feedback { message }` to args + `src/cli/feedback.rs`
- [x] 2.2 Submit via the existing telemetry client; respect opt-out (OPENSPEC_TELEMETRY/DO_NOT_TRACK/CI) → no-op with message when disabled
- [x] 2.3 Confirm payload/event name against upstream `cli/index.ts`; tests for the opt-out path

## 3. show spec-filter flags

- [x] 3.1 Add `--requirements`, `--requirement <n>`, `--no-scenarios` to `show` (args + `src/cli/show.rs`)
- [x] 3.2 Filter the parsed spec model before serializing (mirror upstream `filterSpec`); JSON-only, warn if used with a change
- [x] 3.3 Tests for each filter

## 4. new change flags

- [x] 4.1 Add `--json`, `--goal`, `--affected-areas`, `--initiative`, `--store`, `--store-path` to `new change`
- [x] 4.2 `--json` outputs `{id, path}`; `--goal`/`--affected-areas` write the optional ChangeMetadata fields
- [x] 4.3 `--initiative` links via the initiatives resolution + `.openspec.yaml` (reuse `set change` logic)
- [x] 4.4 Tests for json output + initiative link on create

## 5. validate --concurrency

- [x] 5.1 Add `--concurrency <n>` to `validate`; bound parallel validation with a worker pool
- [x] 5.2 Preserve deterministic aggregated output (sorted); test concurrency bound

## 6. --no-interactive acceptance

- [x] 6.1 Add a global `--no-interactive` clap flag (like `--no-color`), accepted as a no-op everywhere
- [x] 6.2 Test that representative commands accept it without error

## 7. package-schema resolution fix

- [x] 7.1 Change `get_package_schemas_dir`/resolution in `src/core/schema.rs` so the built-in schema resolves from the embedded definition, not `vendor/OpenSpec/schemas`
- [x] 7.2 Verify `schemas`, `schema which`, `templates`, `schema validate` work both in-repo and from an isolated dir; tests

## 8. Finalize

- [x] 8.1 `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `openspec validate --all` all clean
- [x] 8.2 Update CHANGELOG (config BREAKING + new flags/commands) and README command table
- [x] 8.3 Bump version as appropriate (patch — 0.2.0 -> 0.2.1 for continued upstream compatibility verification)
