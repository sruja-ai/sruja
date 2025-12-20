---
title: "Lesson 2: Interview Question - Design a High-Performance Payment System"
weight: 2
summary: "Answer SLO and performance questions with confidence."
---

# Lesson 2: Interview Question - Design a High-Performance Payment System

## The Interview Question

**"Design a payment processing system that can handle 1 million transactions per second with 99.99% availability and < 100ms latency."**

This question tests your understanding of:
- Performance requirements (SLOs)
- High availability
- Low latency systems
- Trade-offs between consistency and performance

## Step 1: Clarify Requirements

**You should ask:**
- "What's the transaction volume? Peak vs average?"
- "What's the availability requirement? 99.9% or 99.99%?"
- "What's the latency requirement? P95 or P99?"
- "What about consistency? Do we need strong consistency?"

**Interviewer's answer:**
- "1M transactions/second at peak"
- "99.99% availability (four nines)"
- "< 100ms p95 latency"
- "Strong consistency required (it's money!)"

## Step 2: Design with SLOs in Mind

This is where SLOs (Service Level Objectives) come in. **Interviewers love when you think about measurable targets.**

Let's model the payment system with explicit SLOs:

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  PaymentService = system "Payment Processing" {
    PaymentAPI = container "Payment API" {
      technology "Go, gRPC"
      
      // This shows production-ready thinking!
      slo {
        availability {
          target "99.99%"
          window "30 days"
          current "99.97%"
        }
        
        latency {
          p95 "100ms"
          p99 "250ms"
          window "7 days"
          current {
            p95 "85ms"
            p99 "200ms"
          }
        }
        
        errorRate {
          target "< 0.01%"
          window "30 days"
          current "0.008%"
        }
        
        throughput {
          target "1000000 txn/s"
          window "1 hour"
          current "950000 txn/s"
        }
      }
      
      scale {
        min 100
        max 10000
        metric "cpu > 70% or requests_per_second > 500000"
      }
    }
    
    FraudDetection = container "Fraud Detection" {
      technology "Python, ML"
      description "Real-time fraud detection"
    }
    
    PaymentDB = datastore "Payment Database" {
      technology "PostgreSQL"
      description "Primary database with 10 read replicas"
    }
    
    Cache = datastore "Payment Cache" {
      technology "Redis"
      description "Caches recent transactions"
    }
    
    PaymentQueue = queue "Payment Queue" {
      technology "Kafka"
      description "Async payment processing"
    }
  }
  
  Stripe = system "Stripe Gateway" {
    tags ["external"]
  }
  
  BankAPI = system "Bank API" {
    tags ["external"]
  }
  
  PaymentService.PaymentAPI -> PaymentService.FraudDetection "Validates"
  PaymentService.PaymentAPI -> PaymentService.Cache "Checks recent transactions"
  PaymentService.PaymentAPI -> PaymentService.PaymentDB "Stores transaction"
  PaymentService.PaymentAPI -> PaymentService.PaymentQueue "Enqueues for async processing"
  PaymentService.PaymentAPI -> Stripe "Processes payment"
  PaymentService.PaymentAPI -> BankAPI "Validates with bank"
}

views {
  view index {
    include *
  }
}
```

## What Interviewers Look For

### ✅ Good Answer (What You Just Did)

1. **Defined SLOs explicitly** - Shows you think about measurable targets
2. **Addressed all requirements** - Availability, latency, throughput
3. **Explained trade-offs** - Strong consistency vs performance
4. **Scalability** - Showed how to handle 1M txn/s
5. **Redundancy** - Multiple replicas, failover strategies

### ❌ Bad Answer (Common Mistakes)

1. Not defining SLOs or performance targets
2. Ignoring availability requirements
3. Not explaining how to achieve 99.99% availability
4. Not addressing consistency requirements
5. No capacity estimation

## Key Points to Mention in Interview

### 1. Availability (99.99% = Four Nines)

**Say**: "99.99% availability means 52.6 minutes of downtime per year. To achieve this, we need:
- Multiple data centers (active-active)
- Automatic failover
- Health checks and monitoring
- Database replication with automatic promotion"

### 2. Latency (< 100ms p95)

**Say**: "To achieve < 100ms latency, we:
- Use in-memory cache (Redis) for hot data
- Keep database queries simple and indexed
- Use connection pooling
- Minimize network hops
- Consider async processing for non-critical paths"

### 3. Throughput (1M txn/s)

**Say**: "To handle 1M transactions/second:
- Horizontal scaling: 100-10,000 API instances
- Database sharding by transaction ID
- Read replicas for scaling reads
- Caching frequently accessed data
- Async processing for non-critical operations"

### 4. Strong Consistency

**Say**: "Since this is financial data, we need strong consistency:
- All writes go to primary database
- Read replicas are eventually consistent (ok for reads)
- Use distributed transactions for critical operations
- Trade-off: Slightly higher latency for correctness"

## Understanding SLO Types (Interview Context)

### Availability SLO

**Interviewer asks**: "How do you ensure 99.99% availability?"

**Your answer with SLO**:
```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  PaymentService = system "Payment Processing" {
    PaymentAPI = container "Payment API" {
      slo {
        availability {
          target "99.99%"
          window "30 days"
          current "99.97%"
        }
      }
    }
  }
}

views {
  view index {
    include *
  }
}
```

**Explain**: "We target 99.99% (four nines), which allows 52.6 minutes downtime per year. Currently at 99.97%, so we're close but need to improve redundancy."

### Latency SLO

**Interviewer asks**: "How fast should payments process?"

**Your answer with SLO**:
```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  PaymentService = system "Payment Processing" {
    PaymentAPI = container "Payment API" {
      slo {
        latency {
          p95 "100ms"
          p99 "250ms"
          window "7 days"
        }
      }
    }
  }
}

views {
  view index {
    include *
  }
}
```

**Explain**: "95% of payments complete in under 100ms, 99% in under 250ms. We use p95/p99 instead of average because they show real user experience - a few slow payments don't skew the metric."

### Error Rate SLO

**Interviewer asks**: "What error rate is acceptable?"

**Your answer with SLO**:
```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  PaymentService = system "Payment Processing" {
    PaymentAPI = container "Payment API" {
      slo {
        errorRate {
          target "< 0.01%"
          window "30 days"
          current "0.008%"
        }
      }
    }
  }
}

views {
  view index {
    include *
  }
}
```

**Explain**: "We target less than 0.01% error rate. Currently at 0.008%, which is good, but we monitor closely because payment errors are critical."

## Real Interview Example: Capacity Estimation

**Interviewer**: "How many servers do you need for 1M txn/s?"

**Your answer**:
1. "Each transaction requires ~10ms processing = 100 transactions/second per server"
2. "1M txn/s ÷ 100 = 10,000 servers needed"
3. "With 2x headroom for spikes and redundancy: ~20,000 servers"
4. "But we can optimize:
   - Caching reduces DB load → fewer DB servers
   - Async processing → can batch operations
   - Database sharding → distributes load
   - Final estimate: ~5,000-10,000 servers"

## Interview Practice: Add High Availability

**Interviewer**: "How do you ensure 99.99% availability?"

Add redundancy to your design:

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  PaymentService = system "Payment Processing" {
    PaymentAPI = container "Payment API" {
      technology "Go, gRPC"
      scale {
        min 100
        max 10000
        metric "cpu > 70%"
      }
      description "Deployed across 3 data centers (active-active)"
    }
    
    PaymentDB = datastore "Payment Database" {
      technology "PostgreSQL"
      description "Primary in US-East, replicas in US-West and EU"
    }
  }
  
  // Show redundancy
  PaymentService.PaymentAPI -> PaymentService.PaymentDB "Writes to primary"
}

views {
  view index {
    include *
  }
}
```

**Explain**: "We deploy across 3 data centers in active-active mode. If one fails, traffic automatically routes to others. Database has primary + replicas with automatic failover."

## Common Follow-Up Questions

Be prepared for:

1. **"What if the database fails?"**
   - Answer: "Automatic failover to replica, data replication with < 1s lag"

2. **"How do you handle network partitions?"**
   - Answer: "CAP theorem - we choose consistency over availability for payments. If partition occurs, we reject transactions rather than risk inconsistency."

3. **"What about data consistency across regions?"**
   - Answer: "Synchronous replication for critical data, eventual consistency for non-critical. Use distributed transactions for cross-region operations."

4. **"How do you monitor SLOs?"**
   - Answer: "Real-time dashboards showing current vs target SLOs. Alerts when we're at risk of violating SLOs. Weekly reviews of SLO performance."

## Exercise: Practice This Question

Design a payment system and be ready to explain:
1. How you achieve 99.99% availability
2. How you keep latency < 100ms
3. How you handle 1M txn/s
4. Your SLO targets and how you measure them

**Practice tip**: Time yourself (40-45 minutes) and explain out loud. Focus on SLOs - interviewers love this!

## Key Takeaways for Interviews

1. **Always define SLOs** - Shows production-ready thinking
2. **Explain trade-offs** - Availability vs consistency, latency vs throughput
3. **Show capacity estimation** - Back up your numbers
4. **Mention monitoring** - How you track SLOs
5. **Discuss failure scenarios** - What happens when things break

## Next Steps

You've learned how to handle performance and SLO questions. In the next module, we'll tackle modular architecture questions - another common interview topic!
