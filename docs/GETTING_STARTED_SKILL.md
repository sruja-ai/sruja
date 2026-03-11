# Getting started with the Sruja architecture skill

One path from zero to a validated architecture file using the Sruja agent skill.

## 1. Install the skill

In your project (or any folder), run:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent
```

This installs the **sruja-architecture-agent** skill so your AI assistant can discover architecture from your codebase and produce valid Sruja DSL.

## 2. Run one prompt

In Cursor (or your IDE) chat, paste this **recommended prompt**:

*"Analyze this repo and generate a Sruja architecture file (architecture.sruja). Be thorough: main systems, containers, technologies, descriptions for every element, and relationships with clear labels. Run sruja lint and fix until it passes. Use the sruja-architecture-agent skill."*

The agent will explore the repo, generate `architecture.sruja`, and run `sruja lint` until the file passes validation.

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

## 5. Optional: export to Mermaid

Generate a Mermaid diagram from your `.sruja` file (for docs or comparison):

```bash
sruja export mermaid architecture.sruja
```

Use `--view-level 2` or `3` for container/component-level views; see `sruja export --help`.

---

## Summary

| Step | Action |
|------|--------|
| 1 | `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent` |
| 2 | Paste the [recommended prompt](INSTALL_AS_SKILL.md#recommended-prompt-architecture-discovery) in IDE chat |
| 3 | Run `sruja lint architecture.sruja` after edits |
| 4 | (Optional) Run `sruja drift -a architecture.sruja -r .` |
| 5 | (Optional) Run `sruja export mermaid architecture.sruja` for a diagram |

For more options (other skills, editors, prompts), see [Install Sruja as a skill](INSTALL_AS_SKILL.md).
