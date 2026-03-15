# Quick Start

**Get architecture from code in 5 minutes.**

You don't write `.sruja` files. Your AI does it for you. Here's how:

---

## Step 1: Install the AI skill (1 minute)

This teaches your AI editor how to generate Sruja files. The skill will guide you to install the CLI when needed.

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

**Supported editors:** Cursor, GitHub Copilot, Claude, Continue.dev

---

## Step 2: Generate your architecture file (2 minutes)

Open your AI editor and ask it to generate your architecture:

```
Use sruja-architecture skill. Analyze my codebase, generate a repo.sruja file, 
then run sruja lint and fix any errors until it passes.
```

**What happens:**

1. The skill runs `sruja discover` to understand your code structure (installs CLI if needed)
2. Asks you 2-3 questions if anything is unclear (e.g., "What's this service for?")
3. Generates a `repo.sruja` file with your architecture
4. Runs `sruja lint` to check for errors
5. Fixes any errors automatically

**Result:** You now have a `repo.sruja` file in your project root!

---

## Step 3: Validate it (optional)

The skill already validated your file, but you can check manually:

```bash
sruja lint repo.sruja
```

If it says "✅ All checks passed", you're good!

If you see errors, just ask your AI: "Fix these errors."

---

## What's Next?

You have architecture in code. Now what?

**Generate diagrams for documentation:**

```bash
sruja export mermaid repo.sruja > architecture.mmd
sruja export markdown repo.sruja > ARCHITECTURE.md
```

**Keep it in sync:**

When you change your code, run:

```bash
sruja drift -r .
```

This shows you what changed and if your architecture needs updating.

**Learn more:**

- [Beginner path](docs/beginner-path.md) – 7 steps to go deeper
- [System Design 101](../courses/system-design-101/course-overview.md) – Learn patterns
- [Examples](docs/examples.md) – Real-world architectures

---

## Quick Reference

| What you want | Command |
|---------------|----------|
| **Install skill** | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` |
| **Generate with AI** | Ask your AI: "Use sruja-architecture skill to generate my architecture" |
| **Validate** | `sruja lint repo.sruja` |
| **Export diagram** | `sruja export mermaid repo.sruja > diagram.mmd` |
| **Check for drift** | `sruja drift -r .` |

---

## Common Questions

**"What if the command isn't found?"**

The CLI isn't on your PATH. Try:

```bash
# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Or restart your terminal
```

**"My editor doesn't support skills."**

You can still use Sruja manually:
- Run `sruja discover` to get JSON output
- Create `repo.sruja` by hand (see [Language spec](reference/language-spec.md))
- Use `sruja lint` to validate

But AI makes it much easier—consider using Cursor or installing skills.sh.

**"What's the difference between `quickstart` and `discover`?"**

- **`quickstart`** – Quick overview, human-readable output (great for first look)
- **`discover`** – Detailed JSON output (used by AI for generation)

**Option A – install script (downloads from [GitHub Releases](https://github.com/sruja-ai/sruja/releases)):**

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

**Option B – from Git (requires Rust):**

```bash
cargo install sruja-cli --git https://github.com/sruja-ai/sruja
```

**Option C – build from source:**

```bash
git clone https://github.com/sruja-ai/sruja.git && cd sruja
make build
```

Ensure the `sruja` binary is on your `PATH` (install script uses `~/.local/bin` by default).

## Create a `.sruja` file

This is the **minimal style** (explicit kinds, no import). For the full Getting Started guide using stdlib imports, see [Getting Started](docs/getting-started.md). Both styles are valid; use whichever you prefer.

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User" {}
App = system "My App" {
  Web = container "Web Server" { technology "Node.js" }
}
User -> App.Web "visits"
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
