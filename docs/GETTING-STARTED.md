# Getting Started with Sruja

**Don't worry about 171 engines!** You only need a few to get started.

## 🎯 The Truth About Engines

**You don't need all 171 engines.** Most are optional and advanced.

### What You Actually Need

#### ✅ **Phase 1: Just Getting Started** (4 engines)
1. **DSL Parser Engine** - Reads your code
2. **Model Composer** - Builds the architecture model
3. **Validation Engine** - Checks for errors
4. **Diagram Generator** - Creates diagrams

**That's it!** You can model architecture with just these 4.

---

#### ✅ **Phase 2: Basic Usage** (Add 3-5 more)
When you need more features, add:
- **Two-Way Sync Engine** - Edit diagrams and code
- **Reference Resolution** - Link between files
- **Basic Validation Engines** - Per-pillar checks (optional)

**Total: ~10 engines** for most use cases.

---

#### ✅ **Phase 3: Advanced Features** (Add as needed)
Only add engines when you need them:
- **Security Engine** - Only if you need threat modeling
- **Cost Engine** - Only if you need cost analysis
- **Simulation Engine** - Only if you need behavior prediction
- **AI Review Engine** - Only if you want AI assistance

**Most users never need more than 20-30 engines.**

---

## 📊 Engine Organization

### Core Engines (19 total)
**Always available, minimal overhead**

These provide basic functionality:
- Parsing, validation, diagram generation
- Basic support for all 6 pillars
- Essential features only

**You get these automatically** - no configuration needed.

---

### Pillar Engines (By Need)

#### Operational Excellence (20 engines)
**Start with:** Basic Health Check, Basic Metrics, Basic Logging (3 engines)  
**Add later:** Observability, CI/CD, Monitoring (17 advanced engines)

#### Security (11 engines)
**Start with:** Basic Authentication, Basic Encryption, Basic Tags (3 engines)  
**Add later:** Threat Modeling, Compliance, IAM (8 advanced engines)

#### Reliability (24 engines)
**Start with:** Basic Retry, Basic Timeout, Basic Circuit Breaker (3 engines)  
**Add later:** Chaos Engineering, Failure Analysis, Resilience (21 advanced engines)

#### Performance (17 engines)
**Start with:** Basic Latency, Basic Throughput, Basic Scaling (3 engines)  
**Add later:** Optimization, Capacity Planning, Bottleneck Detection (14 advanced engines)

#### Cost (11 engines)
**Start with:** Basic Cost Tracking, Basic Cost Tags (2 engines)  
**Add later:** FinOps, Cost Modeling, Optimization (9 advanced engines)

#### Sustainability (11 engines)
**Start with:** Basic Resource Efficiency, Basic Carbon Tracking (2 engines)  
**Add later:** Carbon Accounting, Green Computing, Optimization (9 advanced engines)

---

### Cross-Pillar Engines (65 engines)
**All optional, add only when needed**

These are advanced features:
- AI-powered analysis
- Simulation and prediction
- Code generation
- Advanced governance

**Most users never need these.**

---

## 🚀 Recommended Paths

### Path 1: "I Just Want to Document Architecture"
**Engines needed:** 4-7

1. DSL Parser
2. Model Composer
3. Validation Engine
4. Diagram Generator
5. Two-Way Sync (optional)
6. Reference Resolution (optional)
7. Basic Validation (optional)

**That's it!** You can document architecture with just these.

---

### Path 2: "I Want Basic Quality Checks"
**Engines needed:** 10-15

Add to Path 1:
- Basic engines for your relevant pillars (3-6 engines)
- Visual Validation Overlays
- Drift Detector (optional)

**Example:** If you care about security, add Basic Security engines (3). If you care about performance, add Basic Performance engines (3).

---

### Path 3: "I Want Enterprise Features"
**Engines needed:** 20-40

Add to Path 2:
- Advanced engines for your pillars (10-20 engines)
- Cross-pillar engines you need (5-10 engines)

**Example:** 
- Security-focused org → Add Security engines (8 advanced)
- Performance-critical → Add Performance engines (14 advanced)
- Cost-conscious → Add Cost engines (9 advanced)

---

### Path 4: "I Want Everything"
**Engines needed:** 100+

Only for:
- Large enterprises
- Complex architectures
- Full governance needs
- Research/development

**Most users never need this.**

---

## 🎓 Learning Path

### Week 1: Core Only
- Learn basic DSL syntax
- Use 4 core engines
- Create simple architectures
- Generate diagrams

### Week 2-4: Add Basic Pillars
- Pick 1-2 pillars you care about
- Add 3-6 basic engines
- Learn validation
- Understand quality checks

### Month 2-3: Add Advanced Features
- Add advanced engines for your pillars
- Explore cross-pillar features
- Use AI/simulation if needed

### Month 4+: Full Platform
- Use all engines you need
- Customize for your org
- Build plugins if needed

---

## 💡 Key Principles

### 1. **Start Minimal**
You only need 4 engines to begin.

### 2. **Add Incrementally**
Add engines when you need them, not all at once.

### 3. **Focus on Your Pillars**
Don't enable all pillars - only the ones you care about.

### 4. **Engines are Optional**
Every engine is optional except the 4 core ones.

### 5. **You Control Complexity**
The system is as simple or complex as you make it.

---

## 📋 Quick Reference

### Minimum Setup (4 engines)
```
✅ DSL Parser
✅ Model Composer
✅ Validation Engine
✅ Diagram Generator
```

### Typical Setup (10-15 engines)
```
✅ 4 Core engines
✅ 3-6 Basic pillar engines (your choice)
✅ 2-3 Utility engines (sync, references)
✅ 1-2 Advanced engines (if needed)
```

### Enterprise Setup (30-50 engines)
```
✅ 4 Core engines
✅ 6-12 Basic pillar engines (all pillars)
✅ 10-20 Advanced pillar engines (your focus areas)
✅ 5-10 Cross-pillar engines (AI, simulation, etc.)
```

---

## 🎯 Decision Framework

### "Do I need this engine?"

Ask yourself:
1. **Do I have this problem?** If no → Skip
2. **Is basic support enough?** If yes → Use basic engine
3. **Do I need advanced features?** If yes → Add advanced engine
4. **Is this critical?** If no → Add later

### Example Questions

**Q: Do I need the Security Threat Modeling Engine?**  
A: Only if you need STRIDE/LINDDUN threat modeling. Basic security validation might be enough.

**Q: Do I need the Cost Modeling Engine?**  
A: Only if you need detailed cost analysis. Basic cost tracking might be enough.

**Q: Do I need the AI Review Engine?**  
A: Only if you want AI-powered architecture reviews. Manual review might be enough.

**Q: Do I need the Simulation Engine?**  
A: Only if you need to predict system behavior. Static analysis might be enough.

---

## 🔧 Configuration Example

### Minimal Configuration
```yaml
engines:
  core:
    - parser
    - composer
    - validator
    - diagram-generator
```

### Typical Configuration
```yaml
engines:
  core:
    - parser
    - composer
    - validator
    - diagram-generator
    - two-way-sync
    - reference-resolution
  
  pillars:
    security:
      - basic-authentication
      - basic-encryption
    performance:
      - basic-latency
      - basic-throughput
```

### Enterprise Configuration
```yaml
engines:
  core: [all]
  
  pillars:
    security: [all]
    performance: [all]
    reliability: [all]
  
  cross-pillar:
    - ai-review
    - simulation
    - code-generation
```

---

## 📚 Next Steps

1. **Start with Core** - [Core DSL Guide](./pillars/core.md)
2. **Pick Your Pillars** - [Pillars Overview](./pillars/README.md)
3. **Add Engines Gradually** - [Engines Catalog](./pillars/engines.md)
4. **Learn Extensions** - [DSL Extensions](./specs/dsl-extensions.md)

---

## ❓ FAQ

### Q: Do I need all 171 engines?
**A:** No! Most users need 10-20 engines. Only enterprises need 50+.

### Q: Can I start with just core?
**A:** Yes! 4 engines is enough to get started.

### Q: How do I know which engines to add?
**A:** Add engines when you encounter problems they solve.

### Q: Are engines expensive?
**A:** Most engines are lightweight. Only advanced engines (AI, simulation) have overhead.

### Q: Can I disable engines?
**A:** Yes! Every engine is optional except the 4 core ones.

### Q: Will I be overwhelmed?
**A:** No! Start minimal, add incrementally. The system grows with you.

---

## 🎉 Remember

**Sruja is designed to grow with you:**
- Start simple (4 engines)
- Add complexity as needed
- Focus on what matters to you
- Ignore what you don't need

**You're in control of the complexity!**

---

*The 171 engines are there when you need them, but you only need a few to get started.*

