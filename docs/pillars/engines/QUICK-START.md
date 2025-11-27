# Engine Quick Start

**Don't be overwhelmed by 171 engines!** Here's what you actually need.

## 🎯 The Minimum (4 Engines)

To get started, you only need these 4:

1. **DSL Parser Engine** - Reads your Sruja code
2. **Global Model Composer** - Builds the architecture model
3. **Validation Engine** - Checks for errors
4. **Diagram Generator** - Creates diagrams

**That's it!** You can model architecture with just these.

---

## 📈 Recommended Progression

### Phase 1: Core Only (4 engines)
**What you can do:**
- Write architecture in DSL
- Generate diagrams
- Basic validation

**Time to learn:** 1-2 hours

---

### Phase 2: Add Basic Pillars (10-15 engines)
**Add 3-6 basic engines for pillars you care about:**

**If you care about Security:**
- Basic Authentication Engine
- Basic Encryption Engine
- Basic Security Tags Engine

**If you care about Performance:**
- Basic Latency Engine
- Basic Throughput Engine
- Basic Scaling Engine

**If you care about Reliability:**
- Basic Retry Engine
- Basic Timeout Engine
- Basic Circuit Breaker Engine

**What you can do:**
- Everything from Phase 1
- Basic quality checks
- Pillar-specific validation

**Time to learn:** 1-2 days

---

### Phase 3: Add Advanced Features (20-30 engines)
**Add advanced engines when you need them:**

**Examples:**
- Need threat modeling? → Add Threat Modeling Engine
- Need cost analysis? → Add Cost Modeling Engine
- Need AI reviews? → Add AI Review Engine
- Need simulation? → Add Simulation Engine

**What you can do:**
- Everything from Phase 2
- Advanced analysis
- Specialized features

**Time to learn:** 1-2 weeks

---

### Phase 4: Full Platform (50+ engines)
**Only for:**
- Large enterprises
- Complex architectures
- Full governance needs

**Most users never need this.**

---

## 🎓 Decision Tree

```
Start Here
    │
    ├─ Do I just want to document architecture?
    │   └─ Yes → Use 4 core engines (Phase 1)
    │
    ├─ Do I need basic quality checks?
    │   └─ Yes → Add 3-6 basic pillar engines (Phase 2)
    │
    ├─ Do I need advanced features?
    │   └─ Yes → Add specific advanced engines (Phase 3)
    │
    └─ Am I building enterprise platform?
        └─ Yes → Add all engines you need (Phase 4)
```

---

## 💡 Key Principles

1. **Start Minimal** - 4 engines is enough
2. **Add Incrementally** - Add when you need, not all at once
3. **Focus on Your Needs** - Don't enable everything
4. **Engines are Optional** - Except the 4 core ones
5. **You Control Complexity** - System grows with you

---

## 📋 Common Configurations

### Configuration 1: Documentation Only
```yaml
engines:
  - dsl-parser
  - model-composer
  - validation
  - diagram-generator
```
**Use case:** Just documenting architecture

---

### Configuration 2: Basic Quality
```yaml
engines:
  core: [all]
  pillars:
    security: [basic-authentication, basic-encryption]
    performance: [basic-latency, basic-throughput]
```
**Use case:** Documentation + basic checks

---

### Configuration 3: Focused Advanced
```yaml
engines:
  core: [all]
  pillars:
    security: [all]  # All security engines
    performance: [basic]  # Only basic performance
  cross-pillar:
    - ai-review
```
**Use case:** Security-focused with AI assistance

---

### Configuration 4: Enterprise
```yaml
engines:
  core: [all]
  pillars: [all]  # All pillar engines
  cross-pillar: [selected]  # Pick what you need
```
**Use case:** Full enterprise platform

---

## ❓ FAQ

### Q: Do I need to understand all 171 engines?
**A:** No! Start with 4, add as needed.

### Q: Can I skip engines?
**A:** Yes! Every engine is optional except the 4 core ones.

### Q: How do I know which engines to add?
**A:** Add engines when you encounter problems they solve.

### Q: Will I be overwhelmed?
**A:** No! Start minimal, add incrementally.

### Q: What if I add too many engines?
**A:** You can disable any engine except the 4 core ones.

---

## 🎯 Next Steps

1. **Start with 4 core engines** - [Core Engines](../engines.md#core-engines-basic---all-pillars)
2. **Add basic pillar engines** - Pick 1-2 pillars you care about
3. **Add advanced engines** - Only when you need them
4. **Read full guide** - [Getting Started Guide](../../GETTING-STARTED.md)

---

*Remember: The 171 engines are there when you need them, but you only need 4 to get started!*

