# Module 1: Fundamentals - Quiz

Let's test your understanding of all eight concepts from this module.

## Quiz Questions

**1. You've been working on a system for months. You fix one bug, test it thoroughly, and celebrate—only to have three new bugs appear the next day. You fix those, but similar bugs keep appearing after every release. Based on what you learned in this module, which approach would help you solve this problem more effectively?**

[ ] A. Keep fixing each bug as it appears—that's the most direct way
[ ] B. Look for patterns across releases—bugs are recurring, so there must be a deeper cause
[ ] C. Focus on optimizing database queries—the performance reports show slow queries
[ ] D. Investigate the team structure and beliefs—maybe they're affecting how code is written

**2. You're creating an architecture diagram for your CTO. They want to understand the big picture: which software systems exist, who uses them, and how they connect to business value. You have diagrams showing individual components (web app, API, database), containers and their interactions, and even internal modules and classes. What's the problem with your approach?**

[ ] A. You're showing too much detail—CTOs don't need to see modules
[ ] B. You're showing exactly what they need—systems, containers, and relationships
[ ] C. You're missing the people and stakeholders who use these systems
[ ] D. All the information is there, just not organized for a high-level view

**3. Imagine you're designing a new e-commerce platform. Your team believes "ship fast, fix bugs later—we can optimize performance once we have users." How might this mental model affect your architecture decisions, and what would be a better approach?**

[ ] A. This mental model is great—it prioritizes getting to market quickly
[ ] B. This mental model might lead to skipping tests and creating technical debt, but that's acceptable for a startup
[ ] C. This mental model could lead to tightly coupled architecture and lack of caching, making performance optimization difficult later
[ ] D. This mental model affects more than just code—it could lead to poor monitoring, unreliable deployments, and lack of documentation

**4. You're looking at a slow order processing flow. You see: `Customer → Web App → API → Database (500ms) → Payment Gateway → API → Customer (Confirmation)`. Where is the bottleneck, and what would you do about it?**

[ ] A. The database query is the bottleneck—500ms is too slow, optimize it
[ ] B. The payment gateway—you can't see how long it takes, but it's external
[ ] C. The flow shows multiple steps—you need to parallelize them or use caching
[ ] D. The bottleneck isn't clear from this flow—you need to add timing information to each step

**5. Think about your current project or a system you've worked on recently. Can you identify one thing that's clearly "inside" your system's boundary and one thing that's clearly "outside"? How does this distinction affect how you work on or think about each one?**

Take a moment to reflect. There's no single right answer here—this is about building awareness and intuition for recognizing boundaries in your own work.

```

## Answers & Discussion

**1. B. Look for patterns across releases—bugs are recurring, so there must be a deeper cause** – This demonstrates applying Lesson 2 (The Iceberg Model). You're experiencing events (individual bugs), and even noticing a pattern (bugs recur after every release), but to solve it effectively you need to go deeper. Looking for patterns is exactly what Lesson 2 teaches. The deeper cause might be a structure (how teams work, testing process) or even a mental model (beliefs about shipping fast). This is systems thinking in action—moving from event-level debugging to pattern-level and structure-level analysis.

**2. C. You're missing the people and stakeholders who use these systems** – This demonstrates understanding Lesson 3 (Systems in Software Architecture). Your diagrams show technical components beautifully, but they miss the human and business context. A CTO cares about: who are the users? Who are the stakeholders? What business value does this system provide? The C4 model helps you remember to start with Person (users, stakeholders) and System (software systems) levels before drilling down to technical details. Without the people layer, your diagrams answer "what exists" but not "why it matters" or "who cares."

**3. C. This mental model could lead to tightly coupled architecture and lack of caching, making performance optimization difficult later** – This demonstrates understanding Lesson 7 (Feedback Loops) and Lesson 5 (Boundaries). The mental model "ship fast, fix bugs later" creates a reinforcing feedback loop: ship fast → get users → more pressure to ship fast → skip tests and optimization → more bugs → need to fix faster. This is a positive (reinforcing) loop that amplifies speed but degrades code quality. The result is tightly coupled architecture (because you're shipping fast without thinking about design) and lack of performance optimizations (because you're "fixing bugs later" not "building it right the first time"). This creates a negative consequence that makes later performance optimization much harder. A better mental model might be "build quality foundations first, then optimize speed" or "invest in testing and automation to ship fast reliably."

**4. A. The database query is the bottleneck—500ms is too slow, optimize it** – This demonstrates understanding Lesson 6 (Flows). Looking at the flow as a sequence, you can see that the database step takes 500ms, while the other steps are likely faster. If the total acceptable time is, say, 1 second, and the database is taking half of that, it's clearly the bottleneck. You don't need to guess—you can see from the flow which step dominates the time. This is the value of modeling flows: they make bottlenecks visible and concrete, not just something you feel intuitively.

**5. (Your reflection)** – This open-ended question tests your ability to apply Lesson 8 (Context) to your own work. There's no right answer—what matters is that you can recognize that things inside your boundary (components you build, code you write, decisions you make) are different from things outside (external APIs, third-party services, organizational constraints beyond your control, business requirements you don't set). This distinction affects how you approach them: inside, you can change directly; outside, you need to understand, negotiate with, plan around, or accept as given constraints. Recognizing this in your own work is the first step to effective systems thinking.

## What's Next?

Congratulations! You've completed Module 1: Fundamentals. You now have a complete toolkit for thinking about systems holistically:

- ✅ Systems thinking fundamentals (seeing the whole, not just parts)
- ✅ The iceberg model (looking deeper than surface events)
- ✅ Systems as systems of systems (understanding layers and dependencies)
- ✅ Parts and relationships (modeling components and interactions)
- ✅ Boundaries (what's inside vs. outside your system)
- ✅ Flows (how things move through your system)
- ✅ Feedback loops (how systems adapt and self-regulate)
- ✅ Context (the environment your system lives in)

In the next module, you'll dive deeper into **Parts & Relationships**—learning to identify components, model their interactions with precision, and create diagrams that communicate clearly to different audiences.

You're ready to apply these foundational concepts to real architecture modeling!