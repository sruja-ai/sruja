# Getting Started with Sruja

**5 minutes to architecture intelligence** using one core skill for evidence-first architecture discovery.

## TL;DR

```bash
# 1. Install CLI
curl -fsSL https://sruja.ai/install.sh | bash

# 2. Install the core skill
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture

# 3. Get instant structural intelligence
sruja quickstart -r .

# 4. Generate architecture (in AI editor)
"Use sruja-architecture. Run `sruja discover --context -r . --format json`,
gather evidence, ask targeted questions if needed,
generate architecture.sruja, run `sruja lint` and fix."
```

---

## Core Concept

**The skill is the product**, not the CLI narrative reports.

The workflow:
1. CLI collects deterministic evidence
2. AI skill interprets evidence and generates DSL
3. CLI validates DSL
4. You review and refine

This ensures:
- Evidence-based architecture (no guessing)
- Machine-readable DSL for version control
- Linting and drift detection
- AI assistance for modeling decisions

---

## Install by Editor

| Editor | Install |
|--------|---------|
| **Cursor** | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` |
| **GitHub Copilot** | Install skills.sh, then run the command above |
| **Claude** | Install skills.sh, then run the command above |
| **Continue.dev** | Add `.cursorrules` to `contextFiles` in config |
| **Any (skills.sh)** | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` |

---

## Step 1: Instant Structural Intelligence

Get immediate insights about your architecture (no AI required):

```bash
sruja quickstart -r .
```

**Output includes:**
- 📊 Architecture inventory (modules, services, databases, APIs)
- 💚 Health score with visual indicator
- 🔍 Top findings with severity levels
- 📎 Evidence references from your code
- 🎯 Scan scope (what was analyzed)

**Example:**
```
══════════════════════════════════════════════════════════════════════
🚀 Sruja Quickstart - Structural Analysis
══════════════════════════════════════════════════════════════════════

📂 Scanning repository...
   ✓ Found 753 components

───────────────────────────────────────────────────────────────────────────────
📊 Architecture Inventory
───────────────────────────────────────────────────────────────────────────────
  Components detected:
    • 750 modules
    • 1 services
    • 2 databases
    • 1533 total dependencies

───────────────────────────────────────────────────────────────────────────────
💚 Architecture Health Score: 75/100
───────────────────────────────────────────────────────────────────────────────
```

---

## Step 2: Generate Architecture with AI

### Collect Evidence

The AI skill starts by running the CLI to gather deterministic evidence:

```bash
sruja discover --context -r . --format json
```

**Returns:**
- Repository structure
- Detected technologies
- Module boundaries
- Entry points
- Dependencies
- Scan scope

### Ask Targeted Questions

The AI asks 2-5 questions only when evidence is ambiguous:
- "What are the main system boundaries?"
- "What external services do you integrate with?"
- "How are components deployed?"

### Generate Minimal DSL

The AI produces a minimal `architecture.sruja` based on evidence:

```sruja
import { * } from 'sruja.ai/stdlib'

user = person "User" {
  description "End user"
}

app = system "My App" {
  api = container "API" {
    technology "Node.js"
    description "REST API"
  }

  db = database "Database" {
    technology "PostgreSQL"
    description "Data storage"
  }
}

user -> app.api "HTTPS"
app.api -> app.db "SQL"
```

### Validate

Always lint the generated architecture:

```bash
sruja lint architecture.sruja
```

Fix all errors before considering complete.

---

## Step 3: Use the Architecture

### Export Documentation

```bash
sruja export markdown architecture.sruja > ARCHITECTURE.md
sruja export mermaid architecture.sruja > ARCHITECTURE.mmd
```

### Detect Drift

When code changes, check for drift:

```bash
sruja drift -r . -a architecture.sruja --format json
```

### Add to CI

```yaml
# .github/workflows/architecture.yml
- name: Lint
  run: find . -name '*.sruja' -exec sruja lint {} \;
- name: Drift check
  run: sruja drift -r . -a architecture.sruja --fail-on all
```

---

## Stable CLI Commands

These are the stable CLI commands used by the skill:

| Command | Purpose | Output |
|---------|---------|--------|
| `discover --context` | Collect evidence | `--format json` |
| `lint` | Validate DSL | `--format json` |
| `fmt` | Format DSL | default |
| `export` | Export documentation | default |
| `drift` | Detect drift | `--format json` |
| `intent check` | Check intent | `--format json` |
| `context` | Export AI context | default |
| `quickstart` | Structural analysis | default or `--format json` |

---

## Common Workflows

### Discovery from New Codebase

```
Use sruja-architecture. Run `sruja discover --context -r . --format json`,
gather evidence from the repo, ask targeted questions if scope or externals are unclear,
generate architecture.sruja with C4 structure (systems/containers/components),
then run `sruja lint` and fix until it passes.
```

### Refine Existing Architecture

```
Use sruja-architecture. Analyze this existing architecture.sruja,
run `sruja discover --context -r . --format json` for current evidence,
compare architecture against evidence, identify discrepancies,
propose updates to align with current code,
then run `sruja lint` and fix all errors.
```

### Detect Drift

```
Use sruja-architecture. Run `sruja drift -r . -a architecture.sruja --format json`,
analyze drift results, propose updates to address drift,
run `sruja lint` and fix all errors. List open questions for uncertainties.
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `sruja: command not found` | `curl -fsSL https://sruja.ai/install.sh \| bash` |
| Skill not loading | Check editor supports skills.sh |
| Agent guesses | Add: "Do not guess. List open questions." |
| Lint E204 (circular) | Remove one edge in the cycle |
| Lint E205 (orphan) | Add relationship or remove element |
| Evidence unclear | Use open questions instead of guessing |

---

## Principles

### Evidence-First

- Trust what the CLI actually finds
- Don't invent components or relationships
- Surface uncertainties as open questions
- Validate with linting

### Minimal Modeling

- Start with C4 context + container levels
- Add component level only when needed
- Prefer fewer, correct elements over speculative detail
- Don't model for completeness

### Validation

- Always lint after generating or editing
- Fix all errors before committing
- Use drift detection for maintenance
- Add to CI for ongoing validation

---

## Next Steps

| Want to... | Go to |
|------------|-------|
| Skill reference | [SKILL.md](../skills/sruja-architecture/SKILL.md) |
| Detailed workflow | [REFERENCE.md](../skills/sruja-architecture/REFERENCE.md) |
| Prompt patterns | [PROMPTS.md](../skills/sruja-architecture/PROMPTS.md) |
| DSL reference | [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md) |
| CLI commands | [RUN_GUIDE.md](RUN_GUIDE.md) |
| CI/CD setup | [USING_SRUJA_IN_YOUR_PROJECT.md](USING_SRUJA_IN_YOUR_PROJECT.md) |
| Architecture intelligence | [internal/architecture-lab/](internal/architecture-lab/) |
