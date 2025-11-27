# Adoption Guide

Everything you need to help teams adopt Sruja DSL successfully.

## Quick Links

- **[Quick Start](./quickstart.md)** - Get started in 5 minutes
- **[Installation Guide](./installation.md)** - Install Sruja DSL
- **[Adoption Strategy](./adoption-strategy.md)** - Complete adoption playbook

## For First-Time Users

### 1. Install (1 minute)

```bash
curl -sSL https://sruja.dev/install.sh | bash
```

### 2. Create Your First Project (30 seconds)

```bash
sruja init
```

### 3. View Your Architecture

```bash
sruja compile architecture.sruja
```

That's it! You're ready to go.

## For Team Leads

### Adoption Playbook

1. **Pilot Phase** (Week 1-2)
   - Install Sruja DSL
   - Model one system
   - Generate first diagram
   - Share with team

2. **Team Adoption** (Week 3-4)
   - Model all systems
   - Add metadata
   - Create journeys
   - Document ADRs

3. **Scale** (Month 2+)
   - Adopt plugins
   - Multi-file architectures
   - CI/CD integration
   - Automated validation

### Key Success Factors

- ✅ Zero-friction onboarding
- ✅ Visual feedback (diagrams)
- ✅ Git-friendly workflow
- ✅ Gradual adoption support
- ✅ Integration with existing tools

## For Developers

### Common Workflows

```bash
# Initialize new project
sruja init --template microservices

# Work on architecture
# Edit architecture.sruja file
# Get autocomplete and hover docs in editor

# Validate
sruja lint architecture.sruja

# Generate diagrams
sruja compile architecture.sruja --format d2

# Get explanations
sruja explain BillingAPI

# List elements
sruja list systems
```

### Editor Setup

**VSCode:**
1. Install "Sruja" extension
2. Open `.sruja` file
3. Get autocomplete, hover, and errors

**Other Editors:**
- Configure LSP client
- Connect to Sruja LSP server

## Templates

Start with proven patterns:

```bash
sruja init --template basic
sruja init --template microservices
sruja init --template event-driven
sruja init --template monolith
sruja init --template api-gateway
sruja init --template service-mesh
```

## Resources

- [Documentation](../README.md) - Complete documentation
- [Examples](../examples/) - Real-world examples
- [Patterns](../patterns/) - Architecture patterns
- [Plugins](../plugins/) - Available plugins

## Support

- 💬 [Community Discord](https://discord.gg/sruja)
- 🐛 [Issue Tracker](https://github.com/sruja-ai/sruja/issues)
- 📧 [Email](mailto:support@sruja.dev)

## Why Adopt Sruja DSL?

- ⚡ **Fast onboarding** - Get started in 5 minutes
- 🎨 **Beautiful diagrams** - Auto-generated visualizations
- ✨ **Smart autocomplete** - LSP-powered suggestions
- 📝 **Documentation** - Architecture as code
- 🔌 **Extensible** - Plugin ecosystem
- 🔄 **Git-friendly** - Version controlled
- 🚀 **Scalable** - Start small, grow naturally

---

**Ready to start?** → [Quick Start Guide](./quickstart.md)

