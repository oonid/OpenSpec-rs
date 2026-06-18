## Why

The Rust port (`OpenSpec-rs`) is synced to upstream OpenSpec `v1.2.0-5-gafdca0d`. Upstream has since released through `v1.4.1` — 62 commits adding 6 new AI tools, several parser/validation/completion/telemetry fixes, and three new subsystems (workspace, initiatives, context-store). Porting these keeps the Rust binary at feature parity with the TypeScript CLI, which is the project's reason for existing (dogfood OpenSpec to port OpenSpec).

## What Changes

**Stable fixes & additions (Bucket B):**
- Add 6 AI tools: Bob Shell (`.bob`), ForgeCode (`.forge`), Junie (`.junie`), Kimi CLI (`.kimi`), Lingma (`.lingma`), Mistral Vibe (`.vibe`).
- Add `detectionPaths` to AI-tool auto-detection; use specific paths for GitHub Copilot to stop false positives from a bare `.github/` directory.
- Spec parser: parse requirement headers case-insensitively; detect requirements hidden inside fenced code blocks during validation.
- Validation: when SHALL/MUST appears only in a requirement header, emit a hint to move the keyword onto the requirement body line.
- Shell completion: make install opt-in; fix zsh completion under oh-my-zsh `compinit`; fix PowerShell encoding corruption.
- Telemetry: swallow PostHog network errors with a 1s timeout (retries/remote-config disabled) so firewalled environments stay silent.
- `--json` output no longer leaks spinner progress text to stderr.
- Include the `sync` workflow in the default `core` profile so new installs generate `/opsx:sync` by default.

**New subsystems (Bucket A) — beta/feature-complete upstream:**
- **BREAKING (new surface)**: `openspec workspace` — beta multi-project workspace planning, openers, and view state stored under `.openspec-workspace/view.yaml`. Top-level `openspec update` must not route into workspace updates; foreign root `workspace.yaml` files are ignored.
- `openspec initiative` — initiatives/collections: a planning primitive that groups related changes.
- Context store — persistent context layer backing workspace/initiative state.

## Capabilities

### New Capabilities
- `workspace`: Beta multi-project workspace — registration, open/view surfaces, workspace-scoped state and skills.
- `initiatives`: Initiatives collection — create/resolve/link initiatives that group changes.
- `context-store`: Persistent context store — foundation, registry, binding, and operations for workspace/initiative data.

### Modified Capabilities
- `ai-tool-integration`: New supported tools; detection supports explicit detection paths (Copilot fix).
- `spec-parser`: Case-insensitive requirement headers; detect requirements hidden in fenced code blocks.
- `cli-commands`: Validate hint for header-only SHALL/MUST; `--json` stderr cleanliness; new `workspace` and `initiative` commands; `update` no longer routes into workspace updates.
- `shell-completion`: Opt-in install; oh-my-zsh `compinit` fix; PowerShell encoding fix.
- `telemetry`: Silent failure in firewalled networks (1s timeout, no retries).
- `config-management`: `sync` workflow added to default `core` profile; foreign root `workspace.yaml` ignored.

## Impact

- **Affected Rust code**: `src/ai_tools/generator.rs`, `src/core/spec_parser.rs`, `src/cli/validate.rs`, `src/cli/completion.rs`, `src/telemetry/`, `src/templates/skills.rs` (profile), `src/cli/` (new `workspace`/`initiative` subcommands), and new modules under `src/core/` for workspace/initiatives/context-store.
- **Reference**: `vendor/OpenSpec/` now at `v1.4.1`; diff artifacts in `docs/sync-v1.2-to-v1.4.1/`.
- **Compatibility**: Bucket B preserves existing CLI behavior. Bucket A adds new commands and a new on-disk `.openspec-workspace/` directory; existing single-project workflows are unaffected.
- **Docs**: `docs/STATE.md` sync marker and CHANGELOG to be updated at finalize.
