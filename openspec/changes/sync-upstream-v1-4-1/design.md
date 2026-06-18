## Context

`OpenSpec-rs` is a simplified, skills-first Rust port of the upstream TypeScript OpenSpec CLI. The vendored reference (`vendor/OpenSpec/`) has been advanced from `v1.2.0-5-gafdca0d` (`rs-sync-baseline`) to `v1.4.1`. The delta is 62 commits / 114 files. Diff artifacts are saved under `docs/sync-v1.2-to-v1.4.1/`.

The port intentionally diverges from upstream structure: AI tools are a flat `AI_TOOLS` table in `src/ai_tools/generator.rs` (each tool just needs a `skills_dir`), not the per-tool command-generation adapter classes upstream uses. Bucket B changes map cleanly onto existing Rust modules. Bucket A (workspace, initiatives, context-store) has **no existing Rust equivalent** and introduces new modules and new on-disk state.

## Goals / Non-Goals

**Goals:**
- Full behavioral parity for Bucket B fixes (tools, parser, validation, completion, telemetry, profile).
- Functional parity for the Bucket A command surfaces (workspace/initiative/context-store) and their on-disk formats, so a `.openspec-workspace/` or context store created by either implementation is interoperable.
- Preserve existing single-project behavior unchanged.

**Non-Goals:**
- Line-by-line translation of the TS subsystems; reimplement behavior idiomatically in Rust.
- Replicating upstream's per-tool command-generation adapter architecture (the Rust port stays skills-first).
- Interactive TUI polish for workspace `open` beyond launching the selected agent/editor.

## Decisions

### 1. AI tool detection: add `detection_paths`
Extend `AITool` with `detection_paths: &'static [&'static str]`. When non-empty, detection checks each path (file or dir) via existence rather than treating `skills_dir` as a directory. GitHub Copilot gets the upstream path list; this fixes false positives from a bare `.github/`. Consolidate the duplicated `detect_available_tools` logic (currently in both `generator.rs` and `init.rs`) onto the `generator.rs` implementation.
*Alternative:* a per-tool detection closure — rejected as overkill for a static table.

### 2. Spec parser: case-insensitive headers + code-block-hidden requirements
Match `### Requirement:`/`#### Scenario:` headers case-insensitively. For validation, detect requirement headers that appear inside fenced code blocks in main specs so they aren't silently dropped. Implement in `src/core/spec_parser.rs` with targeted unit tests.

### 3. Validation hint for header-only SHALL/MUST
In the validator path, when a requirement's normative keyword appears only in the header line and not the body, surface a specific hint instead of the generic error. Implement in `src/cli/validate.rs` / `src/core/schema.rs`.

### 4. New subsystems: new Rust modules + serde_yaml state
- `src/core/context_store/` — registry of local context stores; registry persisted under the OpenSpec global data dir (reuse existing XDG/global-config resolution). Commands: setup, register, unregister, remove, list, doctor.
- `src/core/collections/initiatives/` — initiatives live inside a context store. Commands: create, show, list.
- `src/core/workspace/` — workspace registry, link/relink, doctor, update, open; view state in `.openspec-workspace/view.yaml`.
- CLI: new `workspace`, `initiative`, `context-store` subcommand groups in `src/cli/` wired via clap, mirroring upstream subcommand names/flags.
Use `serde` + `serde_yaml` (already in tree) for all on-disk formats; match upstream YAML field names exactly for interoperability.
*Alternative:* deferring Bucket A — rejected; the user chose a single change covering B + A.

### 5. Completion install becomes opt-in
Remove any implicit completion install from `init`/`update`; install only on explicit `completion install <shell>`. Apply the oh-my-zsh `compinit` fix and PowerShell encoding fix in `src/cli/completion.rs`.

### 6. Telemetry resilience
Ensure the telemetry client uses a short (~1s) timeout, no retries, and swallows network errors silently. Implement in `src/telemetry/`.

## Risks / Trade-offs

- **Bucket A scope is large and partly beta** → port the command surfaces and on-disk formats first; defer cosmetic/interactive polish. Mark workspace as beta in help text.
- **On-disk format drift between Rust and TS** → mitigation: replicate upstream YAML schemas exactly and add round-trip tests against fixtures copied from `vendor/OpenSpec/`.
- **Duplicated detection logic** → consolidating `detect_available_tools` could change `init` ordering/behavior; covered by integration tests.
- **Parser changes affecting existing specs** → guard with unit tests over current `openspec/specs/` so no regression in existing parsing.

## Migration Plan

Implement Bucket B first (low risk, fast parity), then Bucket A subsystem-by-subsystem (context-store → initiatives → workspace, since the latter depend on the former). Run `cargo build/test/clippy` after each area. Finalize by updating `docs/STATE.md` sync marker to `v1.4.1`, CHANGELOG, and the `rs-sync-baseline` note.

## Open Questions

- Should the context-store registry path exactly match upstream's location, or use the Rust port's existing global-config dir? (Default: match upstream for interoperability; confirm during context-store implementation.)
- How much of workspace `open` (agent/editor selection prompts) is in-scope vs. deferred for the beta surface?
