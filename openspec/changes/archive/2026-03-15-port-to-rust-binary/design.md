## Context

OpenSpec is a TypeScript/Node.js CLI tool for spec-driven development with AI assistants. The current implementation:
- Requires Node.js 20.19.0+
- Uses npm/pnpm for distribution
- Has ~30 TypeScript source files in `src/`
- Dependencies: commander (CLI), inquirer (prompts), chalk (colors), fast-glob, yaml, zod (validation), ora (spinners), posthog-node (telemetry)

The port will create a functionally equivalent Rust binary while maintaining CLI compatibility.

## Goals / Non-Goals

**Goals:**
- Single static binary per platform (no runtime dependencies)
- Full CLI command parity with v1.2.0
- Compatible with existing `openspec/` directory structure
- Fast startup and execution
- Cross-platform: Linux (x64, ARM), macOS (x64, ARM), Windows (x64)

**Non-Goals:**
- Rewriting TypeScript implementation line-by-line
- Adding new features during port
- Supporting Node.js-specific integrations
- GUI or TUI dashboard (view command can be deferred)

## Decisions

### 1. CLI Framework: `clap` with derive macros

**Rationale:** De facto standard for Rust CLIs. Derive macros reduce boilerplate while maintaining type safety.

**Alternatives considered:**
- `argh`: Simpler but less featureful
- `lexopt`: Lower-level, more manual work

### 2. Async Runtime: `tokio`

**Rationale:** Required for telemetry (HTTP client). Most ecosystem support.

**Alternatives considered:**
- `async-std`: Less ecosystem support
- No async: Would require blocking HTTP, acceptable but limits future extensibility

### 3. Markdown Parsing: `pulldown-cmark`

**Rationale:** Pull-based parser, zero-copy where possible, widely used.

**Alternatives considered:**
- `markdown-rs`: Simpler but less flexible
- Custom parser: Unnecessary complexity

### 4. YAML Parsing: `serde_yaml`

**Rationale:** Serde-compatible, handles schema.yaml files.

### 5. File System Operations: `std::fs` + `walkdir` + `glob`

**Rationale:** Standard library sufficient for most operations. `walkdir` for recursive traversal, `glob` for pattern matching.

### 6. Terminal Output: `termcolor` + `indicatif`

**Rationale:** `termcolor` for colored output (chalk equivalent), `indicatif` for spinners/progress (ora equivalent).

### 7. Interactive Prompts: `inquire`

**Rationale:** Similar API to inquirer.rs, supports select, multi-select, text input.

### 8. Project Structure

```
openspec-rs/
├── src/
│   ├── main.rs           # Entry point, CLI setup
│   ├── cli/              # Command definitions (clap)
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── update.rs
│   │   ├── list.rs
│   │   ├── show.rs
│   │   ├── validate.rs
│   │   ├── archive.rs
│   │   ├── status.rs
│   │   ├── instructions.rs
│   │   ├── new.rs
│   │   └── config.rs
│   ├── core/             # Business logic
│   │   ├── mod.rs
│   │   ├── artifact_graph/  # Schema resolution, dependencies
│   │   ├── schema/       # Schema parsing and validation
│   │   ├── parser/       # Markdown spec parsing
│   │   ├── archive.rs    # Spec merging logic
│   │   └── validation.rs
│   ├── templates/        # Embedded templates
│   ├── ai_tools/         # AI tool config generation
│   ├── telemetry/        # Optional usage tracking
│   └── utils/            # Helpers
├── Cargo.toml
└── build.rs              # Embed templates at compile time
```

### 9. Template Embedding

Use `include_str!` macro to embed templates at compile time. No runtime file lookups needed.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Behavioral differences from TS version | Comprehensive test suite against vendor/OpenSpec/test fixtures |
| Missing edge cases in markdown parser | Test against real spec files from OpenSpec repo |
| Interactive prompt UX differs | Use inquire which has similar feel to inquirer |
| Larger binary size than expected | Use `strip`, LTO, opt-level=z; target <10MB |
| Shell completion differs | Port completion logic carefully, test on all shells |
| Telemetry incompatibility | Same PostHog events, same opt-out mechanism |

## Migration Plan

1. **Phase 1 - Core Commands** (init, new change, status, instructions, list)
2. **Phase 2 - Validation & Show** (validate, show)
3. **Phase 3 - Archive** (archive with spec merging)
4. **Phase 4 - Config & Update** (config, update)
5. **Phase 5 - Polish** (shell completion, view command, telemetry)

**Rollback:** Keep `scripts/openspec.sh` pointing to npm version as fallback.

## Open Questions

- [ ] Should telemetry be included in v1.0 or deferred?
- [ ] View command (TUI dashboard) - include or defer?
- [ ] Binary naming: `openspec` or `openspec-rs`?
