# OpenSpec Shell Wrapper

This directory previously contained shell scripts used during development before the Rust port was complete.

## openspec.sh (Archived)

**Purpose:** Docker wrapper to run OpenSpec CLI via npx without installing Node.js locally.

**Original Content:**
```bash
#!/bin/bash
docker run --rm -i \
  -v "$(pwd):/workspace" \
  -w /workspace \
  -u "$(id -u):$(id -g)" \
  node:20-alpine \
  sh -c "npx @fission-ai/openspec@latest $*"
```

**Usage:**
```bash
./scripts/openspec.sh init .
./scripts/openspec.sh list
./scripts/openspec.sh status --change my-change
```

**Requirements:**
- Docker installed and running
- Internet connection (to pull node:20-alpine image and download npm package)

**Status:** This script is no longer needed. Use the native Rust binary instead:
```bash
./target/release/openspec init .
./target/release/openspec list
./target/release/openspec status --change my-change
```

**Why it existed:** During the initial phase of porting OpenSpec to Rust, we needed a way to run the original TypeScript version without installing Node.js on the development machine. This Docker wrapper provided that capability.

**Removed:** The `scripts/` directory has been removed from the project as the Rust binary is now feature-complete.
