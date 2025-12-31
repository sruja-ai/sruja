---
title: "Lesson 1: Deployment Strategies"
weight: 1
summary: "Production-ready deployment strategies: Blue/Green, Canary, and real-world patterns."
---

# Lesson 1: Deployment Strategies

## Real-World Problem: Black Friday Deployment

**Scenario**: You need to deploy a critical payment fix on Black Friday morning. The system handles $10M/hour in transactions. How do you deploy without risking downtime?

**Wrong approach**: Deploy directly to production and hope nothing breaks.

**Right approach**: Use a proven deployment strategy that minimizes risk.

## Why Deployment Strategies Matter

**Industry statistics:**

- 60% of outages are caused by bad deployments (Gartner, 2023)
- Average cost of downtime: $5,600/minute for large enterprises
- 99.9% uptime = 8.76 hours downtime/year (still too much for critical systems)

**Product team perspective**: Every minute of downtime means lost revenue, frustrated customers, and damaged reputation.

**DevOps perspective**: Need automated, repeatable, safe deployment processes.

## Blue/Green Deployment

### Concept

You have two identical environments (Blue and Green). One is live, the other is idle. You deploy to the idle one, test it, and then switch traffic.

### Real-World Example: E-Commerce Platform

```sruja
element person
element system
element container
element component
element datastore
element queue

ECommerce = system "E-Commerce Platform" {
    API = container "REST API" {
        technology "Go"
        scale {
            min 10
            max 200
        }
    }
    PaymentService = container "Payment Service" {
        technology "Go"
        description "Critical: Processes all payments"
    }
    OrderDB = datastore "Order Database" {
        technology "PostgreSQL"
    }
}

deployment Production "Production Environment" {
    node Blue "Active Cluster (Blue)" {
        containerInstance API {
            replicas 50
            traffic 100
            status "active"
        }
        containerInstance PaymentService {
            replicas 20
            traffic 100
        }
        containerInstance OrderDB {
            role "primary"
        }
    }

    node Green "Staging Cluster (Green)" {
        containerInstance API {
            replicas 50
            traffic 0
            status "ready"
        }
        containerInstance PaymentService {
            replicas 20
            traffic 0
            status "ready"
        }
        containerInstance OrderDB {
            role "standby"
            description "Synced from Blue, ready for switch"
        }
    }
}

view index {
include *
}
```

### DevOps Workflow

1. **Deploy to Green**: Deploy new version to idle Green environment
2. **Smoke Tests**: Run automated health checks and integration tests
3. **Load Testing**: Verify Green can handle production load
4. **Switch Traffic**: Use load balancer to route 100% traffic to Green
5. **Monitor**: Watch metrics for 30 minutes
6. **Rollback Plan**: Keep Blue ready for instant rollback if issues occur

### When to Use Blue/Green

✅ **Good for:**

- Critical services (payment, authentication)
- Stateful applications with database replication
- Zero-downtime requirements
- Large, infrequent deployments

❌ **Not ideal for:**

- Frequent small deployments (wasteful)
- Stateless services (Canary is better)
- Limited infrastructure budget

### Cost Consideration

**Example**: Running duplicate production environment

- Cost: 2x infrastructure during deployment window
- Typical window: 1-2 hours
- Trade-off: Higher cost for lower risk

## Canary Deployment

### Concept

You roll out the new version to a small percentage of users (e.g., 5%) and monitor for errors. Gradually increase if metrics look good.

### Real-World Example: API Service

```sruja
element person
element system
element container
element component
element datastore
element queue

API = system "REST API" {
    APIv1 = container "API v1.2.3" {
        technology "Go"
        description "Current stable version"
    }
    APIv2 = container "API v1.2.4" {
        technology "Go"
        description "New version with performance improvements"
    }
}

deployment Production {
    node Canary "Canary Cluster" {
        containerInstance APIv2 {
            replicas 2
            traffic 5
            description "5% of traffic, monitoring error rate"
            metadata {
                maxErrorRate "1%"
                rollbackTrigger "error_rate > 1% or latency_p95 > 500ms"
            }
        }
    }

    node Stable "Stable Cluster" {
        containerInstance APIv1 {
            replicas 38
            traffic 95
        }
    }
}

view index {
include *
}
```

### Gradual Rollout Strategy

Document the rollout plan in metadata:

```sruja
element system
element container

ECommerce = system "E-Commerce Platform" {
API = container "API Service" {
  metadata {
    deploymentStrategy "Canary"
    rolloutSteps "5% → 25% → 50% → 100%"
    stepDuration "15 minutes per step"
    monitoringWindow "15 minutes between steps"
    rollbackCriteria "error_rate > 1% OR latency_p95 > 500ms OR cpu > 90%"
  }
}
}

view index {
include *
}
```

### Real-World Rollout Timeline

**Example: Deploying new API version**

```
10:00 AM - Deploy to Canary (5% traffic)
10:15 AM - Monitor: Error rate 0.2%, Latency p95: 180ms ✅
10:15 AM - Increase to 25% traffic
10:30 AM - Monitor: Error rate 0.3%, Latency p95: 195ms ✅
10:30 AM - Increase to 50% traffic
10:45 AM - Monitor: Error rate 0.4%, Latency p95: 210ms ✅
10:45 AM - Increase to 100% traffic
11:00 AM - Deployment complete
```

### When to Use Canary

✅ **Good for:**

- Stateless services
- Frequent deployments (multiple per day)
- A/B testing new features
- Performance-sensitive changes
- Limited infrastructure budget

❌ **Not ideal for:**

- Database schema changes (requires coordination)
- Breaking API changes (incompatible versions)
- Services with complex state

## Rolling Deployment

### Concept

Gradually replace old instances with new ones, one at a time.

```sruja
deployment Production {
    node Cluster "Kubernetes Cluster" {
        containerInstance API {
            replicas 20
            strategy "rolling"
            maxUnavailable 1
            maxSurge 2
            description "Replace 1 pod at a time, max 1 unavailable"
        }
    }
}
```

### When to Use Rolling

✅ **Good for:**

- Kubernetes-native deployments
- Stateless microservices
- Cost-effective (no duplicate infrastructure)
- Automated rollback via health checks

## Feature Flags: Deployment Strategy Alternative

Sometimes you don't need a deployment strategy—use feature flags instead:

```sruja
element system
element container

Platform = system "Platform" {
FeatureFlags = container "Feature Flag Service" {
  technology "LaunchDarkly, Split.io"
  description "Controls feature rollout without deployment"
}

API = container "API Service" {
  metadata {
    featureFlags {
      newPaymentFlow "10% rollout"
      experimentalSearch "5% rollout"
    }
  }
}
}

view index {
include *
}
```

**Use case**: Deploy code with new feature disabled, then gradually enable via feature flags.

## Monitoring During Deployment

Model your observability during deployments:

```sruja
element system
element container

Observability = system "Observability Stack" {
Prometheus = container "Metrics" {
  description "Tracks error rate, latency, throughput during deployment"
}
AlertManager = container "Alerting" {
  description "Alerts on deployment issues"
}
}

// Link monitoring to deployment
deployment Production {
    metadata {
        monitoring {
            metrics ["error_rate", "latency_p95", "cpu_usage", "request_rate"]
            alertThresholds {
                errorRate "> 1%"
                latencyP95 "> 500ms"
                cpuUsage "> 90%"
            }
            rollbackAutomation true
        }
    }
}
```

## Real-World Case Study: Netflix Canary Deployment

**Challenge**: Deploy to 100M+ users without downtime

**Solution**:

- Canary deployment to 1% of users
- Automated analysis of 50+ metrics
- Automatic rollback if any metric degrades
- Gradual rollout over 6 hours

**Result**: 99.99% deployment success rate

## Key Takeaways

1. **Choose the right strategy**: Blue/Green for critical, Canary for frequent, Rolling for cost-effective
2. **Automate everything**: Use CI/CD pipelines to automate deployment and rollback
3. **Monitor aggressively**: Track error rates, latency, and resource usage during deployment
4. **Have a rollback plan**: Always be ready to rollback within minutes
5. **Document in Sruja**: Model your deployment strategy so teams understand the process

## Exercise: Design a Deployment Strategy

**Scenario**: You're deploying a new checkout flow for an e-commerce platform. The system processes $1M/hour.

**Tasks:**

1. Choose a deployment strategy (Blue/Green, Canary, or Rolling)
2. Model it in Sruja with deployment nodes
3. Add monitoring and rollback criteria
4. Document the rollout timeline

**Time**: 15 minutes

## Further Reading

- Tutorial: [Deployment Modeling](/tutorials/advanced/deployment-modeling)
- Course: [System Design 101 - Module 4: Production Readiness](/courses/system-design-101/module-4-production-readiness)
- Docs: [Deployment Concepts](/docs/concepts/deployment)
