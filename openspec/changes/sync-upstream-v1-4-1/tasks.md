## 1. AI tool adapters (Bucket B)

- [ ] 1.1 Add `detection_paths: &'static [&'static str]` field to `AITool` in `src/ai_tools/generator.rs`
- [ ] 1.2 Add new tools in v1.4.1 order: Bob Shell (`.bob`), ForgeCode (`.forge`), Junie (`.junie`), Kimi CLI (`.kimi`), Lingma (`.lingma`), Mistral Vibe (`.vibe`)
- [ ] 1.3 Add GitHub Copilot `detection_paths` list and update `detect_available_tools` to use detection paths when present
- [ ] 1.4 Consolidate duplicated `detect_available_tools` (remove the copy in `src/cli/init.rs`, call generator's)
- [ ] 1.5 Unit tests: new tools listed; Copilot not detected from bare `.github/`; detected when a detection path exists

## 2. Spec parser fixes (Bucket B)

- [ ] 2.1 Match `### Requirement:` / `#### Scenario:` headers case-insensitively in `src/core/spec_parser.rs`
- [ ] 2.2 Detect requirement headers hidden inside fenced code blocks for validation
- [ ] 2.3 Unit tests for case-insensitive headers and code-block-hidden requirements; regress existing `openspec/specs/`

## 3. Validation hint (Bucket B)

- [ ] 3.1 In `src/cli/validate.rs` / `src/core/schema.rs`, detect SHALL/MUST present only in requirement header
- [ ] 3.2 Emit specific hint to move the keyword onto the body line
- [ ] 3.3 Unit test for the header-only hint

## 4. Completion fixes (Bucket B)

- [ ] 4.1 Make completion install opt-in (remove implicit install from init/update) in `src/cli/completion.rs`
- [ ] 4.2 Fix zsh completion under oh-my-zsh `compinit`
- [ ] 4.3 Fix PowerShell completion encoding
- [ ] 4.4 Tests/manual verification per shell

## 5. Telemetry + misc fixes (Bucket B)

- [ ] 5.1 Telemetry client: ~1s timeout, no retries, swallow network errors silently in `src/telemetry/`
- [ ] 5.2 Ensure `--json` suppresses spinner/progress so nothing leaks to stderr
- [ ] 5.3 Tests for opt-out/firewall-silent behavior

## 6. Profile + workflow templates (Bucket B)

- [ ] 6.1 Include `sync` workflow in default `core` profile (`src/templates/skills.rs` + profile logic)
- [ ] 6.2 Port modified workflow template content from `core/templates/workflows/*`
- [ ] 6.3 Ignore foreign root `workspace.yaml` during `update`

## 7. Context store subsystem (Bucket A)

- [ ] 7.1 Create `src/core/context_store/` (registry, metadata, on-disk format via serde_yaml)
- [ ] 7.2 Implement `context-store` CLI group: setup, register, unregister, remove, list, doctor
- [ ] 7.3 Round-trip tests against fixtures; `--json` output parity

## 8. Initiatives subsystem (Bucket A)

- [ ] 8.1 Create `src/core/collections/initiatives/` (create/resolve/list within a context store)
- [ ] 8.2 Implement `initiative` CLI group: create, show, list (+ `--store`/`--store-path`/`--json`)
- [ ] 8.3 Support `--initiative` linking when creating a repo-local change
- [ ] 8.4 Tests for initiative create/show/list and change linking

## 9. Workspace subsystem (Bucket A)

- [ ] 9.1 Create `src/core/workspace/` (registry, link/relink, doctor, view state in `.openspec-workspace/view.yaml`)
- [ ] 9.2 Implement `workspace` CLI group: setup, list/ls, link, relink, doctor, update, open
- [ ] 9.3 Ensure top-level `openspec update` does not route into workspace updates
- [ ] 9.4 Tests for workspace setup/link/list and update isolation

## 10. Finalize sync

- [ ] 10.1 `cargo build --release`, `cargo test`, `cargo clippy` all clean
- [ ] 10.2 Update `docs/STATE.md`: sync status, current version `v1.4.1`, last-synced date
- [ ] 10.3 Update `CHANGELOG.md` with the synced features/fixes
- [ ] 10.4 Bump `Cargo.toml` version as appropriate
- [ ] 10.5 Update `rs-sync-baseline` marker note in STATE.md to point at `v1.4.1`
