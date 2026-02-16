<!-- Auto-generated quiz from TOML -->
<!-- Source: lesson-1-quiz.toml -->

**1. In system design, what do we call requirements that describe the features and functionality of a system (what it should do)?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Functional

**Alternative answers:**
- functional requirements

**Explanation:**
Functional requirements define the features and capabilities of the system. Examples: "User can post a tweet," "User can browse products."


</details>

---

**2. In system design, what do we call requirements that describe how the system should perform (constraints like speed, scalability, reliability)?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Non-functional

**Alternative answers:**
- non-functional
- non functional
- NFR
- NFRs

**Explanation:**
Non-functional requirements define the quality attributes and constraints of the system. Examples: "Must handle 100M users," "Response time <200ms."


</details>

---

**3. A banking system must ensure that account balances are always accurate and transactions cannot be lost. Which trade-off would you prioritize?**

- [ ] a) Prioritize availability over consistency (it's better to show wrong data than no data)
- [ ] b) Prioritize development speed over performance (ship a MVP first)
- [ ] c) Prioritize write speed over read speed (logging-focused optimization)
- [ ] d) Prioritize consistency over availability (brief downtime is acceptable, but data must be correct)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    In banking, financial accuracy is critical. You would choose a CP system (Consistent, Partition-tolerant) over AP, accepting brief outages to ensure no incorrect transactions.

  </div>
</div>

---

**4. You're building a real-time chat application like Discord. Users expect messages to appear instantly across all devices. What's the best architecture approach?**

- [ ] a) Use relational database with strong consistency (PostgreSQL) for all message storage
- [ ] b) Use HTTP polling where clients check for new messages every 5 seconds
- [ ] c) Use eventual consistency with 24-hour delay synchronization
- [ ] d) Use WebSockets for real-time push with eventual consistency for message storage

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Real-time chat requires low latency. WebSockets eliminate the overhead of polling (which generates 20K requests/second for 100K users). Eventual consistency is acceptable for message delivery.

  </div>
</div>

---

**5. Netflix experienced a major outage in 2008 when their single datacenter failed. What was their system design solution?**

- [ ] a) Bought a bigger, more expensive datacenter with better hardware (Vertical scaling)
- [ ] b) Hired more operations engineers to manually failover systems
- [ ] c) Built a single, massive monolithic application on dedicated servers
- [ ] d) Moved to cloud infrastructure with microservices and built Chaos Monkey to test resilience

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Netflix adopted horizontal scaling on AWS, microservices architecture, and Chaos Monkey (randomly kills services to test resilience). This transformed their availability from 99.9% to 99.99%+.

  </div>
</div>

---

**6. Healthcare.gov's initial launch in 2013 was a disaster. Which of these was NOT one of their system design mistakes?**

- [ ] a) No load testing before launch
- [ ] b) Tightly coupled architecture with no caching layer
- [ ] c) Single database bottleneck with no sharding
- [ ] d) Using cloud infrastructure instead of dedicated on-premise servers

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Healthcare.gov's actual mistakes were: no load testing, tight coupling, single database bottleneck, and no graceful degradation. Using cloud infrastructure itself wasn't the problem—it was the poor architecture.

  </div>
</div>

---

**7. You're building a product search engine for an e-commerce site handling 10M products. The search feature generates 99% of traffic. What optimization should you prioritize?**

- [ ] a) Optimize for write speed (since products are added frequently)
- [ ] b) Use a single-node relational database for simplicity
- [ ] c) Disable caching to ensure always-fresh search results
- [ ] d) Use read replicas and a specialized search engine like Elasticsearch

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    With a 99% read workload, you optimize for read performance. Read replicas distribute query load, and Elasticsearch provides full-text search capabilities that relational databases lack.

  </div>
</div>

---

**8. Instagram launched with 2 servers and grew to 10M users in one month. What was their key architectural change to handle this growth?**

- [ ] a) Rewrote the entire application in a different programming language
- [ ] b) Bought the biggest available server (vertical scaling)
- [ ] c) Removed all features to reduce complexity
- [ ] d) Implemented database sharding, CDN for images, and async processing for image handling

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Instagram used database sharding (split data across multiple DBs), CDN for global image delivery, and async processing for heavy operations. This scaled them from 2 servers to 500+ servers in 6 months.

  </div>
</div>

---

**9. Which of the following statements best describes the relationship between latency and throughput?**

- [ ] a) Low latency always means high throughput
- [ ] b) High latency always means high throughput
- [ ] c) Latency and throughput are the same thing
- [ ] d) A system can have low latency but low throughput, or high latency but high throughput

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Latency = time for ONE request. Throughput = number of concurrent requests. Example: A highway can have high latency (traffic jam) and high throughput (10 lanes), or low latency (empty road) and low throughput (1 lane).

  </div>
</div>

---

**10. Your boss says "We need to handle infinite users." What's the most appropriate response?**

- [ ] a) Great! I'll immediately implement Kubernetes and distributed sharding
- [ ] b) Impossible! Let's cap users at 1,000 and reject anyone else
- [ ] c) Let's build the system assuming unlimited resources regardless of cost
- [ ] d) Infinite is expensive. Let's define realistic targets for the next 12 months (e.g., 100K users) and design for that

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    The best engineers clarify requirements first. The design for 1K users costs $50/month, while 100M users costs $50K/month. Always define realistic, time-boxed targets.

  </div>
</div>

---

**11. What is the term for the system design principle that means every decision involves sacrificing one quality to gain another (e.g., choosing consistency means sacrificing availability)?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Trade-off

**Alternative answers:**
- trade-off
- tradeoff

**Explanation:**
Trade-offs are fundamental to system design. There are no perfect solutions—every architecture choice involves benefits and costs. "It depends" is the correct answer because it depends on which trade-offs you choose.


</details>

---
