---
title: "Lesson 1: What is System Design?"
weight: 1
summary: "Introduction to System Design, Requirements, and Trade-offs."
learning_objectives:
  - Understand functional vs non-functional requirements
  - Practice clarifying requirements in interviews
  - Recognize common trade-offs in system design
estimated_time: "20 minutes"
difficulty: "beginner"
---

# Lesson 1: What is System Design?

## Defining System Design

System design is the process of defining the architecture, components, modules, interfaces, and data for a system to satisfy specified requirements. It is the bridge between business requirements and the final code.

At its core, system design is about **managing complexity** and **making trade-offs**. There is rarely a single "correct" design; instead, there are different designs that optimize for different goals (e.g., speed of development vs. system performance).

## Requirements Analysis (Critical Interview Skill)

**In system design interviews, the first thing you should do is clarify requirements.** Interviewers expect this and it shows you think systematically.

Every system design interview or real-world project starts with clarifying requirements. These are generally categorized into two types:

### 1. Functional Requirements
These define **what** the system should do. They describe the specific behaviors or functions.
*   *Example:* "Users should be able to post a tweet."
*   *Example:* "The system should send a notification when a new follower is added."

### 2. Non-Functional Requirements (NFRs)
These define **how** the system should perform. They act as constraints on the design.
*   *Scalability:* Can the system handle 10 million daily active users?
*   *Availability:* Will the system be up 99.99% of the time?
*   *Latency:* Should the API respond within 200ms?
*   *Consistency:* Do all users need to see the same data at the same time?

## The Art of Trade-offs

"Good" system design is all about choosing the right trade-offs.
*   **Consistency vs. Availability:** You often can't have both perfectly (CAP Theorem).
*   **Latency vs. Throughput:** Optimizing for one might hurt the other.
*   **Cost vs. Performance:** Faster hardware is more expensive.

---

## Visualizing Architecture (The C4 Model)

To communicate design effectively, we need a standard way to draw it. Sruja uses the **C4 Model**, which breaks a system down into hierarchical levels:

1.  **System Context (Level 1):** The big picture. Your system and the people/systems it interacts with.
2.  **Container (Level 2):** The deployable units (e.g., API, Database, Mobile App).
3.  **Component (Level 3):** The internal code structure (e.g., Controllers, Managers).

*For a detailed guide, see [The C4 Model](/docs/concepts/c4-model).*

---

## 🛠️ Sruja Perspective: Documenting Requirements

While Sruja is primarily for modeling architecture, it is also an excellent place to capture your requirements right alongside your design.

In Sruja, you can use the `description` field or comments to document high-level requirements.

```sruja
architecture "Social Media Platform" {
    // Native Requirement Support
    requirement R1 functional "Users can post tweets"
    requirement R2 performance "Must handle 10k writes/second"

    person User "User"
    
    system Twitter "Social Media Platform" {
        description "Allows users to post short messages and follow others."
    }

    User -> Twitter "Posts tweets"
}
```

By using the `requirement` keyword, you make requirements a first-class citizen of your architecture. These can be validated and tracked by the Sruja CLI.

---

## Knowledge Check
- Which statement best describes non-functional requirements?
- In interviews, what is the first step before proposing an architecture?
- Name one common trade-off pair in system design.

## Further Reading
- Reference: [`requirements` keyword](/docs/reference/syntax)
- Tutorial: [CLI Basics](/tutorials/basic/cli-basics)
