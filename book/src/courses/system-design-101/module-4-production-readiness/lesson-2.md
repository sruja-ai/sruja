---
title: "Lesson 2: Deployment Architecture"
weight: 2
summary: "Cloud, On-prem, Containers, and real-world deployment strategies."
---

# Lesson 2: Deployment Architecture

It was 4:47 PM on a Friday. I pushed the deploy button.

What could go wrong? It was just a "small database migration." Add a column, update some queries, deploy the API. Done in 10 minutes, right?

By 5:15 PM, the entire platform was down. The migration had locked the database. Every API request was timing out. Customers were calling support. The CEO was texting me. And I couldn't rollback because the migration had partially completed.

We were down for 3 hours and 42 minutes.

That Friday taught me more about deployment architecture than the previous 5 years combined. **How you deploy matters as much as what you deploy.** A great architecture deployed poorly will fail. A mediocre architecture deployed well will survive.

This lesson is about deployment architecture: how to model it, how to choose strategies, and how to avoid becoming a cautionary tale.

## The Two Architectures

Every system has two architectures that most teams confuse:

### Logical Architecture

**What your system does** - the software components and their interactions.

```sruja
// partial
// This is LOGICAL architecture
ECommerce = system "E-Commerce Platform" {
  API = container "REST API" {
    technology "Rust"
  }
  
  WebApp = container "Web Application" {
    technology "React"
  }
  
  Database = database "PostgreSQL" {
    technology "PostgreSQL"
  }
  
  Cache = database "Redis Cache" {
    technology "Redis"
  }
}
```

This shows:
- What services exist
- How they communicate
- What technologies they use

**Audience:** Architects, developers, product managers

### Physical Architecture

**Where your system runs** - the infrastructure and deployment topology.

```sruja
// partial
// This is PHYSICAL architecture
deployment Production "Production Environment" {
  node AWS "AWS Cloud" {
    node USEast1 "US-East-1 Region" {
      node EKS "Kubernetes Cluster" {
        containerInstance ECommerce.API {
          replicas 10
          cpu "2 cores"
          memory "4GB"
        }
      }
      
      node RDS "RDS PostgreSQL" {
        containerInstance ECommerce.Database {
          instance "db.r5.xlarge"
          multi_az true
        }
      }
    }
  }
}
```

This shows:
- Where code runs
- Infrastructure configuration
- Scaling parameters
- Geographic distribution

**Audience:** DevOps, SRE, platform engineers

### Why the Separation Matters

**Story:** A startup I worked with had beautiful logical architecture diagrams. Microservices, event-driven, clean boundaries. But their deployment? Everything ran on one EC2 instance. When that instance failed, all their "resilient microservices" went down together.

**The lesson:** Logical resilience means nothing without physical separation.

**When to model separately:**

- Planning migrations (EC2 → EKS, on-prem → cloud)
- Multi-region deployments
- Disaster recovery planning
- Cost optimization
- Compliance requirements

## Deployment Strategies: When to Use What

Let me walk you through the real-world trade-offs of each strategy.

### On-Premises: When Control Trumps Convenience

**What it is:** Running on your own hardware in your own data center.

**Real-world example:** Goldman Sachs

Goldman runs most of their trading systems on-premises. Why? Microsecond latency matters in high-frequency trading. Cloud latency is too unpredictable. Regulatory requirements demand data sovereignty. And when you're moving billions of dollars, the cost of owning hardware is negligible.

**When to choose on-prem:**
- ✅ Regulatory requirements (data must stay in specific location)
- ✅ Extreme latency requirements (< 1ms)
- ✅ Predictable, massive scale (you know you'll use 10,000 servers)
- ✅ Classified/sensitive data (government, defense)

**When to avoid:**
- ❌ Early-stage startups (capital expense too high)
- ❌ Variable traffic (you'll over-provision)
- ❌ Small teams (maintenance burden)
- ❌ Geographic distribution needs

**Cost reality:**
- Initial investment: $500K - $5M (hardware, data center, networking)
- Ongoing: $50K - $500K/month (power, cooling, staff)
- Break-even point: 3-5 years

**The mistake I see:** Companies choose on-prem for "security" when cloud is actually more secure (AWS spends more on security than most companies' entire revenue).

### Cloud: Speed and Flexibility

**What it is:** Renting infrastructure from AWS, GCP, Azure, etc.

**Real-world example:** Airbnb

Airbnb runs almost entirely on AWS. During the 2022 travel surge, they scaled from 5,000 to 25,000 instances in hours. Try doing that with on-prem.

**When to choose cloud:**
- ✅ Early-stage (pay-as-you-go)
- ✅ Variable traffic (scale up/down)
- ✅ Global distribution (deploy anywhere)
- ✅ Small team (managed services)
- ✅ Speed to market

**When to be careful:**
- ⚠️ Predictable, steady workloads (can be cheaper on-prem)
- ⚠️ Extreme compliance (some certifications require physical control)
- ⚠️ Very high bandwidth (cloud egress gets expensive)

**Cost reality:**
- Startup: $500 - $5,000/month
- Mid-size: $20K - $100K/month
- Enterprise: $500K - $5M/month

**The mistake I see:** "Cloud is always cheaper." It's not. Run the numbers for YOUR workload.

### Containers & Kubernetes: The Standard for Scale

**What it is:** Packaging code with dependencies (Docker) and orchestrating at scale (Kubernetes).

**Real-world example:** Spotify

Spotify runs 150+ services on Google Kubernetes Engine (GKE). Before Kubernetes, deployments took hours and scaling was manual. Now: 2-minute deployments, auto-scaling, self-healing.

**When to choose Kubernetes:**
- ✅ 10+ services (orchestration value)
- ✅ Need auto-scaling
- ✅ Multi-cloud strategy
- ✅ Dev teams want self-service deployment

**When to avoid:**
- ❌ < 5 services (overkill)
- ❌ Simple stateless apps (ECS or Cloud Run is easier)
- ❌ Small team (K8s expertise required)
- ❌ Just getting started (add complexity later)

**Cost reality:**
- Control plane: Free (managed) or $150/month (self-managed)
- Worker nodes: $500 - $50,000/month depending on scale
- Hidden cost: Engineering time (steep learning curve)

**The mistake I see:** "We need Kubernetes because Netflix uses it." Netflix has 700 engineers. You have 5. Start simpler.

## Real-World Deployment Patterns

### Pattern 1: Blue/Green Deployment

**What it is:** Run two identical environments (Blue = current, Green = new). Switch traffic instantly.

**Real-world example:** Amazon

Amazon uses Blue/Green for most services. Their deployment philosophy: "If you can't roll back in 30 seconds, you're doing it wrong."

**How it works:**
1. Blue environment is live (100% traffic)
2. Deploy new version to Green environment
3. Run tests on Green
4. Switch 10% traffic to Green
5. Monitor for 15 minutes
6. Gradually increase to 100%
7. Keep Blue warm for instant rollback

**Sruja model:**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce" {
  API = container "API Service" {
    technology "Rust"
  }
}

deployment Production "Production" {
  node Blue "Blue Environment (Active)" {
    status "active"
    containerInstance ECommerce.API {
      replicas 10
      traffic 100
      version "v2.3.1"
    }
  }
  
  node Green "Green Environment (Standby)" {
    status "standby"
    containerInstance ECommerce.API {
      replicas 10
      traffic 0
      version "v2.3.2"  // New version ready
    }
  }
}

view index {
  include *
}
```

**When to use:**
- ✅ Zero-downtime requirement
- ✅ Critical services (payments, auth)
- ✅ Need instant rollback
- ✅ Complex deployments (db migrations + code)

**When to avoid:**
- ❌ Resource-constrained (doubles infrastructure cost)
- ❌ Simple apps (rolling update is fine)
- ❌ Stateless, stateless services (no migration needed)

**Cost:** 2x infrastructure (two full environments)

### Pattern 2: Canary Deployment

**What it is:** Gradually shift traffic to new version while monitoring for issues.

**Real-world example:** Netflix

Netflix's deployment philosophy: "Deploy to 1%, watch for 30 minutes. If good, deploy to 5%, watch. Continue until 100%."

**How it works:**
1. Deploy new version alongside old
2. Route 1% traffic to new version
3. Monitor error rates, latency, business metrics
4. If good → increase to 5%, then 10%, then 25%, then 100%
5. If bad → automatic rollback

**Sruja model:**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce" {
  API = container "API Service" {
    technology "Rust"
  }
}

deployment Production "Production" {
  node Stable "Stable Version" {
    containerInstance ECommerce.API {
      replicas 20
      traffic 95  // 95% of traffic
      version "v2.3.1"
    }
  }
  
  node Canary "Canary Version" {
    containerInstance ECommerce.API {
      replicas 1
      traffic 5  // 5% of traffic
      version "v2.3.2"
      
      auto_rollback {
        enabled true
        error_rate "> 1%"
        latency_p95 "> 500ms"
        trigger_time "5 minutes"
      }
    }
  }
}

view index {
  include *
}
```

**When to use:**
- ✅ Large user base (1% = statistically significant)
- ✅ Can tolerate some users hitting issues
- ✅ Want early warning before full rollout
- ✅ Continuous deployment (ship daily)

**When to avoid:**
- ❌ Small user base (1% = 1 user)
- ❌ Zero-tolerance for errors (B2B, healthcare)
- ❌ Simple, well-tested changes

**Cost:** Minimal (canary is usually small % of capacity)

### Pattern 3: Rolling Deployment

**What it is:** Gradually replace old instances with new ones.

**Real-world example:** Uber

Uber deploys 1,000+ times per day using rolling deployments. Each service has multiple instances. Update one at a time, keeping enough capacity.

**How it works:**
1. Service has 10 instances running
2. Terminate 1 instance
3. Start 1 new instance
4. Wait for health check
5. Repeat until all updated

**Sruja model:**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce" {
  API = container "API Service" {
    technology "Rust"
  }
}

deployment Production "Production" {
  node Cluster "Kubernetes Cluster" {
    containerInstance ECommerce.API {
      replicas 10
      version "v2.3.2"
      
      rolling_update {
        max_unavailable 1  // Only 1 down at a time
        max_surge 1  // Can create 1 extra during update
      }
    }
  }
}

view index {
  include *
}
```

**When to use:**
- ✅ Stateless services
- ✅ Resource-efficient (no extra capacity)
- ✅ Quick deployments
- ✅ Multiple replicas (3+)

**When to avoid:**
- ❌ Single replica (downtime during update)
- ❌ Stateful services (session draining issues)
- ❌ Complex migrations (need Blue/Green)

**Cost:** Minimal (uses existing capacity)

### Decision Framework: Which Pattern?

**Ask these questions:**

**1. Can you tolerate any downtime?**
- No → Blue/Green or Canary
- Yes → Rolling is fine

**2. How many replicas?**
- 1 → Blue/Green (can't do rolling)
- 2-3 → Canary or Rolling
- 5+ → Any pattern works

**3. What's your budget?**
- Tight → Rolling (free)
- Normal → Canary (minimal extra)
- Generous → Blue/Green (2x cost)

**4. How critical is the service?**
- Critical (payments, auth) → Blue/Green
- Important → Canary
- Normal → Rolling

**5. What's your traffic volume?**
- High (10k+ req/s) → Canary
- Medium → Any
- Low → Rolling

**Quick decision guide:**

```
┌─ Can tolerate downtime?
│  ├─ No → Blue/Green
│  └─ Yes
│     ├─ Multiple replicas?
│     │  ├─ Yes → Rolling
│     │  └─ No → Blue/Green
│     └─ Single replica → Blue/Green
│
└─ High traffic + continuous deploy? → Canary
```

## Multi-Region & Disaster Recovery

### Pattern: Active-Active Multi-Region

**Real-world example:** Netflix

Netflix runs active-active across three AWS regions (US-East, US-West, EU). Each region handles traffic. If one fails, others absorb.

**Sruja model:**

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Netflix = system "Netflix Platform" {
  API = container "Streaming API"
}

deployment Global "Global Deployment" {
  node AWS "AWS Global" {
    node USEast "US-East-1" {
      status "active"
      traffic 50  // 50% of global traffic
      
      containerInstance Netflix.API {
        replicas 100
        region "us-east-1"
      }
    }
    
    node USWest "US-West-2" {
      status "active"
      traffic 30  // 30% of global traffic
      
      containerInstance Netflix.API {
        replicas 60
        region "us-west-2"
      }
    }
    
    node EU "EU-West-1" {
      status "active"
      traffic 20  // 20% of global traffic
      
      containerInstance Netflix.API {
        replicas 40
        region="eu-west-1"
      }
    }
  }
}

view index {
  include *
}
```

**Cost:** 3x infrastructure (but you're paying for capacity you use)

**When to use:**
- ✅ Global user base
- ✅ 99.99%+ availability requirement
- ✅ Latency matters (users need local region)
- ✅ Budget allows

### Pattern: Active-Passive (Failover)

**Real-world example:** Most SaaS companies

Run primary region active. Secondary region on standby (minimal capacity). Failover when primary fails.

**Cost:** ~1.2x infrastructure (secondary runs minimal)

**When to use:**
- ✅ Regional user base
- ✅ Can tolerate 5-15 minute outage
- ✅ Budget-conscious

## CI/CD: Making Deployment Boring

The best deployment is a boring deployment. Routine. Uneventful.

**Real-world example:** Etsy

Etsy deploys 50+ times per day. Their deployment process is so reliable it's boring. That's the goal.

### Modeling Your Pipeline

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

CICD = system "CI/CD Pipeline" {
  GitHub = container "GitHub" {
    description "Code repository, triggers pipeline on push"
  }
  
  Build = container "Build Service" {
    technology "GitHub Actions"
    description "Builds Docker images, runs unit tests"
  }
  
  Test = container "Test Runner" {
    description "Integration tests, E2E tests"
  }
  
  Staging = container "Staging Deploy" {
    description "Deploys to staging environment"
  }
  
  Production = container "Production Deploy" {
    technology "ArgoCD"
    description "GitOps deployment to production"
  }
  
  // Pipeline flow
  GitHub -> Build "Push triggers build"
  Build -> Test "If build succeeds"
  Test -> Staging "If tests pass"
  Staging -> Production "After manual approval"
}

ECommerce = system "E-Commerce Platform" {
  API = container "API Service"
}

// Link CI/CD to your services
CICD.Production -> ECommerce.API "Deploys"

view index {
  include *
}
```

**Best practices:**
1. **Automate everything** - Manual steps cause errors
2. **Fast feedback** - Developers should know in < 10 minutes
3. **Immutable artifacts** - Same artifact through all environments
4. **Rollback automation** - One button, instant rollback
5. **Observability** - Every deploy tracked, monitored

## Service Level Objectives (SLOs)

**Real-world example:** Google

Google popularized SLOs. Every service has defined reliability targets. If you're within SLO, you can deploy. If not, freeze.

### Modeling SLOs

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce Platform" {
  API = container "API Service" {
    technology "Rust"
    
    slo {
      availability {
        target "99.9%"  // 8.76 hours downtime/year
        window "30 days"
        current "99.95%"
      }
      
      latency {
        p95 "200ms"
        p99 "500ms"
        window "7 days"
        current {
          p95 "180ms"
          p99 "420ms"
        }
      }
      
      error_rate {
        target "< 0.1%"
        window "30 days"
        current "0.05%"
      }
    }
  }
  
  Database = database "PostgreSQL" {
    technology "PostgreSQL"
    
    slo {
      availability {
        target "99.99%"  // 52 minutes downtime/year
        window "365 days"
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

**Why model SLOs:**
- Clear expectations (what does "reliable" mean?)
- Deployment gates (only deploy if SLO allows)
- Stakeholder communication (SLAs become commitments)
- Living documentation (SLOs evolve with architecture)

## Observability: The Three Pillars

**Real-world example:** Stripe

Stripe's observability is legendary. They can diagnose almost any issue in minutes because they have complete visibility.

### The Three Pillars

**1. Metrics (Prometheus, Datadog)**
- What's happening? (counts, rates, percentiles)
- Example: "API latency p95 is 200ms"

**2. Logs (ELK, Splunk)**
- What happened? (events, errors, debug info)
- Example: "Payment failed: card declined"

**3. Traces (Jaeger, Zipkin)**
- Where did it happen? (request flow across services)
- Example: "Request took 300ms: 150ms in DB, 100ms in API, 50ms in network"

### Modeling Observability

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

Observability = system "Observability Stack" {
  Metrics = container "Prometheus" {
    description "Time-series metrics from all services"
  }
  
  Dashboards = container "Grafana" {
    description "Visualize metrics and SLOs"
  }
  
  Logs = container "ELK Stack" {
    description "Centralized logging"
  }
  
  Traces = container "Jaeger" {
    description "Distributed tracing"
  }
  
  Alerts = container "PagerDuty" {
    description "Alert routing and on-call"
  }
}

ECommerce = system "E-Commerce Platform" {
  API = container "API Service" {
    description "Instrumented with metrics, logs, and traces"
  }
}

// Observability relationships
ECommerce.API -> Observability.Metrics "Exposes metrics on /metrics"
ECommerce.API -> Observability.Logs "Sends logs via Fluentd"
ECommerce.API -> Observability.Traces "Sends spans via Jaeger client"
Observability.Metrics -> Observability.Dashboards "Feeds dashboards"
Observability.Metrics -> Observability.Alerts "Triggers alerts"

view index {
  include *
}
```

## Common Deployment Mistakes

### Mistake #1: Deploying on Friday

**What happens:** You deploy at 5 PM Friday. Something breaks. Now you're debugging while everyone else is at happy hour.

**Why it fails:**
- Less support available
- Tired team
- Ruined weekend
- Desperate decisions

**The fix:** Deploy Tuesday-Thursday, morning only. Leave Friday for emergencies only.

### Mistake #2: No Rollback Plan

**What happens:** Deployment fails. You have no way to revert. You're fixing forward under pressure.

**Why it fails:**
- Fixing forward takes longer
- Mistakes under pressure
- Extended outage

**The fix:** Every deployment has a tested rollback procedure. Blue/Green makes this easy.

### Mistake #3: Database Migrations in the Deployment

**What happens:** You deploy code AND migrate database in one step. Migration locks table. Everything hangs.

**Why it fails:**
- Can't rollback easily
- Locks cause timeouts
- Tight coupling

**The fix:**
1. Migrate database separately (backward compatible)
2. Deploy code (works with old and new schema)
3. Verify
4. Remove backward compatibility

### Mistake #4: Deploying All Services at Once

**What happens:** You deploy 10 services simultaneously. Something breaks. Which service caused it?

**Why it fails:**
- Hard to isolate issues
- Blast radius maximized
- Debugging nightmare

**The fix:** Deploy one service at a time. Monitor. Repeat.

### Mistake #5: Insufficient Capacity for Deployment

**What happens:** Rolling deployment starts. Old instances terminate. New instances not ready. Traffic spikes. Cascading failure.

**Why it fails:**
- Running at capacity limit
- No buffer for deployment
- Resource exhaustion

**The fix:** Always have 30-50% headroom. Scale up before deploying.

### Mistake #6: No Observability During Deployment

**What happens:** You deploy. Something breaks. But you don't know because alerts aren't configured.

**Why it fails:**
- Blind deployment
- Late detection
- Longer MTTR

**The fix:** Every deployment has dashboard open, alerts verified, team watching.

## Deployment Checklist

Before every production deployment:

**Pre-Deployment:**
- [ ] Code reviewed and approved
- [ ] Tests passing (unit, integration, E2E)
- [ ] Deployed to staging and verified
- [ ] Rollback procedure documented and tested
- [ ] Capacity verified (30%+ headroom)
- [ ] Observability dashboards open
- [ ] Team notified (Slack, email)
- [ ] Not Friday afternoon

**During Deployment:**
- [ ] Deploy to canary/staging first
- [ ] Monitor metrics (latency, errors, throughput)
- [ ] Check business metrics (signups, orders)
- [ ] Verify health checks passing
- [ ] Review logs for errors
- [ ] Gradually increase traffic

**Post-Deployment:**
- [ ] Verify all services healthy
- [ ] Check SLOs are met
- [ ] Monitor for 30-60 minutes
- [ ] Update changelog
- [ ] Close deployment ticket
- [ ] Celebrate (small wins matter)

**If Something Goes Wrong:**
- [ ] Don't panic
- [ ] Rollback immediately (don't try to fix forward first)
- [ ] Communicate to stakeholders
- [ ] Document what happened
- [ ] Post-mortem within 48 hours

## Complete Example: E-Commerce at Scale

Let me show you a complete deployment architecture for a growing e-commerce platform:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// Logical Architecture
ECommerce = system "E-Commerce Platform" {
  WebApp = container "Web Application" {
    technology "React"
    description "Customer-facing storefront"
  }
  
  API = container "API Service" {
    technology "Rust"
    description "Core business logic"
  }
  
  Database = database "PostgreSQL" {
    technology "PostgreSQL"
    description "Primary data store"
  }
  
  Cache = database "Redis" {
    technology "Redis"
    description "Session and query cache"
  }
}

// CI/CD Pipeline
CICD = system "CI/CD Pipeline" {
  GitHub = container "GitHub"
  Build = container "Build Service"
  Deploy = container "Deploy Service"
  
  GitHub -> Build "Push triggers build"
  Build -> Deploy "Deploy if tests pass"
}

// Observability Stack
Observability = system "Observability" {
  Metrics = container "Prometheus"
  Logs = container "ELK Stack"
  Traces = container "Jaeger"
}

// Production Deployment
deployment Production "Production Environment" {
  node AWS "AWS Cloud" {
    // Primary Region
    node USEast1 "US-East-1 (Primary)" {
      node EKS "EKS Cluster" {
        containerInstance ECommerce.API {
          replicas 10
          min_replicas 5
          max_replicas 50
          
          deployment_strategy "canary"
          canary_percentage 5
          
          slo {
            availability {
              target "99.9%"
            }
            latency {
              p95 "200ms"
              p99 "500ms"
            }
          }
        }
        
        containerInstance ECommerce.WebApp {
          replicas 5
          cdn "CloudFront"
        }
      }
      
      node RDS "RDS PostgreSQL" {
        containerInstance ECommerce.Database {
          instance "db.r5.xlarge"
          multi_az true
          backup_retention "7 days"
        }
      }
      
      node ElastiCache "ElastiCache Redis" {
        containerInstance ECommerce.Cache {
          node_type "cache.r5.large"
          replicas 2
        }
      }
    }
    
    // DR Region
    node USWest2 "US-West-2 (DR)" {
      status "standby"
      
      node EKS "EKS Cluster" {
        containerInstance ECommerce.API {
          replicas 2
          traffic 0  // Standby
        }
      }
      
      node RDS "RDS Read Replica" {
        containerInstance ECommerce.Database {
          role "read-replica"
        }
      }
    }
  }
}

// Link observability
ECommerce.API -> Observability.Metrics "Exposes metrics"
ECommerce.API -> Observability.Logs "Sends logs"
ECommerce.API -> Observability.Traces "Sends traces"

view index {
  include *
}
```

## What to Remember

1. **Logical ≠ Physical** - Model what (services) separately from where (infrastructure)

2. **Deployment strategy matters** - Blue/Green for critical, Canary for scale, Rolling for efficiency

3. **Never deploy without rollback** - If you can't revert in 30 seconds, you're not ready

4. **Observe everything** - Metrics, logs, traces for every service

5. **SLOs define reliability** - Clear targets, measured continuously

6. **Automate deployment** - Manual steps cause errors

7. **Deploy early in the week** - Tuesday-Thursday morning, never Friday

8. **Test deployment procedures** - Rollback isn't real until you've tested it

9. **Capacity matters** - Always have 30-50% headroom

10. **Make deployment boring** - The best deployment is uneventful

## When to Start Modeling Deployment

You don't need deployment models on day one. Here's when to start:

**Phase 1: Prototype (Skip deployment modeling)**
- Focus on logical architecture
- Deploy manually
- Learn what works

**Phase 2: MVP (Start documenting)**
- Basic deployment diagram
- Document where things run
- Simple CI/CD

**Phase 3: Production (Model thoroughly)**
- Full deployment architecture
- SLOs defined
- Multiple regions
- Disaster recovery

**Phase 4: Scale (Live in deployment models)**
- Multi-region active-active
- Chaos engineering
- Advanced deployment patterns

## Practical Exercise

Design deployment architecture for a real or hypothetical system:

**Step 1: Choose Your System**
- Something you work on, or
- Hypothetical: "SaaS platform, 100K users, US + EU"

**Step 2: Choose Deployment Strategy**
- Based on requirements and constraints
- Justify your choice

**Step 3: Model Logical Architecture**
- Services, databases, caches
- Technology choices

**Step 4: Model Physical Architecture**
- Cloud provider(s)
- Regions
- Instance types and counts

**Step 5: Add Observability**
- Metrics, logs, traces
- SLOs for critical services

**Step 6: Define CI/CD Pipeline**
- Build, test, deploy stages
- Rollback procedures

**Time:** 30-45 minutes

---

**Next up:** Lesson 3 explores observability and monitoring in depth - how to see what's happening in your production systems.
