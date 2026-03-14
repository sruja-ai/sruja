# Getting Started with Sruja

**5 minutes to architecture intelligence** using one core skill for design and discovery.

## TL;DR

```bash
# 1. Install CLI
curl -fsSL https://sruja.ai/install.sh | bash

# 2. Install the core skill
npx skills add sruja-ai/sruja --skill sruja-architecture

# 3. Get instant intelligence
sruja quickstart -r .

# 4. Generate architecture (in AI editor)
"Use sruja-architecture. Run `sruja discover --context -r .`,
gather evidence, ask targeted questions if needed,
generate architecture.sruja, run `sruja lint` and fix."
```

---

## Install by Editor

| Editor | Install |
|--------|---------|
| **Cursor** | `npx skills add sruja-ai/sruja --skill sruja-architecture` |
| **GitHub Copilot** | Copy [.copilot-instructions.md](../.copilot-instructions.md) to repo root |
| **Continue.dev** | Add `.cursorrules` to `contextFiles` in config |
| **Any (skills.sh)** | `npx skills add sruja-ai/sruja --skill sruja-architecture` |

---

## Available Skills

| Skill | Purpose |
|-------|---------|
| `sruja-architecture` | **Primary** - Design, discover, and generate Sruja architecture |

```bash
# Install the skill
npx skills add sruja-ai/sruja --skill sruja-architecture
```

---

## Workflow

### Step 1: Instant Intelligence

```bash
sruja quickstart -r .
```

Output: architecture inventory, health score, top findings.

### Step 2: Generate Architecture (in AI editor)

```
Use sruja-architecture. Run `sruja discover --context -r .`,
generate architecture.sruja with C4 structure (systems/containers/components),
ask targeted questions if scope or externals are unclear,
run `sruja lint` and fix until it passes.
```

### Step 3: Validate

```bash
sruja lint architecture.sruja
```

## CLI Commands

| Command | Purpose |
|---------|---------|
| `sruja quickstart -r .` | Instant architecture inventory |
| `sruja drift -r .` | Detect structural issues |
| `sruja why "question" -r .` | Evidence-based answers |
| `sruja analyze -r .` | Full analysis |
| `sruja lint file.sruja` | Validate DSL |

---

## CI/CD Integration

```yaml
# .github/workflows/architecture.yml
- name: Install Sruja
  run: cargo install sruja-cli --git https://github.com/sruja-ai/sruja --locked
- name: Lint
  run: find . -name '*.sruja' -exec sruja lint {} \;
- name: Drift check
  run: sruja drift -r . -a architecture.sruja
```

See [USING_SRUJA_IN_YOUR_PROJECT.md](USING_SRUJA_IN_YOUR_PROJECT.md).

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `sruja: command not found` | `curl -fsSL https://sruja.ai/install.sh \| bash` |
| Skill not loading | Check editor supports skills.sh |
| Agent guesses | Add: "Do not guess. List open questions." |
| Lint E204 (circular) | Remove one edge in the cycle |
| Lint E205 (orphan) | Add relationship or remove element |

---

## Next Steps

| Want to... | Go to |
|------------|-------|
| CLI deep dive | [RUN_GUIDE.md](RUN_GUIDE.md) |
| Use in your project | [USING_SRUJA_IN_YOUR_PROJECT.md](USING_SRUJA_IN_YOUR_PROJECT.md) |
| DSL reference | [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md) |
| Architecture intelligence | [internal/architecture-lab/INTELLIGENCE_ANALYSIS.md](internal/architecture-lab/INTELLIGENCE_ANALYSIS.md) |
