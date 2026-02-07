---
title: "Extending the CLI (Rust)"
weight: 80
summary: "The CLI is implemented in Rust with clap. Add or change subcommands in crates/sruja-cli."
tags: ["cli", "rust", "advanced"]
---

# Extending the CLI (Rust)

Sruja’s CLI lives in `crates/sruja-cli` and uses [clap](https://docs.rs/clap) for argument parsing. To add or change subcommands:

1. **Open** `crates/sruja-cli/src/main.rs` (or the relevant module) and see how existing commands (e.g. `lint`, `export`) are defined.
2. **Add a subcommand** using clap’s `Subcommand` enum and match on it in the main entrypoint; run your logic and return `Result` with `?` for errors.
3. **Run and test** with `cargo run -p sruja-cli -- <subcommand> ...`.

Shell completions are available:

```bash
sruja completion bash
sruja completion zsh
sruja completion fish
```

For patterns and conventions, see the repo’s [AGENTS.md](https://github.com/sruja-ai/sruja/blob/main/AGENTS.md) (Rust skills) and [docs/CODING_GUIDELINES.md](https://github.com/sruja-ai/sruja/blob/main/docs/CODING_GUIDELINES.md).
