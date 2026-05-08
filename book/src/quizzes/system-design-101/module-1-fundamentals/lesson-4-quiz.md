<!-- Auto-generated quiz from TOML -->
<!-- Source: lesson-4-quiz.toml -->

**1. What term in the CAP theorem means every read receives the most recent write or an error (all nodes see the same data)?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Consistency

**Alternative answers:**
- consistency
- C

**Explanation:**
Consistency ensures all nodes see the same data at the same time. When a write is confirmed, any subsequent read returns that value.


</details>

---

**2. What term in the CAP theorem means every request receives a non-error response, without guaranteeing it contains the most recent write?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Availability

**Alternative answers:**
- availability
- A

**Explanation:**
Availability means the system is always responsive. Even if some nodes are out of sync, the system returns a response (possibly stale data) rather than an error.


</details>

---

**3. What term in the CAP theorem means the system continues to operate despite network failures or message loss between nodes?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Partition Tolerance

**Alternative answers:**
- partition tolerance
- partition-tolerance
- P

**Explanation:**
Partition Tolerance ensures the system works even when network communication between nodes fails. In distributed systems, partitions are inevitable, so P is mandatory.


</details>

---

**4. In a distributed system, you must choose between CP and AP when a network partition occurs. Why?**

- [ ] a) Because P is optional in most systems
- [ ] b) Because you can only implement two of the three guarantees simultaneously
- [ ] c) Because C and A are mutually exclusive by definition
- [ ] d) Because network partitions (P) are inevitable in distributed systems

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    The CAP theorem states you can only have 2 of 3. Since network partitions are inevitable in distributed systems, P is mandatory. You must choose between C (Consistency) or A (Availability) during partitions.

  </div>
</div>

---

**5. A banking system must ensure that account balances are always correct. During a network partition, the system rejects transactions if it can't confirm data consistency. This is what type of system?**

- [ ] a) AP (Available) - better to allow transactions with possibly incorrect balances
- [ ] b) CA (Consistent and Available) - possible in single-node systems only
- [ ] c) P (Partition Tolerance) only - data consistency isn't important
- [ ] d) CP (Consistent and Partition Tolerant)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    CP systems prioritize data correctness. During a partition, the system returns errors rather than allowing inconsistent operations.
Banking uses CP because incorrect balances = financial disasters.

  </div>
</div>

---

**6. A social media feed shows posts from friends. If a partition occurs, users see slightly outdated posts rather than an error page. This is what type of system?**

- [ ] a) CP (Consistent) - reject requests if data isn't perfectly synced
- [ ] b) CA (Consistent and Available) - impossible in distributed systems
- [ ] c) P (Partition Tolerance) only - doesn't describe the full trade-off
- [ ] d) AP (Available and Partition Tolerant)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    AP systems prioritize availability. Users see stale data rather than errors.
Social media uses AP because it's better to show slightly old posts than a broken feed.

  </div>
</div>

---

**7. Which system would prioritize CP (Consistency) over Availability?**

- [ ] a) Instagram photo feed
- [ ] b) YouTube video recommendations
- [ ] c) E-commerce product catalog (non-critical)
- [ ] d) PayPal payment processing

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Payment systems prioritize CP because incorrect transactions are unacceptable.
Instagram, YouTube, and product catalogs can use AP (showing slightly stale content is acceptable).

  </div>
</div>

---

**8. Which system would prioritize AP (Availability) over Consistency?**

- [ ] a) Banking transaction system
- [ ] b) Inventory management for critical medical supplies
- [ ] c) Stock trading platform for high-frequency trading
- [ ] d) Netflix video streaming recommendations

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Netflix recommendations can be slightly stale and still provide value. AP is acceptable.
Banking, medical inventory, and HFT require strong consistency (CP) because errors are unacceptable.

  </div>
</div>

---

**9. What is the difference between Strong Consistency and Eventual Consistency?**

- [ ] a) Strong Consistency is slower, Eventual is always faster
- [ ] b) Strong Consistency allows stale reads, Eventual doesn't
- [ ] c) There's no difference, they're synonyms
- [ ] d) Strong Consistency: all nodes see same data immediately. Eventual: nodes eventually converge if no new writes occur.

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Strong Consistency: Read after write always returns the latest value (e.g., banking).
Eventual Consistency: Reads might return stale data, but eventually all nodes converge (e.g., social media).

  </div>
</div>

---

**10. A user posts a tweet. The tweet immediately appears in their own timeline but takes up to 30 seconds to appear in followers' feeds. What consistency model is this?**

- [ ] a) Strong Consistency (everyone sees the tweet immediately)
- [ ] b) No consistency (data is randomly shown or hidden)
- [ ] c) CP system (rejects posts during partitions)
- [ ] d) Eventual Consistency (followers eventually see the tweet)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Twitter uses Eventual Consistency for timeline delivery. The tweet is written immediately, but propagation to followers happens asynchronously.
This is acceptable because users don't expect perfect real-time synchronization.

  </div>
</div>

---

**11. A database has a replication factor of 3 (3 copies of data). Writes are confirmed after writing to 2 nodes. If one node is down, what happens?**

- [ ] a) Write fails because all 3 nodes must be available
- [ ] b) Write succeeds because only 2 nodes are required (quorum)
- [ ] c) System becomes completely unavailable
- [ ] d) Write succeeds because quorum (2 out of 3 nodes) is available

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    With a quorum-based approach, writes succeed if a majority of nodes acknowledge.
This provides both consistency (majority agreement) and availability (tolerates 1 node failure).
This is a practical implementation of CP with tunable consistency.

  </div>
</div>

---

**12. Cassandra is a distributed database designed for AP systems. If you need strong consistency in Cassandra, what configuration would you use?**

- [ ] a) Replication factor of 1 (single node)
- [ ] b) Read/write with QUORUM consistency level
- [ ] c) Set consistency level to ONE (fast but weak)
- [ ] d) Read/write with QUORUM consistency level (majority of replicas)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Cassandra defaults to AP (ONE consistency), but you can tune it:
- ONE: Fast, AP behavior (eventual consistency)
- QUORUM: Slower, CP-like behavior (strong consistency from majority)
- ALL: Slowest, strongest consistency (all replicas must acknowledge)

  </div>
</div>

---

**13. You're designing a global e-commerce platform with product catalogs, user sessions, and order processing. Which should use strong consistency?**

- [ ] a) Product catalog (catalog changes are frequent)
- [ ] b) User sessions (session data isn't critical)
- [ ] c) Search results (stale results are acceptable)
- [ ] d) Order processing and inventory management

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Order processing and inventory require strong consistency to prevent:
- Double-selling the same item
- Incorrect charge amounts
- Lost or duplicate orders

Product catalogs, user sessions, and search can use eventual consistency for performance.

  </div>
</div>

---

**14. In a globally distributed system, network latency between regions is 200ms. If you need strong consistency, what's the minimum write latency?**

- [ ] a) 0ms (write happens locally and asynchronously replicates)
- [ ] b) 50ms (compression reduces latency)
- [ ] c) 200ms (only need to write to one region)
- [ ] d) 400ms+ (must wait for acknowledgment from majority of regions)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    For strong consistency across regions:
1. Write to primary region: ~0ms local
2. Replicate to at least one other region: 200ms round-trip
3. Wait for acknowledgment: 400ms+

This is why global AP systems are faster - they write locally and replicate asynchronously.

  </div>
</div>

---

**15. What's the difference between BASE (Basically Available, Soft state, Eventual consistency) and ACID (Atomicity, Consistency, Isolation, Durability)?**

- [ ] a) BASE is stricter than ACID, requiring perfect consistency
- [ ] b) They're synonyms, just different names for the same concept
- [ ] c) BASE is for distributed systems, ACID is only for single-node databases
- [ ] d) BASE is an alternative to ACID that prioritizes availability over strong consistency

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    ACID: Strict consistency, all-or-nothing transactions (e.g., PostgreSQL).
BASE: Flexible, available, eventual consistency (e.g., Cassandra, DynamoDB).
ACID = CP systems (banking, inventory)
BASE = AP systems (social media, caching)

  </div>
</div>

---

**16. A read-after-write consistency model ensures that a client always sees their own writes. What consistency level is this?**

- [ ] a) Weak Consistency (writes might not be visible)
- [ ] b) Strong Consistency (all clients see all writes immediately)
- [ ] c) Eventual Consistency (eventually converges)
- [ ] d) Causal Consistency (session consistency)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Causal Consistency ensures that a client sees their own writes immediately (session-level consistency).
This is stronger than eventual consistency but weaker than strong consistency.
It's commonly used in distributed databases to provide a good balance of performance and correctness.

  </div>
</div>

---

**17. In Sruja, how would you document that a database uses eventual consistency for high availability?**

- [ ] a) Use a 'relaxed' tag in the relationship
- [ ] b) Don't model it - Sruja assumes strong consistency by default
- [ ] c) Create two databases and say they're 'somewhat consistent'
- [ ] d) Add tags like 'AP-System' and 'Eventual-Consistency' to the database definition

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    In Sruja, you explicitly document consistency guarantees:

```sruja
// partial
UserDB = database "User Database" {
    technology "Cassandra"
    description "Replication factor 3, eventual consistency for high availability"
    tags ["AP-System", "Eventual-Consistency"]
}
```

This makes architectural trade-offs visible in your documentation.

  </div>
</div>

---

**18. A system needs 99.999% availability but can tolerate 5-second data staleness. What's the best approach?**

- [ ] a) Strong consistency with synchronous replication across all nodes
- [ ] b) Single-node database (simplest, no network issues)
- [ ] c) Reject all writes during network partitions
- [ ] d) Eventual consistency with asynchronous replication and caching

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    99.999% availability requires AP design. 5-second staleness is acceptable for eventual consistency.
Approach:
- Write to local node immediately (low latency, always available)
- Replicate asynchronously (eventual consistency)
- Cache reads aggressively (reduce staleness perception)
- Use read-your-writes consistency for critical user operations

  </div>
</div>

---

**19. What happens when a CP system experiences a network partition?**

- [ ] a) The system continues serving all requests with slightly stale data
- [ ] b) The system becomes completely unavailable (no data can be read or written)
- [ ] c) The system switches to AP mode automatically
- [ ] d) The system rejects requests that can't be guaranteed to be consistent (returns errors)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    CP systems prioritize consistency. During a partition:
- If a write can't be replicated to a majority, it's rejected
- If a read can't get the latest data, it's rejected
- Users see errors, but never incorrect data
- This is acceptable in banking, inventory, and other correctness-critical systems

  </div>
</div>

---

**20. A distributed database has 5 nodes. Network partition splits them: 3 nodes in group A, 2 nodes in group B. What happens in a CP system?**

- [ ] a) Both groups accept writes (both available)
- [ ] b) Group B accepts writes (it's smaller, so it's backup)
- [ ] c) Neither group can operate (complete system failure)
- [ ] d) Only group A (3 nodes, majority) can accept writes. Group B is read-only or unavailable.

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    In CP systems with majority consensus:
- Group A (3 nodes) has quorum (majority): can accept writes and reads
- Group B (2 nodes) lacks quorum: read-only or rejects writes
- After partition heals, Group B syncs data from Group A
- This ensures consistency (no conflicting writes) but reduces availability for Group B users

  </div>
</div>

---

**21. What's the relationship between latency and consistency in distributed systems?**

- [ ] a) Strong consistency always has lower latency than eventual consistency
- [ ] b) Eventual consistency always has lower latency, regardless of design
- [ ] c) Latency and consistency are independent - no relationship exists
- [ ] d) Strong consistency typically requires more coordination, resulting in higher latency

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Strong consistency needs:
- Synchronous replication across nodes
- Distributed consensus protocols (Paxos, Raft)
- Waiting for acknowledgments

Eventual consistency:
- Write locally, replicate asynchronously
- No coordination overhead
- Lower latency but risk of stale reads

Trade-off: Stronger consistency = higher latency

  </div>
</div>

---
