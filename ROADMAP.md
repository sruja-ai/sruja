# Sruja Roadmap

This roadmap outlines the planned development of Sruja. Dates are approximate and subject to change based on community feedback and contributions.

---

## Current Focus (v0.17.x)

**Theme: Polish and Stability**

- [ ] Complete test coverage per [TEST_COVERAGE_PLAN.md](docs/internal/TEST_COVERAGE_PLAN.md)
- [ ] Improve error messages for common DSL mistakes
- [ ] VS Code extension stability improvements
- [ ] Documentation refinements

---

## Near-Term (v0.18 - v0.19)

**Theme: Developer Experience**

### CLI Enhancements
- [ ] `sruja init` - Interactive project initialization
- [ ] `sruja watch` - File watcher for live validation
- [ ] Shell completions (bash, zsh, fish, powershell)
- [ ] Progress indicators for long-running operations

### Language Features
- [ ] Custom validation rules via configuration
- [ ] Tags and metadata on elements
- [ ] Views/filters for large architectures
- [ ] Include/import for multi-file architectures

### Export Improvements
- [ ] PlantUML export format
- [ ] Structurizr export format
- [ ] Customizable Mermaid themes
- [ ] SVG/PNG export via CLI

---

## Medium-Term (v0.20 - v0.22)

**Theme: AI Integration**

### Discovery & Analysis
- [ ] Enhanced multi-language code scanning
- [ ] Framework-specific patterns (React, Spring, Django, etc.)
- [ ] Incremental discovery for large repos
- [ ] Confidence scores for detected components

### Drift Detection
- [ ] PR-integrated drift reports
- [ ] Historical drift tracking
- [ ] Automated architecture baseline updates
- [ ] GitHub App for continuous monitoring

### Context Export
- [ ] Optimized context for different LLM providers
- [ ] Architecture-aware code context
- [ ] Integration with popular AI coding assistants

---

## Long-Term (v1.0+)

**Theme: Enterprise & Scale**

### Federation
- [ ] Multi-repo architecture federation
- [ ] Architecture registry
- [ ] Cross-repo dependency visualization

### Collaboration
- [ ] Architecture review workflows
- [ ] Commenting and annotations
- [ ] Version comparison and diffing

### Governance
- [ ] Architecture decision records (ADRs) integration
- [ ] Compliance checking (SOC2, GDPR patterns)
- [ ] Architecture metrics and dashboards

---

## Future Considerations

Ideas we're exploring but haven't committed to:

- Real-time collaborative editing
- Architecture simulation and what-if analysis
- Integration with cloud infrastructure tools (Terraform, Pulumi)
- Architecture cost estimation
- Performance impact analysis

---

## Contributing to the Roadmap

We welcome community input on priorities:

1. **Upvote issues**: React with 👍 on GitHub issues you care about
2. **Open discussions**: Use [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions) for feature requests
3. **Submit PRs**: We welcome contributions for any roadmap items
4. **Sponsor**: Funded development accelerates roadmap delivery

---

## Version Cadence

- **Patch releases** (0.17.x): Bug fixes, minor improvements
- **Minor releases** (0.x.0): New features, enhancements
- **Major releases** (x.0.0): Breaking changes (rare)

See [VERSIONING.md](VERSIONING.md) for details.
