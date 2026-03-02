# How to Run Sruja (Step-by-Step)

This guide walks you through running the Sruja OSS project from a fresh clone. It covers CLI (required), optional demo, desktop app, VS Code extension, and evaluation.

---

## Prerequisites

- **Rust ≥ 1.70** – [rustup.rs](https://rustup.rs/)
- **Node.js ≥ 18** – Only needed for the VS Code extension and (optionally) npm-based tooling

Verify:

```bash
rustc --version   # e.g. 1.70+
node --version    # optional; only for extension
```

---

## Step 1: Clone and enter the repo

```bash
git clone https://github.com/sruja-ai/sruja.git
cd sruja
```

---

## Step 2: Install dependencies and build

```bash
# Fetch Rust dependencies
make install
# or: cargo fetch

# Build release binary (CLI)
make build
# or: cargo build --release
```

The CLI binary is at **`target/release/sruja`**.

---

## Step 3: Put the CLI on your PATH (optional but recommended)

**Option A – Use the built binary directly**

```bash
./target/release/sruja --version
```

**Option B – Install into Cargo’s bin (so `sruja` works anywhere)**

```bash
cargo install --path crates/sruja-cli
# Then: sruja --version
# Binary is in ~/.cargo/bin (ensure that’s on your PATH)
```

**Option C – Symlink**

```bash
sudo ln -sf "$(pwd)/target/release/sruja" /usr/local/bin/sruja
# or: ln -sf "$(pwd)/target/release/sruja" ~/.local/bin/sruja
```

---

## Step 4: First value (no config, no API keys)

Run architecture intelligence on the repo itself:

```bash
# If you used Option B or C:
sruja quickstart -r .

# Or with the built binary:
./target/release/sruja quickstart -r .
```

You should see: component inventory, health score, top findings, and next steps. No `.sruja` file or API keys required.

Other useful commands:

```bash
sruja drift -r .                    # Drift (cycles, orphans, layer violations)
sruja analyze -r .                  # Full analysis (structural + semantic + intent)
sruja lint examples/                # Validate .sruja files
sruja export markdown file.sruja    # Export to Markdown
```

---

## Step 5: Run the E2E demo (optional, ~2 min)

The demo clones Express.js (if needed), runs quickstart + drift, and optionally baseline/LLM eval.

```bash
cd evaluation/real-world-test
./run_demo.sh
```

- **No flags** – Fast path only (quickstart + drift). No config.
- **`--baseline`** – Also compare to an example architecture.
- **`--llm`** – Add LLM evaluation (requires an API key in `.env`; see Step 7).
- **`--all`** – Baseline + LLM.

If the script says “sruja CLI not found”, ensure `sruja` is on PATH or build from repo root first (`make build`) and add `target/release` to PATH.

---

## Step 6: Run the desktop app (optional)

The Slack-style desktop app (chat, agents, extraction) needs an LLM API key.

```bash
export OPENROUTER_API_KEY="sk-or-v1-..."   # or OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.
cargo run -p sruja-app
```

Or use a `.env` in the repo root with the same variables; the app may pick them up depending on how it’s started.

---

## Step 7: LLM / API keys (optional)

Only needed for:

- `sruja eval <path>`
- `./run_demo.sh --llm` or `./evaluate_architecture.sh <repo> --llm`
- **sruja-app** (desktop)

**Quick setup for evaluation/demo:**

```bash
cd evaluation/real-world-test
cp .env.example .env
# Edit .env and set one key, e.g.:
#   OPENROUTER_API_KEY=sk-or-v1-...
#   OPENAI_API_KEY=sk-...
#   ANTHROPIC_API_KEY=sk-ant-...
#   GEMINI_API_KEY=...
# Or local: SRUJA_LLM_PROVIDER=ollama
```

Then:

```bash
./run_demo.sh --llm
```

---

## Step 8: VS Code extension (optional)

For syntax highlighting and LSP (e.g. validation, autocomplete) for `.sruja` files:

```bash
cd extension
npm install
npm run compile
```

Then in VS Code/Cursor: **Run → Start Debugging** (F5) with “Extension” launch config to open a new window with the extension loaded.  
Or build a VSIX and install it: `npm run package` then install the generated `.vsix` from the Extensions view.

---

## Step 9: Run tests

```bash
make test
# or: cargo test --workspace
```

Other targets:

```bash
make fmt          # Format Rust code
make lint         # Clippy
make test-arch-intel   # Architecture intelligence E2E tests
```

---

## Step 10: Book (mdBook docs, optional)

```bash
make book-deps    # Install mdbook, mdbook-mermaid (one-time)
make wasm         # Needed for in-book WASM diagrams
make book         # Build book
make book-serve   # Serve at http://localhost:3000 (live reload)
```

---

## Summary: minimal path to “running” Sruja

| Step | Command | Purpose |
|------|---------|--------|
| 1 | `git clone ... && cd sruja` | Get repo |
| 2 | `make install && make build` | Dependencies + CLI binary |
| 3 | `./target/release/sruja --version` | Verify CLI |
| 4 | `./target/release/sruja quickstart -r .` | First value (no config) |
| 5 | `cd evaluation/real-world-test && ./run_demo.sh` | Optional demo |

**Troubleshooting**

- **“sruja: command not found”** – Use full path `./target/release/sruja` or add it to PATH (Step 3).
- **“Cargo not found”** – Install Rust: https://rustup.rs/
- **Demo fails** – Ensure CLI is built and on PATH; for `--llm`, set one LLM key in `evaluation/real-world-test/.env`.

For contribution workflow (hooks, lint, test), see [CONTRIBUTING.md](CONTRIBUTING.md) and [DEVELOPMENT.md](DEVELOPMENT.md).
