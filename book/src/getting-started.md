# Quick Start

**Get architecture from code in 5 minutes.**

You don't write `.sruja` files. Your AI does it for you. Here's how:

---

## Step 1: Install the CLI (2 minutes)

The CLI is what analyzes your code and validates architecture files.

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

**What this does:** Downloads the latest Sruja binary and adds it to your system path.

**Check it worked:**

```bash
sruja --version
```

You should see something like: `sruja version v0.2.0`

---

## Step 2: Install the AI skill (1 minute)

This teaches your AI editor how to generate Sruja files.

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

**Supported editors:** Cursor, GitHub Copilot, Claude, Continue.dev

---

## Step 3: See what's in your codebase (1 minute)

Run this in your project folder:

```bash
cd your-project
sruja quickstart -r .
```

**What you'll see:**

```
📊 Architecture Inventory
  • 5 services detected
  • 2 databases
  • 12 modules

💚 Architecture Health Score: 75/100

🔍 Top Findings
  • Missing: Payment module not connected
  • Orphan: Analytics service has no consumers
```

**No AI needed yet**—this is just code analysis.

---

## Step 4: Generate your architecture file (1 minute)

Open your AI editor and paste this prompt:

```
Use sruja-architecture. Run `sruja discover --context -r . --format json`,
gather evidence from my code, ask targeted questions if needed,
generate repo.sruja, then run `sruja lint` and fix until it passes.
```

**What your AI will do:**

1. Run `sruja discover` to understand your code structure
2. Ask you 2-3 questions if anything is unclear (e.g., "What's this service for?")
3. Generate a `repo.sruja` file with your architecture
4. Run `sruja lint` to check for errors
5. Fix any errors automatically

**Result:** You now have a `repo.sruja` file in your project root!

---

## Step 5: Validate it (optional but recommended)

```bash
sruja lint repo.sruja
```

If it says "✅ All checks passed", you're good!

If you see errors, just paste the output to your AI: "Fix these errors."

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
| **Install** | `curl -fsSL https://sruja.ai/install.sh \| bash` |
| **Install skill** | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` |
| **Analyze code** | `sruja quickstart -r .` |
| **Generate with AI** | See Step 4 above |
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
