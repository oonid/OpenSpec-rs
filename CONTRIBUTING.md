# Contributing to OpenSpec-rs

Thank you for your interest in contributing to OpenSpec-rs! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Testing](#testing)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Syncing with Upstream](#syncing-with-upstream)

## Code of Conduct

Be respectful and constructive. We welcome contributions from everyone.

## How to Contribute

### Reporting Issues

1. Check if the issue already exists in [GitHub Issues](https://github.com/oonid/OpenSpec-rs/issues)
2. If not, create a new issue with:
   - Clear title and description
   - Steps to reproduce (for bugs)
   - Expected vs actual behavior
   - Your environment (OS, Rust version)

### Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests and lints
5. Submit a pull request

## Development Setup

### Prerequisites

- **Rust 1.75+** (MSRV - Minimum Supported Rust Version)
- **Git** (for version control)
- **Just** (optional, for running commands - `cargo install just`)

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/OpenSpec-rs.git
cd OpenSpec-rs

# Initialize submodule (optional, for upstream reference)
git submodule update --init --recursive

# Build
cargo build

# Run tests
cargo test

# Run the binary
cargo run -- <command>
```

### Useful Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_schema_parsing

# Run with verbose output
cargo test -- --nocapture

# Check code without building
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Build release binary
cargo build --release
```

## Project Structure

```
OpenSpec-rs/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli/                 # CLI command implementations
│   │   ├── mod.rs
│   │   ├── init.rs          # init command
│   │   ├── new.rs           # new change command
│   │   ├── status.rs        # status command
│   │   ├── list.rs          # list command
│   │   ├── show.rs          # show command
│   │   ├── validate.rs      # validate command
│   │   ├── archive.rs       # archive command
│   │   ├── config.rs        # config command
│   │   ├── completion.rs    # shell completion
│   │   └── update.rs        # update command
│   ├── core/                # Core functionality
│   │   ├── mod.rs
│   │   ├── schema.rs        # Schema parsing and validation
│   │   ├── artifact.rs      # Artifact graph and status
│   │   ├── parser.rs        # Markdown spec parser
│   │   ├── config.rs        # Configuration management
│   │   └── change.rs        # Change discovery
│   ├── templates/           # Embedded templates
│   │   ├── mod.rs
│   │   ├── skills.rs        # AI skill templates
│   │   └── schema.rs        # Embedded schema YAML
│   ├── ai_tools/            # AI tool generators
│   │   ├── mod.rs
│   │   └── config.rs        # Tool configurations
│   ├── telemetry/           # Optional telemetry
│   │   └── mod.rs
│   └── utils/               # Utilities
│       ├── mod.rs
│       ├── output.rs        # Colored output
│       ├── error.rs         # Error types
│       └── spinner.rs       # Progress indicators
├── templates/
│   └── skills/              # Skill template markdown files
├── tests/
│   └── integration_tests.rs # Integration tests
├── vendor/
│   └── OpenSpec/            # Upstream TypeScript source (submodule)
├── openspec/                # OpenSpec project files
├── .opencode/               # OpenCode AI configuration
├── docs/                    # Documentation
├── Cargo.toml
└── README.md
```

## Code Style

### Rust Conventions

- Follow standard Rust conventions (`cargo fmt`)
- Use `clippy` for linting: `cargo clippy -- -D warnings`
- Document public APIs with `///` doc comments

### General Guidelines

- **Keep functions small** - Each function should do one thing well
- **Use meaningful names** - Prefer `change_directory` over `dir`
- **Handle errors properly** - Use `Result<T, E>` and `thiserror`
- **No panics in library code** - Return `Result` instead
- **Minimize dependencies** - Only add what's necessary

### Adding a New CLI Command

1. Create a new file in `src/cli/` (e.g., `src/cli/mycommand.rs`)
2. Define the command struct with `clap` derive macros
3. Implement the `run` method
4. Add the module to `src/cli/mod.rs`
5. Add the variant to `src/cli/Args` enum
6. Add tests in `tests/integration_tests.rs`

Example:

```rust
// src/cli/mycommand.rs
use crate::utils::error::Result;

#[derive(Debug, clap::Args)]
pub struct MyCommandArgs {
    #[arg(short, long)]
    pub option: Option<String>,
}

pub fn run(args: MyCommandArgs, ctx: &crate::core::config::Context) -> Result<()> {
    // Implementation
    Ok(())
}
```

## Testing

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# Specific test
cargo test test_init_creates_openspec_directory
```

### Writing Tests

- Place unit tests in the same file using `#[cfg(test)] mod tests { ... }`
- Place integration tests in `tests/integration_tests.rs`
- Use `tempfile` for tests that create files/directories
- Test both success and error cases

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_yaml() {
        let input = "name: test\nversion: \"1.0\"";
        let result = parse_schema(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let input = "invalid: [unclosed";
        let result = parse_schema(input);
        assert!(result.is_err());
    }
}
```

## Commit Messages

Follow conventional commit format:

```
<type>: <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding/updating tests
- `chore`: Maintenance tasks

### Examples

```
feat: add --json flag to status command

fix: handle missing .openspec.yaml gracefully

docs: update installation instructions

test: add integration tests for archive command
```

## Pull Request Process

1. **Create a feature branch** from `main`
2. **Make focused changes** - One feature/fix per PR
3. **Write/update tests** for your changes
4. **Update documentation** if needed
5. **Run the test suite**: `cargo test && cargo clippy`
6. **Submit the PR** with a clear description

### PR Checklist

- [ ] Code compiles (`cargo build`)
- [ ] Tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated (for significant changes)

## Syncing with Upstream

The `vendor/OpenSpec/` submodule contains the upstream TypeScript implementation. When syncing:

1. Update the submodule:
   ```bash
   cd vendor/OpenSpec
   git fetch origin
   git checkout <new-tag>
   cd ../..
   git add vendor/OpenSpec
   ```

2. Compare and port changes:
   ```bash
   diff -r vendor/OpenSpec/src/commands src/cli/
   ```

3. Update `docs/STATE.md` with the new sync version

4. Add tests for new/changed functionality

## Questions?

Open an issue for questions or discussion about contributions.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
