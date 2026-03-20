# How to Run Sruja (Step-by-Step)

This guide walks you through running the Sruja OSS project from a fresh clone. It covers CLI (required), optional demo, VS Code extension, evaluation, and the mdBook site.

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

Run context engineering on the repo itself:

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
sruja quickstart -r .                # Quick architecture overview (inventory, health score, findings)
# For full analysis: sruja drift -r . -a architecture.sruja (vs baseline) or sruja runtime analyze -t <trace_file>
sruja lint book/valid-examples/*.sruja # Validate canonical book examples
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

### Optional: Context Engineering microservices demo (~2 min)

This demo walks through the full flow: **intent (rulebook) → scan → drift → analyze → AI ask**, using the small Python microservices in `demo/`.

```bash
make demo-intel
# or: cd demo && ./run_demo.sh
```

- **No API key** – Steps 1–4 run; step 5 (AI ask) is skipped with a hint, and `sruja why` is run as a deterministic fallback when possible.
- **With API key** – Set `OPENROUTER_API_KEY` or `OPENAI_API_KEY` in repo root `.env` to enable the full AI ask step.

See `demo/README.md` for details.

---

## Step 6: LLM / API keys (optional)

Only needed for:

- `sruja eval <path>`
- `./run_demo.sh --llm` or `./evaluate_architecture.sh <repo> --llm`

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

## Step 7: VS Code extension (optional)

For syntax highlighting and LSP (e.g. validation, autocomplete) for `.sruja` files:

```bash
cd extension
npm install
npm run compile
```

Then in VS Code/Cursor: **Run → Start Debugging** (F5) with “Extension” launch config to open a new window with the extension loaded.  
Or build a VSIX and install it: `npm run package` then install the generated `.vsix` from the Extensions view.

---

## Step 8: Run tests

```bash
make test
# or: cargo test --workspace
```

Other targets:

```bash
make fmt          # Format Rust code
make lint         # Clippy
cargo test -p sruja-cli --test why_e2e   # Why command E2E (optional)
```

---

## Step 9: Book (mdBook docs, optional)

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
| 5a | `cd evaluation/real-world-test && ./run_demo.sh` | Optional: E2E demo (quickstart + drift) |
| 5b | `make demo-intel` | Optional: Context Engineering demo (intent → scan → drift → analyze → AI) |

**Troubleshooting**

- **“sruja: command not found”** – Use full path `./target/release/sruja` or add it to PATH (Step 3).
- **“Cargo not found”** – Install Rust: https://rustup.rs/
- **Demo fails** – Ensure CLI is built and on PATH; for `--llm`, set one LLM key in `evaluation/real-world-test/.env`.

For contribution workflow (hooks, lint, test), see [CONTRIBUTING.md](CONTRIBUTING.md) and [DEVELOPMENT.md](DEVELOPMENT.md).
