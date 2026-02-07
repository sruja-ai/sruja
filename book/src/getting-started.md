# Quick start

## Install CLI

```bash
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
```

Or build from source (requires Rust):

```bash
git clone https://github.com/sruja-ai/sruja.git && cd sruja
make build
```

## Create a `.sruja` file

This is the **minimal style** (explicit kinds, no import). The full [Getting started (full)](docs/getting-started.md) uses `import { * } from 'sruja.ai/stdlib'` for less boilerplate — both work.

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"

user = person "User" {}
app = system "My App" {
  web = container "Web Server" { technology "Node.js" }
}
user -> app.web "visits"
```

## Validate and export

```bash
sruja lint example.sruja
sruja export json example.sruja
sruja export markdown example.sruja
```

## VS Code

Install the **Sruja** extension for syntax, diagnostics, and optional diagram preview in the editor.

---

**Next:** [Beginner path](docs/beginner-path.md) builds on this in 7 steps (2–3 hours). For a longer "first architecture" walkthrough with a view and stdlib import, see [Getting started (full)](docs/getting-started.md).
