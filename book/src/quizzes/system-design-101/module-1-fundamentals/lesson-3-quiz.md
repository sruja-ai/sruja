<!-- Auto-generated quiz from TOML -->
<!-- Source: lesson-3-quiz.toml -->

**1. In system design, what term describes the percentage of time a system is operational and accessible (uptime)?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Availability

**Alternative answers:**
- availability
- uptime

**Explanation:**
Availability measures how often a system is up and accessible. It's about uptime percentage (e.g., 99.9%).


</details>

---

**2. In system design, what term describes the probability that a system will function correctly without failure for a specified period (correctness)?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Reliability

**Alternative answers:**
- reliability
- correctness

**Explanation:**
Reliability measures how often a system functions correctly without errors. A system can be available (up) but unreliable (returning 500 errors).


</details>

---

**3. Which availability level allows approximately 8.76 hours of downtime per year?**

- [ ] a) 99% (Two nines) - 3.65 days/year
- [ ] b) 99.99% (Four nines) - 52.6 minutes/year
- [ ] c) 99.999% (Five nines) - 5.26 minutes/year
- [ ] d) 99.9% (Three nines) - 8.76 hours/year

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    99.9% availability = 99.9% uptime = 0.1% downtime = 0.001 × 365 days × 24 hours = 8.76 hours/year.
Each additional "9" reduces downtime by a factor of 10.

  </div>
</div>

---

**4. AWS S3 provides 99.999999999% durability (11 nines). What does this mean in practical terms?**

- [ ] a) S3 can be down for 5 minutes per year
- [ ] b) If you store 10,000 objects, you'll lose one per year
- [ ] c) S3 guarantees 11 nines availability (uptime)
- [ ] d) If you store 10,000 objects, you'll lose one on average every 10,000,000 years

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    99.999999999% durability means 0.000000001% chance of data loss. For 10,000 objects, that's a 0.01% chance per year = once every 10,000 years.
Note: Durability ≠ Availability. S3's availability is 99.99% (52.6 minutes/year downtime).

  </div>
</div>

---

**5. A system has a Single Point of Failure (SPOF). What happens if that component fails?**

- [ ] a) The system continues operating normally with degraded performance
- [ ] b) The load balancer automatically routes traffic to other components
- [ ] c) The system fails gracefully with an error message
- [ ] d) The entire system becomes unavailable or non-functional

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    A Single Point of Failure is a component whose failure causes the entire system to fail. Redundancy is used to eliminate SPOFs by having backup components.

  </div>
</div>

---

**6. In an Active-Passive redundancy setup, what happens when the active server fails?**

- [ ] a) Both servers were handling traffic, so traffic continues normally
- [ ] b) The passive server automatically starts handling all traffic (failover)
- [ ] c) The system sends an error to users until manual intervention
- [ ] d) The passive server takes over (failover), but there's a brief interruption during switch

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Active-Passive: One server handles traffic, the other is on standby. Failover happens automatically, but there's typically a few seconds of interruption as the passive server comes online.

  </div>
</div>

---

**7. In an Active-Active redundancy setup, what happens when one server fails?**

- [ ] a) The entire system goes down because both servers were critical
- [ ] b) The other server takes 5 minutes to restart before accepting traffic
- [ ] c) Users connected to the failed server experience errors until they reconnect
- [ ] d) Traffic is redistributed to the remaining server(s) with minimal or no interruption

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Active-Active: Both servers handle traffic simultaneously. If one fails, the other continues handling its traffic plus takes over some traffic from the failed server.
Minimal interruption if load balancer detects failure quickly.

  </div>
</div>

---

**8. You're building a banking application that processes financial transactions. Which redundancy approach is most appropriate?**

- [ ] a) Active-Active with no failover testing (cheaper)
- [ ] b) Active-Passive with automatic failover (good balance)
- [ ] c) No redundancy (transactions are rare, so SPOF is acceptable)
- [ ] d) Active-Active with rigorous failover testing and synchronous replication

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Banking requires both high availability and strong consistency. Active-Active with synchronous replication ensures no data loss during failover.
Regular chaos engineering tests (like Netflix's Chaos Monkey) ensure failover actually works when needed.

  </div>
</div>

---

**9. A content delivery network (CDN) has 100 edge servers worldwide. If 5 servers fail simultaneously, the CDN remains operational. This is an example of:**

- [ ] a) Vertical scaling
- [ ] b) Single Point of Failure
- [ ] c) Active-Passive redundancy
- [ ] d) Elimination of Single Points of Failure through redundancy

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    With 100 servers, the failure of 5 has minimal impact because there's no single critical component. This is horizontal scaling providing both availability and resilience.

  </div>
</div>

---

**10. Your database server fails. You have a standby replica that can take over in 30 seconds. What's your Recovery Time Objective (RTO)?**

- [ ] a) 0 seconds (no data loss)
- [ ] b) 30 seconds (time to detect and switch)
- [ ] c) 1 minute (time to fully restore service)
- [ ] d) 30 seconds (time to detect failure and failover to standby)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    RTO (Recovery Time Objective) is the maximum acceptable time to restore service after a disruption.
In this case, RTO = 30 seconds (detection + failover time).
RPO (Recovery Point Objective) would be how much data is lost (depends on replication lag).

  </div>
</div>

---

**11. Netflix uses Chaos Monkey, a tool that randomly terminates instances in production. What is the purpose of this?**

- [ ] a) To save money by reducing server count
- [ ] b) To test if the system can handle automatic failover and resilience
- [ ] c) To identify which servers are underutilized
- [ ] d) To proactively test that the system remains available when components fail

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    Chaos Engineering is about intentionally causing failures to test resilience. If Chaos Monkey kills a server and Netflix users don't notice, the system is resilient.
This practice transformed Netflix's availability from 99.9% to 99.99%+ by finding and fixing weaknesses before real outages occur.

  </div>
</div>

---

**12. Your e-commerce site's primary database fails. You have a standby replica with 5 minutes of replication lag (meaning it's missing the last 5 minutes of orders). What's your RPO?**

- [ ] a) 0 minutes (no data lost)
- [ ] b) 5 minutes (you lose up to 5 minutes of data)
- [ ] c) Infinite (you can't recover the data)
- [ ] d) 5 minutes (you lose up to 5 minutes of transaction data)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    RPO (Recovery Point Objective) is the maximum acceptable data loss measured in time.
With 5 minutes of replication lag, RPO = 5 minutes. Any orders placed in the last 5 minutes would need to be recovered from logs or customer records.

  </div>
</div>

---

**13. A system has 99.9% availability. How much downtime is this per month?**

- [ ] a) 8.76 hours (same as per year)
- [ ] b) 43.8 minutes (8.76 hours ÷ 12)
- [ ] c) 5 minutes (same as five nines per year)
- [ ] d) 43.8 minutes (8.76 hours ÷ 12 months)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    99.9% = 8.76 hours downtime/year ÷ 12 months = 0.73 hours = 43.8 minutes per month.
This means the system can be down for ~43 minutes each month while maintaining 99.9% availability SLA.

  </div>
</div>

---

**14. You're designing a global video streaming service. Users expect 99.99% availability. What's the maximum acceptable downtime per month?**

- [ ] a) 43.8 minutes (same as 99.9%)
- [ ] b) 5.26 minutes (same as five nines per year)
- [ ] c) 4.38 minutes (52.6 minutes/year ÷ 12)
- [ ] d) 4.38 minutes (52.6 minutes/year ÷ 12 months)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    99.99% = 52.6 minutes downtime/year ÷ 12 months = 4.38 minutes per month.
Achieving this requires Active-Active setup across multiple regions with automatic failover, as any maintenance or failure costs precious minutes.

  </div>
</div>

---

**15. In Sruja, how would you model a database with a standby replica for high availability?**

- [ ] a) Use a single database component with 'standby' in description
- [ ] b) Create two databases and don't connect them (Sruja auto-discovers)
- [ ] c) Use a load balancer with one database connection
- [ ] d) Define two database components (Primary and Standby) with a replication relationship

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    In Sruja, you explicitly model redundant components:

```sruja
PrimaryDB = database "Primary Database" { ... }
StandbyDB = database "Standby Database" {
    description "Replicates from PrimaryDB. Promoted to primary if PrimaryDB fails."
}
PrimaryDB -> StandbyDB "Replicates data"
```

This makes the redundancy strategy visible in your architecture diagrams.

  </div>
</div>

---

**16. Which scenario demonstrates the difference between availability and reliability?**

- [ ] a) Server crashes and the site goes down (neither available nor reliable)
- [ ] b) Site loads quickly and returns correct data (both available and reliable)
- [ ] c) Site is accessible but returns random 500 errors to users
- [ ] d) Site is accessible but returns random 500 errors to users

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    This is AVAILABLE (site is up) but NOT RELIABLE (not functioning correctly).
- Available = 100% uptime (never down)
- Reliable = 100% correct (no errors)
- Perfect = Both available and reliable

  </div>
</div>

---

**17. Your company's SLA (Service Level Agreement) promises 99.9% uptime. In the last month, you had 1 hour of downtime. What's the penalty?**

- [ ] a) No penalty (1 hour < 43.8 minutes allowed)
- [ ] b) 50% penalty for missing the SLA
- [ ] c) Calculate the difference: (60 min - 43.8 min) × penalty rate
- [ ] d) 16.2 minutes exceeded (60 min - 43.8 min = 16.2 min beyond SLA)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    99.9% SLA = 43.8 minutes downtime/month allowed.
Actual = 60 minutes downtime.
Exceeded by 16.2 minutes.
Penalty is typically calculated based on the exceeded minutes times a penalty rate (e.g., 1% credit per minute exceeded).
This demonstrates why monitoring availability in real-time is critical for SLA compliance.

  </div>
</div>

---

**18. What is the term for the process of switching from a failed primary component to a redundant standby component?**

<details>
<summary><strong>Click to see answer</strong></summary>

**Answer:** Failover

**Alternative answers:**
- failover
- fail-over
- switchover

**Explanation:**
Failover is the process of automatically or manually switching to a redundant system upon failure. This is critical for high availability systems.
Failover can be automatic (system detects failure and switches) or manual (operator triggers switch).


</details>

---

**19. A healthcare application must be available 24/7 for emergency access. The database can only be down for maintenance 4 hours per year. What's the minimum availability required?**

- [ ] a) 99% (allows 3.65 days/year)
- [ ] b) 99.9% (allows 8.76 hours/year)
- [ ] c) 99.999% (allows 5.26 minutes/year)
- [ ] d) 99.95% (allows 4.38 hours/year, just under the 4-hour requirement)

<button class="check-answer-btn" data-correct="d">Check Answer</button>

<div class="answer-feedback" style="display: none;">
  <p class="feedback-text"></p>
  <div class="explanation" style="display: none;">
    4 hours/year downtime tolerance:
4 hours ÷ (365 days × 24 hours) = 4 ÷ 8760 = 0.0457% downtime
Availability = 100% - 0.0457% = 99.954%

Rounding to standard availability levels: 99.95% allows 4.38 hours/year, which meets the requirement.
This requires careful planning: Active-Active setup, scheduled maintenance windows, and minimal unplanned outages.

  </div>
</div>

---
