title: "Lesson 1: Load Balancers"
weight: 1
summary: "L4 vs L7 Load Balancing, Algorithms, Real-World Case Studies"
learning_objectives:
  - Understand the difference between L4 and L7 load balancing
  - Learn load balancing algorithms and when to use each
  - See real-world examples from NGINX and HAProxy
  - Practice making trade-off decisions
estimated_time: "20 minutes"
difficulty: "beginner"
---

# Lesson 1: Load Balancers

## What is a Load Balancer?

A load balancer sits between clients and servers, distributing incoming network traffic across a group of backend servers. This ensures that no single server bears too much load.

**Without a load balancer:**
```
Clients → Server 1 (overwhelmed, 99% CPU)
         → Server 2 (idle, 5% CPU)
         → Server 3 (idle, 3% CPU)
```

**With a load balancer:**
```
Clients → Load Balancer → Server 1 (33% CPU)
                              → Server 2 (33% CPU)
                              → Server 3 (34% CPU)
```

## Types of Load Balancing

### Layer 4 (Transport Layer)

**What it does:** Makes decisions based on IP address and TCP/UDP ports.

**Characteristics:**
- ✅ **Fast** - No content inspection, just reads packet headers
- ✅ **Low CPU** - Simple routing logic
- ❌ **Limited intelligence** - Can't inspect HTTP headers, URLs, or cookies
- ✅ **Protocol-agnostic** - Works for HTTP, TCP, UDP, gRPC

**Best for:** High-throughput services where you need raw speed over intelligence.

**Examples:** AWS ELB (classic), HAProxy (L4 mode), F5 LTM

### Layer 7 (Application Layer)

**What it does:** Makes decisions based on the content of the message (URL, HTTP headers, cookies, cookies).

**Characteristics:**
- ❌ **Slower** - Needs to parse HTTP content
- ❌ **CPU intensive** - More complex routing logic
- ✅ **Smart routing** - Can route /images to image servers, /api to API servers
- ✅ **Content-aware** - Inspects HTTP headers, cookies, SSL termination

**Best for:** Web applications needing content-based routing, SSL termination, or session stickiness.

**Examples:** NGINX, AWS ALB, Envoy, Traefik

### Real-World Comparison

| Factor | Layer 4 | Layer 7 |
|--------|---------|---------|
| **Speed** | 100K+ RPS per core | 50K+ RPS per core |
| **CPU Usage** | 5-10% | 30-50% |
| **Features** | Basic routing | SSL termination, URL routing, session affinity |
| **Use Case** | High-throughput APIs | Web apps, microservices |

## Load Balancing Algorithms

### Round Robin

**How it works:** Requests are distributed sequentially across servers: 1, 2, 3, 4, then back to 1.

**Pros:**
- ✅ Simple to implement and understand
- ✅ Works well when servers have similar capacity
- ✅ No state to maintain

**Cons:**
- ❌ Doesn't account for server load (one might be slow with 100 connections, another idle)
- ❌ Doesn't work well when servers have different capabilities

**Real-world use:** Good for stateless services where all servers are identical (e.g., API servers).

### Least Connections

**How it works:** Sends request to the server with the fewest active connections.

**Pros:**
- ✅ Better than round robin when servers have different loads
- ✅ Accounts for connection time (long-running connections get fewer new ones)

**Cons:**
- ❌ Still doesn't account for server capacity (CPU, memory)
- ❌ Requires tracking active connections (more state)

**Real-world use:** Good for services with varying request processing times (e.g., database queries).

### IP Hash

**How it works:** Uses a hash of the client's IP address to determine which server receives the request.

**Pros:**
- ✅ **Session stickiness** - Same IP always goes to same server
- ✅ Works great for stateful applications (session stored in memory)

**Cons:**
- ❌ Uneven distribution if many clients share the same IP (NAT, proxies)
- ❌ Doesn't rebalance when servers are added/removed

**Real-world use:** Stateful applications needing session affinity, WebSocket connections.

### Random

**How it works:** Randomly selects a server for each request.

**Pros:**
- ✅ No state to maintain
- ✅ Even distribution over time (law of large numbers)

**Cons:**
- ❌ Can have temporary uneven distribution
- ❌ No session stickiness

**Real-world use:** Simple load balancing with large number of requests where stickiness doesn't matter.

## Real-World Case Studies

### Case Study 1: NGINX at Scale

**The Challenge:** NGINX (the company) needed to serve 1M+ requests per second (RPS) with low latency.

**The Architecture:**
```
Clients → Layer 4 Load Balancer (HAProxy) → NGINX Layer 7 LBs → Application Servers
         (10 Gbps)                          (100+ instances)         (1000+ servers)
```

**Key Decisions:**
1. **Layer 4 at edge** - Raw speed for SSL termination and initial routing
2. **Layer 7 closer to apps** - Content-based routing (/static vs /api)
3. **Least connections algorithm** - Servers have varying loads

**The Results:**
- **Throughput:** 1M+ RPS per load balancer
- **Latency:** <10ms at p95
- **Server utilization:** Balanced at 65-75% CPU (good headroom)
- **Availability:** 99.99% with automatic failover

> **💡 Key Insight:** NGINX uses a **layered approach**. Layer 4 for raw speed at the edge, Layer 7 for intelligence closer to the applications. This gives them both performance and features.

### Case Study 2: HAProxy Configuration Patterns

**Scenario:** E-commerce platform with 3 types of traffic:
- Product catalog (read-heavy, 90% of traffic)
- Checkout (write-heavy, requires session stickiness)
- API (stateless, needs high throughput)

**The Solution:**
```
# Product Catalog - Round Robin (read-heavy)
backend catalog
    balance roundrobin
    server web1 10.0.0.1:80 check
    server web2 10.0.0.2:80 check
    server web3 10.0.0.3:80 check

# Checkout - IP Hash (session stickiness)
backend checkout
    balance source
    server app1 10.0.0.10:8080 check
    server app2 10.0.0.11:8080 check

# API - Least Connections (varying request times)
backend api
    balance leastconn
    server api1 10.0.0.20:443 check
    server api2 10.0.0.21:443 check
    server api3 10.0.0.22:443 check
```

**Why this works:**
- **Catalog:** Round robin works great for stateless, uniform requests
- **Checkout:** IP hash ensures user stays on same server (session in memory)
- **API:** Least connections handles varying query complexity (some API calls are fast, others slow)

**Performance Results:**
- **Catalog servers:** 5000 RPS, 2ms latency
- **Checkout servers:** 500 RPS, 20ms latency (acceptable for checkout flow)
- **API servers:** 10000 RPS, 5ms latency (balanced load)

### Case Study 3: WhatsApp's Load Balancing Evolution

**Stage 1 (Early days):** Single server → **Crashed** at 10K users

**Stage 2:** Round robin across 10 servers → **Uneven load**, some servers overwhelmed

**Stage 3 (Solution):** Erlang's built-in load balancing + consistent hashing
```
Clients → Load Balancer → Erlang Nodes → Message Store
                    (Consistent hashing for key-based routing)
```

**Key Decision:** Use **consistent hashing** so messages from the same user always go to the same node. This reduces cross-node synchronization.

**The Results:**
- **Scale:** From 10K users to 1B+ users
- **Throughput:** 65B+ messages/day
- **Efficiency:** Minimal cross-node data movement (99% of traffic stays local)

## Production Metrics

### Load Balancer Performance

| System | Algorithm | Throughput | Latency (p95) | Servers |
|--------|-----------|------------|----------------|---------|
| **NGINX** | Round Robin | 1M+ RPS | <10ms | 100 |
| **HAProxy** | Least Connections | 500K RPS | <5ms | 50 |
| **AWS ALB** | Round Robin | 100K RPS | <50ms | Managed |
| **Envoy** | Random + Health Checks | 1M+ RPS | <20ms | 200 |

### Resource Utilization

| Resource | Layer 4 Load Balancer | Layer 7 Load Balancer |
|----------|---------------------|---------------------|
| **CPU** | 5-15% | 30-60% |
| **Memory** | 1-2 GB | 4-8 GB |
| **Network** | 10 Gbps | 10 Gbps |
| **Throughput** | 1M+ RPS | 500K RPS |

## Trade-Off Scenarios

### Scenario 1: API Gateway for Microservices

**Context:** Building an API gateway that routes to 50 microservices. Some are fast (profile service), others slow (report generation).

**The Trade-Off:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Layer** | Layer 4 (fast) | Layer 7 (smart) | **Layer 7** - Need URL-based routing (/users → users service) |
| **Algorithm** | Round Robin | Least Connections | **Least Connections** - Services have varying response times |
| **Health Checks** | Basic TCP | HTTP /health | **HTTP /health** - Need to detect slow/failing services, not just offline ones |
| **SSL** | At load balancer | At service | **At load balancer** - Centralized SSL management, cheaper certificates |

**Result:**
- **Pros:** Intelligent routing, good load balancing, centralized SSL
- **Cons:** Higher CPU usage, more complex configuration
- **Performance:** 50K RPS at p95 < 50ms (acceptable for API gateway)

### Scenario 2: Video Streaming Platform

**Context:** Streaming video to 10M concurrent users. Each stream needs sustained bandwidth (2-5 Mbps). Low latency is critical.

**The Trade-Off:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Layer** | Layer 4 (speed) | Layer 7 (features) | **Layer 4** - Raw speed for streaming, no content inspection needed |
| **Algorithm** | Round Robin | IP Hash | **Round Robin** - Streams are independent, no session affinity needed |
| **Geo-distribution** | Single datacenter | Edge locations | **Edge locations** - Reduce latency by serving from closest datacenter |

**Result:**
- **Pros:** Maximum throughput, minimal latency, simple configuration
- **Cons:** No intelligent routing (but not needed for streaming)
- **Performance:** 1M+ concurrent streams, <100ms latency globally

### Scenario 3: Stateful WebSocket Application

**Context:** Real-time chat application where users are connected via WebSockets. User messages and presence data must go to the same server.

**The Trade-Off:**

| Decision | Option A | Option B | What You Choose & Why |
|----------|----------|----------|----------------------|
| **Layer** | Layer 4 | Layer 7 | **Layer 7** - Need to inspect WebSocket upgrade requests |
| **Algorithm** | Round Robin | IP Hash | **IP Hash** - Session stickiness required for WebSocket connections |
| **Failover** | Break connections | Graceful reconnect | **Graceful reconnect** - Clients auto-reconnect on disconnect |
| **Persistence** | In-memory | Redis | **In-memory** - Faster for local data, Redis for cross-server sync |

**Result:**
- **Pros:** Session affinity, real-time performance, good user experience
- **Cons:** Uneven distribution (some servers have more active users), complexity in failover
- **Performance:** 100K concurrent WebSockets, <50ms message delivery

## 🛠️ Sruja Perspective: Modeling Load Balancers

In Sruja, we treat load balancers as critical infrastructure components with clear trade-offs documented.

### Why Model Load Balancers?

Modeling load balancers in your architecture provides:

1. **Capacity Planning:** See how much traffic the LB can handle before bottlenecks
2. **Failure Analysis:** Understand what happens if the LB fails (single point of failure?)
3. **Algorithm Clarity:** Document which algorithm and why
4. **Performance Visibility:** Track RPS, latency, server distribution

### Example: E-Commerce Platform Load Balancing

```sruja
import { * } from 'sruja.ai/stdlib'

ECommerce = system "E-Commerce Platform" {
    description "Multi-tenant e-commerce with intelligent load balancing"
    
    // LAYER 4: Edge load balancer for SSL termination
    EdgeLB = container "Edge Load Balancer" {
        technology "HAProxy"
        description "Layer 4 LB for SSL termination and initial routing"
        tags ["load-balancer", "layer4"]
        
        capacity {
            requests_per_second "1_000_000"
            bandwidth_gbps "10"
        }
    }
    
    // LAYER 7: Application load balancer for content routing
    AppLB = container "Application Load Balancer" {
        technology "NGINX"
        description "Layer 7 LB for URL-based routing and session affinity"
        tags ["load-balancer", "layer7"]
        
        tradeoff {
            decision "Use NGINX (Layer 7) for application routing"
            sacrifice "Raw throughput (L4 would be 2x faster)"
            reason "Need URL-based routing: /catalog → catalog servers, /checkout → checkout servers"
            mitigation "Use L4 edge LB for SSL termination to reduce NGINX load"
        }
        
        slo {
            latency {
                p95 "50ms"
                window "7 days"
            }
            availability {
                target "99.99%"
                window "30 days"
            }
        }
    }
    
    // BACKEND SERVICES
    CatalogServer = container "Catalog Service" {
        technology "Python, Django"
        description "Product catalog (read-heavy)"
        tags ["service"]
        quantity 10
    }
    
    CheckoutServer = container "Checkout Service" {
        technology "Node.js"
        description "Checkout flow (stateful, requires session affinity)"
        tags ["service"]
        quantity 5
    }
    
    APIServer = container "API Service" {
        technology "Go"
        description "Public API (stateless, high throughput)"
        tags ["service"]
        quantity 20
    }
    
    // TRAFFIC FLOW
    EdgeLB -> AppLB "Distributes traffic (Round Robin)"
    AppLB -> CatalogServer "Routes /catalog (Least Connections)"
    AppLB -> CheckoutServer "Routes /checkout (IP Hash for session affinity)"
    AppLB -> APIServer "Routes /api (Least Connections)"
}

view index {
    title "E-Commerce Load Balancing Architecture"
    include *
}

view load-balancing {
    title "Load Balancer Configuration"
    include ECommerce.EdgeLB ECommerce.AppLB
}
```

### Key Trade-Offs Documented

**1. Layer Choice:**
- **Why Layer 4 at edge?** Raw speed for SSL termination
- **Why Layer 7 at app?** Need URL-based routing and session affinity

**2. Algorithm Selection:**
- **Round Robin:** For catalog (stateless, uniform requests)
- **IP Hash:** For checkout (requires session stickiness)
- **Least Connections:** For API (varying query complexity)

**3. Performance vs Features:**
- Sacrifice raw throughput for intelligent routing
- Mitigated by using layered approach (L4 + L7)

## Knowledge Check

<details>
<summary><strong>Q: My app needs to route /images to image servers and /api to API servers. Which load balancing layer should I use?</strong></summary>

**Layer 7 (Application Layer)**

Layer 7 load balancers can inspect HTTP content (URLs, headers) and route based on that. Layer 4 only looks at IP/port and can't do content-based routing.

</details>

<details>
<summary><strong>Q: I'm building a WebSocket chat app. Users connect and stay connected for hours. Which algorithm should I use?</strong></summary>

**IP Hash (or consistent hashing)**

You need session affinity - when a user connects, they should stay on the same server. IP hash ensures the same IP always routes to the same server. Round robin would break the WebSocket connection when requests go to different servers.

</details>

<details>
<summary><strong>Q: I have 10 identical servers handling high-throughput API requests. Speed is the priority. What algorithm?</strong></summary>

**Round Robin**

When servers are identical and stateless, round robin is perfect. It's simple, fast, and gives even distribution. No need for IP hash (stateful) or least connections (servers have similar load).

</details>

## Quiz: Test Your Knowledge

### Q1: What type of load balancing operates at the transport layer and uses IP addresses and ports?

- [ ] Layer 4 (Transport Layer)
- [ ] Layer 7 (Application Layer)
- [ ] Layer 3 (Network Layer)

<details>
<summary>Answer</summary>
**Layer 4 (Transport Layer)** operates at the transport layer and makes decisions based on IP addresses and TCP/UDP ports. Layer 7 works at the application layer and inspects HTTP content.

</details>

### Q2: Which load balancing algorithm is best for applications requiring session stickiness?

- [ ] Round Robin
- [ ] Least Connections
- [ ] IP Hash

<details>
<summary>Answer</summary>
**IP Hash** uses the client's IP address to determine which server receives the request, ensuring the same client always goes to the same server. This is essential for session stickiness in stateful applications like WebSockets or applications with in-memory sessions.

</details>

### Q3: You're building an API gateway routing to microservices with varying response times. Which algorithm should you use?

- [ ] Round Robin
- [ ] Least Connections
- [ ] IP Hash

<details>
<summary>Answer</summary>
**Least Connections** sends requests to the server with the fewest active connections. This is ideal when services have varying request processing times (some API calls are fast, others slow) because it accounts for actual server load rather than just distributing requests evenly.

</details>

### Q4: Which of these is NOT a characteristic of Layer 7 load balancing?

- [ ] Can inspect HTTP headers and URLs
- [ ] Slower than Layer 4 due to content inspection
- [ ] Cannot do SSL termination
- [ ] More CPU intensive than Layer 4

<details>
<summary>Answer</summary>
**Cannot do SSL termination** is NOT correct. Layer 7 load balancers CAN and commonly DO perform SSL termination. They terminate SSL at the load balancer, decrypt traffic, inspect HTTP content, then route it. This is one of their key features.

</details>

### Q5: NGINX uses a layered approach with Layer 4 at the edge and Layer 7 closer to applications. Why?

- [ ] To reduce complexity
- [ ] To balance speed and intelligence
- [ ] To minimize costs
- [ ] To reduce latency

<details>
<summary>Answer</summary>
**To balance speed and intelligence**. Layer 4 provides raw speed for SSL termination and initial routing (high throughput). Layer 7 provides intelligent features like URL-based routing closer to the applications. This layered approach gives you both performance (L4) and features (L7).

</details>

## Next Steps

Now that we understand load balancing, let's learn about databases and how to choose between SQL and NoSQL.
👉 **[Lesson 2: Databases (SQL vs NoSQL, Replication, Sharding)](./lesson-2)**