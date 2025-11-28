### Summary

Provide a concise overview of the change and its motivation.

### Changes

- What was added/updated/removed
- Relevant files/modules touched

### Test Plan

- Go tests: `go test ./...`
- Hugo build: `hugo -D` builds without errors
- Docs preview: verify pages touched render correctly
- Sruja examples: any touched ` ```sruja` blocks compile via WASM (Run button produces SVG or clear error)

### Checklist

- [ ] Linked issue or clear rationale
- [ ] Small, focused PR (prefer incremental changes)
- [ ] Documentation updated where relevant
- [ ] Hugo builds locally without errors
- [ ] Touched ` ```sruja` examples compile via WASM
- [ ] Error messages include filename context when applicable
- [ ] No secrets/keys committed
- [ ] Code formatted (`go fmt ./...`) and passes CI

### Impact Areas

- [ ] language/parser
- [ ] engine/validation
- [ ] wasm/runtime
- [ ] tooling/cli
- [ ] docs/site
- [ ] examples

### Risks & Rollback

- Potential risk(s)
- Rollback steps if needed

### Screenshots / Demos (optional)

Include visuals for UI changes or example outputs.
