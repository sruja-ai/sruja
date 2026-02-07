### Summary

Provide a concise overview of the change and its motivation.

### Changes

- What was added/updated/removed
- Relevant files/modules touched

### Test Plan

- Rust: `cargo test`
- Sruja files: CI runs `sruja lint` on all `**/*.sruja` (or locally: `./target/release/sruja lint **/*.sruja`)
- Docs/book: verify any touched pages render correctly

### Checklist

- [ ] Linked issue or clear rationale
- [ ] Small, focused PR (prefer incremental changes)
- [ ] Documentation updated where relevant
- [ ] Code formatted (`cargo fmt` / `make fmt`) and passes `cargo fmt -- --check` and `cargo clippy`
- [ ] Tests pass (`make test`)
- [ ] No secrets/keys committed

### Impact Areas

- [ ] language/parser
- [ ] engine/validation
- [ ] lsp
- [ ] cli / export
- [ ] docs/book
- [ ] examples

### Risks & Rollback

- Potential risk(s)
- Rollback steps if needed
