## Why

After the v1.2→v1.4.1 sync and parity audit, the Rust port matches upstream OpenSpec on behavior and on-disk formats, but a set of **CLI surface gaps** remain: commands and flags present in the npm CLI (v1.4.1) that the Rust binary is missing. These break muscle memory and scripts for users switching between the two. This change closes the remaining gaps to reach full CLI parity with upstream v1.4.1.

(Upstream v1.4.1 is the latest release; this is not a new version sync — it's the unported remainder identified by the audit.)

## What Changes

**In scope — parity gaps to close:**
- **`config` subcommands**: replace the flat `--set/--get/--list` with upstream's subcommand shape — `config path | list | get <key> | set <key> <value> | unset <key> | reset [--all] | edit | profile [preset]`.
- **`feedback <message>`**: submit anonymous feedback (respects telemetry opt-out / `--body`).
- **`show` spec-filter flags**: `--requirements` (requirements only), `--requirement <id>` (one requirement, 1-based), `--no-scenarios`.
- **`new change` flags**: `--json`, `--goal`, `--affected-areas`, and initiative linking `--initiative`/`--store`/`--store-path`.
- **`validate --concurrency <n>`**: bounded parallel validation.
- **`--no-interactive`**: accepted as a no-op wherever upstream defines it (the Rust port is already non-interactive), so scripts passing it don't error.
- **Fix: package-schema resolution** — stop resolving package schemas from the dev-only `vendor/OpenSpec/schemas` path; installed binaries SHALL resolve the embedded schema (and any project/user schemas), so `schema which`/`templates` report correct paths for end users.

**Out of scope (documented decisions, not gaps to fix here):**
- **`view`** interactive TUI dashboard — a large, different class of work (full-screen terminal UI) with low value for a scriptable CLI port; deferred to its own change if ever wanted.
- Deprecated noun command groups `spec <show|list|validate>` and `change <show|list|validate>`, and the deprecated `experimental` alias — upstream emits deprecation warnings for these; the Rust port exposes the verb-first equivalents (`show`, `validate`, `list`) already.

## Capabilities

### Modified Capabilities
- `cli-commands`: add `feedback`; `show` spec-filter flags; `new change` flags (`--json`/`--goal`/`--affected-areas`/`--initiative`/`--store`/`--store-path`); `validate --concurrency`; accept `--no-interactive` as a no-op; correct package-schema resolution for installed binaries.
- `config-management`: `config` gains the upstream subcommand surface (path/list/get/set/unset/reset/edit/profile) replacing the flat flags.

## Impact

- **Affected code**: `src/cli/args.rs` (clap surface for config/new/show/validate/feedback + global `--no-interactive`), `src/cli/config.rs` (subcommand rewrite), `src/cli/show.rs` (filter flags), `src/cli/new_change.rs` (flags + initiative link, reusing the initiatives resolution), `src/cli/validate.rs` (concurrency), a new `src/cli/feedback.rs`, and `src/core/schema.rs` (package-schema resolution).
- **Compatibility**: `config` changes are the one BREAKING surface change (flat `--set/--get/--list` → subcommands) — it aligns with upstream; document in CHANGELOG. Everything else is additive.
- **Reference**: upstream `vendor/OpenSpec/` at v1.4.1 (`commands/config.ts`, `commands/spec.ts` filters, `cli/index.ts` feedback/new/templates).
