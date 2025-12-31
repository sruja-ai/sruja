---
title: "Lesson 2: Deployment Architecture"
weight: 2
summary: "Cloud, On-prem, Containers, and real-world deployment strategies."
---

# Lesson 2: Deployment Architecture

## Real-World Scenario: Startup to Scale

**Context**: A fintech startup begins with a single server, grows to 10M users, and needs to plan deployment architecture.

**Challenge**: How do you model and evolve deployment architecture as you scale?

## Logical vs. Physical Architecture

- **Logical Architecture:** The software components and how they interact (Containers, Components).
- **Physical Architecture:** Where those components actually run (Servers, VMs, Kubernetes Pods).

**Why it matters**: Understanding this separation helps you:

- Plan migrations (e.g., moving from EC2 to EKS)
- Model multi-cloud strategies
- Document disaster recovery plans
- Communicate with DevOps teams

## Deployment Strategies: Real-World Trade-offs

### On-Premises

Running on your own hardware in a data center.

**Real-world use cases:**

- Financial institutions (regulatory compliance)
- Healthcare systems (HIPAA data residency)
- Government systems (sovereignty requirements)

* **Pros:** Total control, security, data sovereignty.
* **Cons:** High maintenance, capital expense, slower scaling.
* **Cost example:** $500K+ initial investment, 3-5 year hardware refresh cycles

### Cloud (AWS, GCP, Azure)

Renting infrastructure from a provider.

**Real-world use cases:**

- SaaS platforms (99% of modern startups)
- E-commerce (seasonal scaling)
- Media streaming (global distribution)

* **Pros:** Pay-as-you-go, infinite scale, managed services.
* **Cons:** Vendor lock-in, variable costs, compliance complexity.
* **Cost example:** $5K/month for small SaaS, $50K+ for mid-size, $500K+ for enterprise

### Containers & Orchestration

Packaging code with dependencies (Docker) and managing them at scale (Kubernetes).

**Real-world adoption:**

- 2024: 70% of enterprises use Kubernetes
- Common pattern: Docker → Kubernetes → Service Mesh (Istio/Linkerd)

**Production considerations:**

- Resource limits (CPU/memory)
- Health checks and liveness probes
- Rolling updates and rollback strategies
- Multi-region deployments

---

## 🛠️ Sruja Perspective: Deployment Nodes

Sruja allows you to map your logical containers to physical deployment nodes, making it clear where code runs in production.

### Example: Multi-Region E-Commerce Platform

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
            min 3
            max 100
            metric "cpu > 70%"
        }
    }
    WebApp = container "React Frontend" {
        technology "React"
    }
    PrimaryDB = datastore "PostgreSQL" {
        technology "PostgreSQL"
    }
    Cache = datastore "Redis" {
        technology "Redis"
    }
}

// Production deployment across regions
deployment Production "Production Environment" {
    node AWS "AWS Cloud" {
        // Primary region: US-East-1
        node USEast1 "US-East-1 (Primary)" {
            node EKS "EKS Cluster" {
                containerInstance API {
                    replicas 10
                }
                containerInstance WebApp {
                    replicas 5
                }
            }
            node RDS "RDS Multi-AZ" {
                containerInstance PrimaryDB {
                    role "primary"
                }
            }
            node ElastiCache "ElastiCache" {
                containerInstance Cache
            }
        }

        // Secondary region: EU-West-1 (for DR and latency)
        node EUWest1 "EU-West-1 (Secondary)" {
            node EKS "EKS Cluster" {
                containerInstance API {
                    replicas 5
                }
                containerInstance WebApp {
                    replicas 3
                }
            }
            node RDS "RDS Read Replica" {
                containerInstance PrimaryDB {
                    role "read-replica"
                }
            }
        }
    }
}

view index {
include *
}
```

### DevOps Integration: CI/CD Pipeline

Model your deployment pipeline alongside architecture:

```sruja
element person
element system
element container
element component
element datastore
element queue

CICD = system "CI/CD System" {
    GitHubActions = container "GitHub Actions" {
        description "Triggers on push to main branch"
    }
    BuildService = container "Build Service" {
        technology "Docker"
        description "Builds container images"
    }
    TestRunner = container "Test Runner" {
        description "Runs unit, integration, and E2E tests"
    }
    DeployService = container "Deploy Service" {
        technology "ArgoCD"
        description "Deploys to Kubernetes clusters"
    }

    GitHubActions -> BuildService "Builds image"
    BuildService -> TestRunner "Runs tests"
    TestRunner -> DeployService "Deploys if tests pass"
}

// Link deployment to CI/CD
// Note: Deployment metadata (CI/CD pipeline, strategy) is modeled in the deployment node
deployment Production "Production Deployment" {
    node Infrastructure "Production Infrastructure" {
    }
}

view index {
include *
}
```

## Real-World Patterns

### Pattern 1: Blue/Green Deployment

**Use case**: Zero-downtime deployments for critical services

```sruja
deployment Production {
    node Blue "Active Environment" {
        containerInstance API {
            traffic 100
        }
    }
    node Green "Staging Environment" {
        containerInstance API {
            traffic 0
            status "ready-for-switch"
        }
    }
}
```

**DevOps workflow:**

1. Deploy new version to Green
2. Run smoke tests
3. Switch 10% traffic to Green
4. Monitor for 15 minutes
5. Gradually increase to 100%
6. Keep Blue ready for rollback

### Pattern 2: Canary Deployment

**Use case**: Gradual rollout with automatic rollback

```sruja
deployment Production {
    node Canary "Canary Cluster" {
        containerInstance API {
            traffic 5
            description "5% of traffic, auto-rollback on error rate > 1%"
        }
    }
    node Stable "Stable Cluster" {
        containerInstance API {
            traffic 95
        }
    }
}
```

### Pattern 3: Multi-Cloud Strategy

**Use case**: Avoiding vendor lock-in, disaster recovery

```sruja
deployment Production {
    node AWS "AWS Primary" {
        containerInstance API {
            region "us-east-1"
            traffic 80
        }
    }
    node GCP "GCP Secondary" {
        containerInstance API {
            region "us-central1"
            traffic 20
            description "Failover target"
        }
    }
}
```

## Service Level Objectives (SLOs)

Define reliability targets directly in your architecture model:

```sruja
element system
element container
element datastore

ECommerce = system "E-Commerce Platform" {
API = container "REST API" {
  technology "Go"

  // Define SLOs for production monitoring
  slo {
    availability {
      target "99.99%"
      window "30 days"
      current "99.95%"
      description "Four nines availability target"
    }
    latency {
      p95 "200ms"
      p99 "500ms"
      window "7 days"
      current {
        p95 "180ms"
        p99 "450ms"
      }
    }
    errorRate {
      target "< 0.1%"
      window "30 days"
      current "0.05%"
    }
    throughput {
      target "1000 req/s"
      window "1 hour"
      current "950 req/s"
    }
  }
}

Database = datastore "PostgreSQL" {
  technology "PostgreSQL"
  slo {
    availability {
      target "99.9%"
      window "30 days"
    }
    latency {
      p95 "50ms"
      p99 "100ms"
    }
  }
}
}

view index {
include *
}
```

### Benefits of Modeling SLOs

1. **Clear Targets**: Everyone knows what "good" looks like
2. **Monitoring Guidance**: SLOs define what metrics to track
3. **Stakeholder Communication**: Clear reliability commitments
4. **Living Documentation**: SLOs live with architecture, not separate docs

## Monitoring & Observability

Model your observability stack:

```sruja
element system
element container

Observability = system "Observability Stack" {
Prometheus = container "Metrics" {
  technology "Prometheus"
  description "Collects metrics from all services"
}
Grafana = container "Dashboards" {
  technology "Grafana"
  description "Visualizes metrics and alerts"
}
ELK = container "Logging" {
  technology "Elasticsearch, Logstash, Kibana"
  description "Centralized logging"
}
Jaeger = container "Tracing" {
  technology "Jaeger"
  description "Distributed tracing"
}
}

// Link to your services
ECommerce.API -> Observability.Prometheus "Exposes metrics"
ECommerce.API -> Observability.ELK "Sends logs"
ECommerce.API -> Observability.Jaeger "Sends traces"
```

## Key Takeaways

1. **Separate logical from physical**: Model what your system does (logical) separately from where it runs (physical)
2. **Document deployment strategies**: Use deployment nodes to show Blue/Green, Canary, or multi-region setups
3. **Link to CI/CD**: Show how code flows from commit to production
4. **Model observability**: Include monitoring, logging, and tracing in your architecture
5. **Plan for scale**: Document scaling strategies (min/max replicas, regions)

## Exercise: Model Your Deployment

1. Choose a real system you work on (or a hypothetical one)
2. Model the logical architecture (containers, datastores)
3. Map to physical deployment (cloud regions, clusters)
4. Add deployment strategy (Blue/Green, Canary, or Rolling)
5. Include observability components

**Time**: 20 minutes

## Further Reading

- Tutorial: [Deployment Modeling](/tutorials/advanced/deployment-modeling)
- Docs: [Deployment Concepts](/docs/concepts/deployment)
- Course: [E-Commerce Platform - Module 5: Ops](/courses/ecommerce-platform/module-5-ops)
