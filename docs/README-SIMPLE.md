# Sruja: Simple Explanation

A quick guide to understanding Sruja's three main concepts.

## 🎯 What is Sruja?

Sruja is a **text-based language for describing software architecture**. Instead of drawing diagrams, you write code that describes your system.

---

## 1️⃣ **Sruja Core Language**

**What it is:** The basic language syntax for describing architecture.

**Think of it like:** The grammar of English - the basic rules for how to write sentences.

### What You Can Describe:

```sruja
system PaymentService {
  container API {
    component PaymentProcessor
  }
  
  container Database {
    technology "PostgreSQL"
  }
  
  API -> Database "Reads/Writes"
}
```

**Core Language includes:**
- ✅ Systems, containers, components
- ✅ Relationships (arrows)
- ✅ Basic properties (technology, descriptions)
- ✅ Basic support for all 6 Well-Architected pillars

**Example:** Like learning to write "The cat sat on the mat" - it's the foundation.

---

## 2️⃣ **Extensions**

**What they are:** Optional add-ons that let you describe specialized aspects of architecture.

**Think of it like:** Specialized vocabularies - medical terms for doctors, legal terms for lawyers.

### When You Need Extensions:

**Without Extension (Core only):**
```sruja
system PaymentService {
  retry: 3
  timeout: "2s"
}
```

**With Resilience Extension:**
```sruja
system PaymentService {
  retry_policy ExponentialRetry {
    attempts: 3
    backoff: "exponential"
    base_delay: "100ms"
    max_delay: "5s"
  }
  
  circuit_breaker PaymentCB {
    failure_threshold: "50%"
    open_duration: "30s"
  }
}
```

**Available Extensions:**
- 🔄 **Resilience & Reliability** - Detailed retry, circuit breaker, failover
- ⚡ **Performance** - Latency, throughput, SLOs, scaling
- 💰 **Cost & FinOps** - Cost models, budgets, optimization
- 🔒 **Security** - Authentication, encryption, compliance
- 📊 **Observability** - Metrics, logging, tracing
- ✅ **Compliance & Governance** - Policies, rules, approvals

**Example:** Like adding medical terminology when writing about health - you can be more precise.

---

## 3️⃣ **Engines**

**What they are:** The programs that process your Sruja code and do useful things with it.

**Think of it like:** Tools in a workshop - each tool does a specific job.

### What Engines Do:

1. **Parse your code** → Understand what you wrote
2. **Validate it** → Check if it's correct
3. **Generate diagrams** → Create visual representations
4. **Check for problems** → Find issues before deployment
5. **Simulate behavior** → Predict how system will behave
6. **Generate code** → Create actual code from architecture
7. **Track changes** → Monitor architecture evolution

### Engine Categories:

#### **Core Engines** (Always Available)
- **DSL Parser** - Reads your Sruja code
- **Model Composer** - Builds the architecture model
- **Validation Engine** - Checks for errors
- **Diagram Generator** - Creates diagrams

#### **Pillar Engines** (By Well-Architected Pillar)
- **Operational Excellence** - Observability, monitoring, CI/CD
- **Security** - Threat modeling, compliance, IAM
- **Reliability** - Resilience, chaos engineering, failure analysis
- **Performance** - Latency analysis, optimization, capacity planning
- **Cost** - FinOps, cost modeling, optimization
- **Sustainability** - Carbon tracking, resource efficiency

#### **Cross-Pillar Engines** (Advanced Features)
- **AI Review Engine** - AI-powered architecture analysis
- **Simulation Engine** - Predict system behavior
- **Drift Detection** - Find architecture vs code mismatches
- **Code Generation** - Generate code from architecture

**Example:** Like having a spell-checker (validation engine), a translator (code generator), and a style guide (linting engine) for your architecture.

---

## 📊 How They Work Together

```
┌─────────────────┐
│  Your Sruja     │
│  Code (Core +   │
│  Extensions)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Core Engines   │
│  (Parser,       │
│   Validator)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Pillar Engines │
│  (Security,     │
│   Performance)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Output:        │
│  - Diagrams     │
│  - Validation   │
│  - Code         │
│  - Reports      │
└─────────────────┘
```

---

## 🎓 Simple Analogy

**Core Language** = The alphabet and basic grammar  
**Extensions** = Specialized vocabulary (medical, legal, technical terms)  
**Engines** = Tools that use your writing (spell-checker, translator, formatter)

---

## 🚀 Getting Started

1. **Start with Core** - Learn basic Sruja syntax
2. **Add Extensions** - When you need more detail
3. **Use Engines** - Let tools process your code

---

## 📚 Learn More

- [DSL Overview](./specs/dsl-overview.md) - Complete language guide
- [Core DSL](./pillars/core.md) - Basic pillar support
- [DSL Extensions](./specs/dsl-extensions.md) - All available extensions
- [Engines Catalog](./pillars/engines.md) - Complete engine list

---

*Think of Sruja as: **Language (Core + Extensions) + Tools (Engines) = Complete Architecture Platform***

