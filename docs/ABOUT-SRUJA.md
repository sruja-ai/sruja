# About Sruja

**Sruja** is a general-purpose domain-specific language for software systems architecture design.

## What is Sruja?

Sruja is a **text-based architecture modeling language** that lets you:
- Describe software systems in code
- Model architecture at multiple levels (VHLD, HLD, LLD)
- Validate architecture against best practices
- Generate diagrams (Mermaid)
- Track architecture evolution
- Support any software system (cloud, on-premise, hybrid, distributed)

## Well-Architected Framework

Sruja uses the **Well-Architected Framework** pillars as an organizational structure:

1. **Operational Excellence** - Running and monitoring systems
2. **Security** - Protecting data and systems
3. **Reliability** - Recovering from failures
4. **Performance Efficiency** - Using resources efficiently
5. **Cost Optimization** - Avoiding unneeded cost
6. **Sustainability** - Minimizing environmental impact

**Important**: These pillars are **general software architecture principles**, not tied to any specific cloud provider or platform.

## Platform Agnostic

Sruja works for **any software system**:

### ✅ Cloud Architectures
- AWS
- Google Cloud Platform (GCP)
- Microsoft Azure
- Multi-cloud
- Cloud-native

### ✅ On-Premise Systems
- Traditional data centers
- Private clouds
- Enterprise systems

### ✅ Hybrid Architectures
- Cloud + on-premise
- Edge computing
- Distributed systems

### ✅ Any Software System
- Microservices
- Monoliths
- Serverless
- Containers
- Event-driven
- Distributed systems

## Not Cloud-Specific

**Sruja is NOT:**
- ❌ Tied to AWS
- ❌ Cloud-provider specific
- ❌ Platform-dependent
- ❌ Infrastructure-as-code tool

**Sruja IS:**
- ✅ General-purpose architecture tool
- ✅ Platform-agnostic
- ✅ Works for any software system
- ✅ Architecture-as-code language

## Why Well-Architected Framework?

The Well-Architected Framework provides a **proven structure** for comprehensive architecture modeling:

- **Comprehensive**: Covers all aspects of software architecture
- **Proven**: Based on industry best practices
- **Structured**: Clear organization of concerns
- **Extensible**: Easy to add custom extensions

We use it as an **organizational framework**, not because we're tied to AWS or any cloud provider.

## Example Use Cases

### Cloud Architecture (AWS)
```sruja
system PaymentService {
  container API {
    technology "AWS Lambda"
  }
  container Database {
    technology "Amazon DynamoDB"
  }
}
```

### Cloud Architecture (GCP)
```sruja
system PaymentService {
  container API {
    technology "Cloud Functions"
  }
  container Database {
    technology "Cloud Firestore"
  }
}
```

### On-Premise Architecture
```sruja
system PaymentService {
  container API {
    technology "Spring Boot"
  }
  container Database {
    technology "PostgreSQL"
  }
}
```

### Hybrid Architecture
```sruja
system PaymentService {
  container API {
    technology "Kubernetes"
    deployment "on-premise"
  }
  container Database {
    technology "AWS RDS"
    deployment "cloud"
  }
}
```

**All work the same way!** Sruja doesn't care about the platform.

---

*Sruja is a general-purpose software architecture tool that works for any system, any platform, any deployment model.*

