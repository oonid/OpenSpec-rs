# Installation Guide

This guide covers all ways to install OpenSpec-rs on your system.

## Requirements

- **Operating System:** Linux, macOS, or Windows
- **Architecture:** x86_64 or ARM64
- **From source:** Rust 1.75 or later

## Option 1: Download Pre-built Binary (Recommended)

Download the latest release for your platform from the [Releases](https://github.com/oonid/OpenSpec-rs/releases) page.

### Linux

```bash
# x86_64
curl -sSL https://github.com/oonid/OpenSpec-rs/releases/latest/download/openspec-linux-x86_64 -o openspec
chmod +x openspec
sudo mv openspec /usr/local/bin/

# ARM64
curl -sSL https://github.com/oonid/OpenSpec-rs/releases/latest/download/openspec-linux-arm64 -o openspec
chmod +x openspec
sudo mv openspec /usr/local/bin/
```

### macOS

```bash
# x86_64 (Intel)
curl -sSL https://github.com/oonid/OpenSpec-rs/releases/latest/download/openspec-macos-x86_64 -o openspec
chmod +x openspec
sudo mv openspec /usr/local/bin/

# ARM64 (Apple Silicon)
curl -sSL https://github.com/oonid/OpenSpec-rs/releases/latest/download/openspec-macos-arm64 -o openspec
chmod +x openspec
sudo mv openspec /usr/local/bin/
```

### Windows

Download `openspec-windows-x86_64.exe` from the [Releases](https://github.com/oonid/OpenSpec-rs/releases) page.

```powershell
# Using PowerShell (run as Administrator)
Invoke-WebRequest -Uri "https://github.com/oonid/OpenSpec-rs/releases/latest/download/openspec-windows-x86_64.exe" -OutFile "openspec.exe"
Move-Item openspec.exe "C:\Program Files\openspec.exe"
```

Or add the downloaded binary to a directory in your PATH.

## Option 2: Install with Cargo

If you have Rust installed, you can install directly from the repository:

```bash
cargo install --git https://github.com/oonid/OpenSpec-rs
```

Or from a local clone:

```bash
git clone https://github.com/oonid/OpenSpec-rs.git
cd OpenSpec-rs
cargo install --path .
```

## Option 3: Build from Source

### Prerequisites

1. Install Rust 1.75 or later:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Verify installation:
   ```bash
   rustc --version
   cargo --version
   ```

### Build Steps

```bash
# Clone the repository
git clone https://github.com/oonid/OpenSpec-rs.git
cd OpenSpec-rs

# Initialize submodule (optional, for reference only)
git submodule update --init --recursive

# Build release binary
cargo build --release

# Binary will be at target/release/openspec
```

### Install Locally

```bash
# Install to ~/.cargo/bin/
cargo install --path .

# Or copy manually
sudo cp target/release/openspec /usr/local/bin/
```

## Verify Installation

```bash
openspec --version
# Output: openspec 0.1.3

openspec --help
```

## Shell Completions

Generate shell completions after installation:

### Bash

```bash
openspec completion generate bash > /etc/bash_completion.d/openspec
# Or for user-only:
openspec completion generate bash > ~/.local/share/bash-completion/completions/openspec
```

### Zsh

```bash
openspec completion generate zsh > "${fpath[1]}/_openspec"
# Or:
openspec completion generate zsh > ~/.zfunc/_openspec
```

### Fish

```bash
openspec completion generate fish > ~/.config/fish/completions/openspec.fish
```

## Next Steps

- [README.md](../README.md) - Project overview and quick start
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Understanding the codebase

## Troubleshooting

### "command not found: openspec"

Ensure the binary is in your PATH:
```bash
echo $PATH
which openspec
```

### Permission denied

Make the binary executable:
```bash
chmod +x /path/to/openspec
```

### Old Rust version

Update Rust:
```bash
rustup update
```
