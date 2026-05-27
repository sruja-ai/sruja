---
title: "CLI Basics"
weight: 10
summary: "Install, verify, and use the Sruja CLI to work with architecture models."
tags: ["cli", "getting-started"]
---

# CLI Basics

This tutorial teaches the essential Sruja CLI commands for day‑to‑day work.

## Install and Verify

**Option A – install script (downloads from [GitHub Releases](https://github.com/sruja-ai/sruja/releases)):**

```bash
curl -fsSL https://sruja.ai/install.sh | bash
sruja --version
```

**Option B – from Git (requires Rust):**

```bash
cargo install sruja-cli --git https://github.com/sruja-ai/sruja
sruja --version
```

**Option C – build from source:**

```bash
git clone https://github.com/sruja-ai/sruja.git && cd sruja && just build  # or: make build
# Add target/release to PATH or copy target/release/sruja to a directory on PATH
sruja --version
```

If `sruja` is not found, ensure the install directory is on your `PATH` (install script uses `~/.local/bin` by default; Option B uses `~/.cargo/bin`; Option C uses `target/release`).

## Create a Model

```sruja
// partial
import { * } from 'sruja.ai/stdlib'


App = system "My App" {
Web = container "Web Server"
DB = database "Database"
}
User = person "User"

User -> App.Web "Visits"
App.Web -> App.DB "Reads/Writes"

view index {
include *
}
```

## Lint and Compile

```bash
sruja lint example.sruja
sruja compile example.sruja
```

## Format

```bash
sruja fmt example.sruja > example.formatted.sruja
```

## Tree View

```bash
sruja tree example.sruja
```

## Export to Mermaid

```bash
sruja export mermaid example.sruja > example.mmd
```
