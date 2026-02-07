---
title: "CLI Basics"
weight: 10
summary: "Install, verify, and use the Sruja CLI to work with architecture models."
tags: ["cli", "getting-started"]
---

# CLI Basics

This tutorial teaches the essential Sruja CLI commands for day‑to‑day work.

## Install and Verify

```bash
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
sruja --version
```

If `sruja` is not found, ensure the install directory is in your `PATH`:

```bash
# If installed via install script, ensure ~/.local/bin (or script’s target) is in PATH
export PATH="$HOME/.local/bin:$PATH"
```

## Create a Model

```sruja
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
sruja tree --file example.sruja
```

## Export to D2

```bash
sruja export d2 example.sruja > example.d2
```
