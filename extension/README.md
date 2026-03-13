# Sruja Language Support

VS Code extension for the [Sruja](https://github.com/sruja-ai/sruja) architecture DSL. **Lint and Markdown export run in-process using bundled WebAssembly** (no CLI required). You can optionally use the Sruja CLI by setting `sruja.lsp.path`.

## Features

- **Diagnostics** – Lint runs in the extension via bundled WASM (or the Sruja CLI if `sruja.lsp.path` is set). When using the CLI, the extension runs `sruja lint --format json` for reliable, machine-readable diagnostics (same codes as the skill’s lint→fix table). Errors and warnings appear in the editor (Problems panel, underlines). Supports unsaved buffers. Debounced on type; pending lint is cancelled when the document is closed.
- **Syntax highlighting** – TextMate grammar for keywords, relations, strings, comments
- **Language configuration** – Comment toggling (`//`), bracket matching and autoclosing, **word pattern** for double‑click selection, **indentation rules** for `{`/`}`, **folding** with `// #region` / `// #endregion` markers
- **Snippets** – Kind declarations (person, system, container, database), elements, relations (`->`), views, description blocks
- **Generate Markdown from DSL** – Run **Sruja: Export to Markdown** (or right-click in a .sruja file): generates Markdown via bundled WASM (or CLI), opens it in the editor, and optionally saves to a `.md` file next to the source.
- **Diagram preview** – Run **Sruja: Open Diagram Preview** (or the preview icon in the editor title when a .sruja file is active): renders the architecture as a Mermaid diagram in a side panel using bundled WASM.
- **AI features (skills & rules)** – Browse skills, open SKILL.md / AGENTS.md, list and open rules, copy rule or agent guide to clipboard for use with AI assistants (e.g. Cursor, Copilot). **Multi‑root workspaces**: skills are collected from every folder that has a `skills` subfolder.
- **Workspace support** – Extension runs in the workspace (remote/SSH); supports untrusted workspaces.

## Architecture intelligence (CLI)

**Drift, why, and analyze** run via the Sruja CLI and are surfaced in the extension:

- **Sruja: Run drift (architecture health)** – Runs `sruja drift -r .` in the workspace; output appears in the **Sruja** output channel. Detects structural drift (cycles, orphans, layer violations).
- **Sruja: Analyze repository** – Runs `sruja analyze -r .`; full structural/semantic/intent report in the **Sruja** output channel, including a **health score** and **architecture completion score** with a breakdown of structural/operational/security coverage.
- **Sruja: Why (explain dependencies)** – Prompts for a component or question, then runs `sruja why "<query>" -r .`; explanation appears in the **Sruja** output channel.

These commands require the **Sruja CLI** (on PATH or `sruja.lsp.path`). In CI or headless use, run the CLI directly:

- `sruja quickstart -r .` – Architecture inventory and top findings (no .sruja required).
- `sruja drift -r .` – Detect structural drift.
- `sruja why "<question>" -r .` – Explain dependencies with evidence.
- `sruja analyze -r .` – Full report (health + completion score + recommendations).

## Requirements

- **None** for lint and export when using the published extension (bundled WASM). Optionally set **`sruja.lsp.path`** to use the Sruja CLI instead (e.g. for a specific binary or when developing the extension without running `copy-assets`).

## Commands

| Command | Description |
|--------|-------------|
| **Sruja: Run validation (check after AI/edit)** | Run `sruja lint` on the active .sruja file now (unsaved content is linted via temp file). Use after every AI code iteration to ensure the file is valid and per standards. |
| **Sruja: Export to Markdown** | Export the active .sruja file to Markdown (opens in editor; optional save to `.md`) |
| **Sruja: Open Diagram Preview** | Render the active .sruja file as a Mermaid diagram in a webview (uses bundled WASM) |
| **Sruja: Open Skills Overview** | Open SKILL.md for a skill (quick-pick if multiple) |
| **Sruja: Open Agent Guide (AGENTS.md)** | Open AGENTS.md for a skill |
| **Sruja: List Rules…** | Quick-pick list of all rules; open selected rule |
| **Sruja: Copy Rule for AI** | Copy a rule’s markdown to clipboard for pasting into an AI chat |
| **Sruja: Copy Agent Guide for AI** | Copy AGENTS.md content to clipboard |
| **Sruja: Run drift (architecture health)** | Run `sruja drift -r .`; output in Sruja channel |
| **Sruja: Analyze repository** | Run `sruja analyze -r .`; report in Sruja channel |
| **Sruja: Why (explain dependencies)** | Run `sruja why "<query>" -r .`; answer in Sruja channel |

## Views

- **Sruja Skills** (Explorer sidebar) – Tree of skills → SKILL.md, AGENTS.md, and rules. Click a file to open it. Skills are resolved from `sruja.skills.path`, workspace `skills` folder, or the extension’s bundled `skills` (if present).

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `sruja.lsp.path` | (empty) | Path to the Sruja CLI binary. If set, lint and export use the CLI instead of bundled WASM. |
| `sruja.skills.path` | (empty) | Path to skills folder for AI rules. Empty = use workspace `skills` or extension `skills`. |

## Publishing to Open VSX

The extension is published to the [Open VSX Registry](https://open-vsx.org) via GitHub Actions when a release is published (see [.github/workflows/README.md](../.github/workflows/README.md)). Uses org secret **`OPEN_VSX_TOKEN`** (Personal Access Token from open-vsx.org). On release, the extension version is taken from the release tag (e.g. `sruja-v0.7.7` or `v0.2.0` → `0.7.7` / `0.2.0`).

**Cursor users:** Cursor’s marketplace (`marketplace.cursorapi.com`) mirrors Open VSX and can lag behind. If you don’t see the latest version in Cursor, install from [Open VSX](https://open-vsx.org/extension/srujaai/sruja) or use **Install from VSIX** with the `.vsix` download from that page.

## Development

```bash
cd extension
npm install
npm run compile
```

Run from VS Code (F5). Ensure the Sruja CLI is on PATH or set `sruja.lsp.path`. For the Sruja Skills view, open this repo (or any workspace with a `skills` folder) or set `sruja.skills.path`.

## How to check the extension is working

1. **Build the Sruja CLI** (required for lint and export):
   ```bash
   cargo build --release -p sruja-cli
   ```
   For lint and export: run `npm run copy:assets` from the extension directory (copies WASM into `extension/wasm/`), or set `sruja.lsp.path` to your Sruja CLI binary.

2. **Launch the extension**  
   Open this repo in VS Code, press **F5** (or Run → Start Debugging). A new VS Code window opens with the extension loaded (“Extension Development Host” in the title).

3. **In the Extension Development Host window:**

   | Check | How |
   |-------|-----|
   | **Syntax highlighting** | Open a `.sruja` file (e.g. `book/valid-examples/getting-started.sruja`). Keywords, strings, `->` should be colored. |
   | **Diagnostics** | In a .sruja file, introduce an error (e.g. duplicate ID or invalid ref). After a short delay (or on save), Problems panel and red squiggles should show. |
   | **Run validation** | With a .sruja file active: Command Palette → “Sruja: Run validation”. Message should say “no issues” or “N error(s), M warning(s)”. |
   | **Export to Markdown** | With a .sruja file active: Command Palette → “Sruja: Export to Markdown” (or right‑click → same). A markdown preview tab opens; “Save” writes a `.md` next to the file. |
   | **Snippets** | In a .sruja file, type `person` or `->` and accept the snippet suggestion. |
   | **Sruja Skills view** | In the Explorer sidebar, open “Sruja Skills”. This repo has `skills/sruja-architecture`; you should see SKILL.md, AGENTS.md, and rules. Click to open. |
   | **Commands** | Command Palette → “Sruja:” to see all commands (Open Skills Overview, List Rules, Copy Rule for AI, etc.). |

4. **If something fails:**  
   - **Lint/Export not running:** If using WASM, ensure `extension/wasm/` exists (run `npm run copy:assets`). If using CLI, confirm `sruja.lsp.path` or `sruja` on PATH.  
   - **No diagnostics:** Ensure the file has a `.sruja` extension and the document language is “Sruja”.  
   - **Debug output:** Run → Start Debugging (F5) and check the “Debug Console” in the *original* window for errors.

## Cursor “Open Preview” / “Error Generating Preview”

In Cursor (or VS Code), if you use a built-in **Open Preview** on a `.sruja` file and see:

- **“WASM parser failed: WASM functions not registered after 10 seconds”**
- **“Ensure WASM files are bundled (wasm/sruja.wasm and wasm/wasm_exec.js)”**

that preview expects a different (Go-based) WASM setup. This extension uses **Rust** WASM (`wasm/sruja_wasm.js` and `wasm/sruja_wasm_bg.wasm`). Use the extension’s own preview instead:

- **Command Palette** → **Sruja: Open Diagram Preview**, or  
- With a `.sruja` file active, click the **preview/diagram** icon in the editor title bar.

That command uses the bundled Rust WASM to export to Mermaid and renders it in a webview. No `sruja.wasm` or `wasm_exec.js` required.
