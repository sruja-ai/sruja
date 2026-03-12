# Getting started with the Sruja architecture skill

One path from zero to a validated architecture file using the Sruja agent skill.

## 1. Install the skill

In your project (or any folder), run:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent
```

This installs the **sruja-architecture-agent** skill so your AI assistant can discover architecture from your codebase and produce valid Sruja DSL. The skill uses **discovery modes** (overview, standard, deep-dive, diff) and a **phased playbook** for more accurate capture; see [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md).

## 2. Run one prompt

In Cursor (or your IDE) chat, paste this **one prompt** (same as in [INSTALL_AS_SKILL](INSTALL_AS_SKILL.md#recommended-prompt-architecture-discovery---one-prompt-easy)):

*"Use the sruja-architecture-agent skill. Run \`sruja discover --context -r .\`, then generate \`architecture.sruja\` with systems, containers, components, and relationships (evidence-based; no guessing). If you find requirements, ADRs, or key flows in repo docs (README, docs/, adr/, SECURITY.md, etc.), add them to the file with citations; otherwise list 'Open questions' and do not invent. Run \`sruja lint architecture.sruja\` and fix until it passes."*

The agent will discover context, generate `architecture.sruja` (and add requirements/ADRs/flows when it finds evidence in docs), and run `sruja lint` until the file passes.

### Want the richest intent capture (with user confirmation)?

Use the **confirm-first 3-pass workflow** in [INSTALL_AS_SKILL](INSTALL_AS_SKILL.md#recommended-workflow-confirm-first-richest-output). It generates C4 structure first, then drafts an “Intent Review” with citations, then encodes requirements/ADRs/scenarios/flows only after the user confirms.

## 3. Validate (if you edit by hand)

After any change to a `.sruja` file, run:

```bash
sruja lint architecture.sruja
```

If you don’t have the CLI yet: [Install the Sruja CLI](https://sruja.ai) (`curl -fsSL https://sruja.ai/install.sh | bash`) or build from source: `make build` in the Sruja repo.

## 4. Optional: check drift

Compare the generated architecture to the current codebase:

```bash
sruja drift -a architecture.sruja -r .
```

This highlights where the code has diverged from the documented architecture.

## 5. Optional: export to diagram or docs

**Mermaid diagram:** `sruja export mermaid architecture.sruja` (use `--view-level 2` or `3` for container/component views).

**Markdown doc:** `sruja export markdown architecture.sruja` — generates `architecture.md` with a context diagram and sections.

---

## Summary

| Step | Action |
|------|--------|
| 1 | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent` |
| 2 | Paste the [one-prompt](INSTALL_AS_SKILL.md#recommended-prompt-architecture-discovery---one-prompt-easy) in IDE chat |
| 3 | Run `sruja lint architecture.sruja` after edits |
| 4 | (Optional) Run `sruja drift -a architecture.sruja -r .` |
| 5 | (Optional) Run `sruja export mermaid` or `sruja export markdown` for diagram/docs |

For more options (other skills, editors, prompts), see [Install Sruja as a skill](INSTALL_AS_SKILL.md).
