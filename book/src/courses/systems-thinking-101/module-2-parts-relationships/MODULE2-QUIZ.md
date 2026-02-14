---
title: "Module 2 Quiz: Parts and Relationships"
weight: 5
summary: "Test your understanding of identifying parts, modeling elements, defining relationships, and organizing systems"
time: "10 min"
---

# Module 2 Quiz: Parts and Relationships

Test your understanding of how to identify system parts, model them with Sruja elements, define meaningful relationships, and organize everything with clear hierarchy.

This quiz covers all four lessons in Module 2. Take your time, think through each question, and read the explanations to reinforce your learning.

---

## Question 1: The C4 Hierarchy

You're modeling a food delivery app and reading these requirements:

> "Customers can browse restaurants, place orders, and track deliveries. Restaurant owners can manage their menus and update prices. The app sends SMS notifications for order updates. Order data is stored in a database. The system integrates with Google Maps for route planning."

At the **person** level of the C4 hierarchy, which parts should you identify?

**A)** Customer, Restaurant, Order, SMS, Database, Google Maps
**B)** Customer, Restaurant Owner
**C)** Food Delivery App, SMS Service, Database, Google Maps
**D)** Browse Restaurant, Place Order, Track Delivery, Manage Menu

<details>
<summary>Click to see the answer</summary>

**Answer: B) Customer, Restaurant Owner**

**Explanation:**

The C4 hierarchy's first level (Level 1) is all about **people**—humans who interact with your system. Let's analyze each option:

- **A)** Incorrect. Order, SMS, Database, and Google Maps are not people. Orders are things the system handles. SMS, Database, and Google Maps are systems or services, not humans.
- **B)** Correct! Customers and restaurant owners are the humans who interact with the food delivery app. These are exactly what you should model at the person level.
- **C)** Incorrect. These are all systems or services, not people. The Food Delivery App is the main system you're building. SMS, Database, and Google Maps are external systems you depend on. They belong at Level 2 (system), not Level 1 (person).
- **D)** Incorrect. These are actions or functions the system provides, not people. "Browse restaurant" is something a customer does, but the customer is the person, not the action.

**Key Takeaway:** Always start with people. Every system exists to serve humans. Ask yourself: "Who interacts with this system?" Those are your people.

</details>

---

## Question 2: Choosing Element Types

You're modeling a healthcare platform with these requirements:

> "Doctors can view patient records and prescribe medications. Patients can book appointments and view their medical history. The platform has a web application for both doctors and patients, a backend API that processes requests, and a PostgreSQL database for storing records. The API has separate services for authentication, patient management, and appointment scheduling. The platform sends email notifications for appointment reminders."

Which parts should you model as **containers** (not systems or components)?

**A)** Doctor, Patient, Web Application, API, PostgreSQL Database
**B)** Web Application, Backend API, PostgreSQL Database, Email Service
**C)** Authentication Service, Patient Management Service, Appointment Scheduling Service
**D)** Healthcare Platform, Web Application, Backend API

<details>
<summary>Click to see the answer</summary>

**Answer: B) Web Application, Backend API, PostgreSQL Database, Email Service**

**Explanation:**

Containers are the **deployable units** within your systems—the things you actually deploy to production. Let's analyze each option:

- **A)** Incorrect. Doctor and Patient are people (Level 1), not containers. The other three items are correct containers, but mixing levels makes this wrong.
- **B)** Correct! All four of these are containers:
  - **Web Application** — A deployable frontend application
  - **Backend API** — A deployable API service
  - **PostgreSQL Database** — A database you deploy and manage
  - **Email Service** — Either an internal email service or a third-party integration you deploy

- **C)** Incorrect. These are components (Level 4), not containers. They live inside the Backend API container. You wouldn't deploy them independently.
- **D)** Incorrect. Healthcare Platform is a system (Level 2), not a container. The web application and backend API are containers, but mixing them with a system is inconsistent.

**Key Takeaway:** Containers = what you actually deploy. Web apps, APIs, databases, caches, message queues—these are all containers. Systems contain containers. Components live within containers.

</details>

---

## Question 3: Writing Relationship Labels

You're modeling a bank's mobile banking app. Which relationship label is the most informative and follows best practices?

**A)** `Customer -> MobileApp "Uses"`
**B)** `Customer -> MobileApp "Checks balance"`
**C)** `Customer -> MobileApp "HTTPS request"`
**D)** `Customer -> MobileApp "User interaction"`

<details>
<summary>Click to see the answer</summary>

**Answer: B) Customer -> MobileApp "Checks balance"**

**Explanation:**

Good relationship labels tell a story about what's actually happening. They're specific, use present tense verbs, and describe business behavior rather than technical details. Let's analyze each option:

- **A)** Incorrect. "Uses" is too generic. What are they using it for? Checking balance? Transferring money? Paying bills? The label provides no information about the user's intent.
- **B)** Correct! "Checks balance" is specific and informative. It tells you exactly what the customer is doing with the mobile app. It's a clear, present tense verb that describes business behavior.
- **C)** Incorrect. "HTTPS request" describes the protocol/technology, not the business action. This is an implementation detail, not what's happening from a user's perspective. Stakeholders don't care about HTTPS; they care about what users can do.
- **D)** Incorrect. "User interaction" is vague and unhelpful. What kind of interaction? Every relationship involving a person is a "user interaction." This label provides no useful information.

**Key Takeaway:** Write labels that tell a story. Ask yourself: "What is actually happening here?" Use present tense verbs like "browses," "queries," "processes," "sends," "receives." Be specific about the action and the context.

</details>

---

## Question 4: Nesting and Hierarchy

You're reviewing a team's architecture diagram and notice this structure:

```sruja
ECommerce = system "E-Commerce Platform" {
  Frontend = container "Web App"
  API = container "API Service" {
    AuthService = component "Auth Service"
  }
  Database = database "PostgreSQL"
}
```

What's wrong with this structure?

**A)** Nothing—this is correct and well-structured
**B)** Frontend and Database should also have components for consistency
**C)** Only API has components, which is inconsistent
**D)** AuthService should be a container, not a component

<details>
<summary>Click to see the answer</summary>

**Answer: C) Only API has components, which is inconsistent**

**Explanation:**

The issue here is **inconsistent nesting**. When you add components to one container but not others, it creates confusion and questions:

- Is Frontend simpler than API? Maybe, but why doesn't it have components?
- Is Database internally complex? If so, shouldn't it have components?
- Did the team forget to break down Frontend and Database?
- Is there a meaningful architectural difference we should understand?

Let's analyze each option:

- **A)** Incorrect. While this might be technically valid Sruja, it's architecturally inconsistent and confusing.
- **B)** Incorrect. You don't need to add components to Frontend and Database just for consistency. If they're simple, keep them simple. The problem isn't that Frontend and Database lack components—the problem is that **only** API has components, which creates confusion.
- **C)** Correct! The inconsistency is the issue. Either break down all three containers into components (if they're all complex) or break down none of them (if they're all simple). If API is uniquely complex, explain why—don't just add components randomly.
- **D)** Incorrect. AuthService is appropriately modeled as a component within the API container. It's an internal module of the API, not a standalone deployable unit.

**Key Takeaway:** Consistency matters more than any individual nesting decision. If you add components to one container, have a clear reason why that container is different. Either break everything down to the same level, or explain the architectural differences.

</details>

---

## Question 5: Practical Application

You're modeling a social media platform with these requirements:

> "Users can create profiles, post content, and like posts. They can also comment on posts and follow other users. The platform sends push notifications for likes and comments. Content is stored in a PostgreSQL database and cached in Redis for performance. The platform integrates with Firebase for push notifications and with AWS S3 for storing images and videos."

You want to create a diagram for **business stakeholders** (not developers). Which structure is most appropriate?

**A)** Include people, systems, containers, and components for complete detail
**B)** Include people, systems, and containers—skip components
**C)** Include only systems—people and containers are too detailed
**D)** Include only containers—systems and people are unnecessary

<details>
<summary>Click to see the answer</summary>

**Answer: B) Include people, systems, and containers—skip components**

**Explanation:**

The golden rule: **match the level of detail to your audience**. Business stakeholders care about the big picture—not implementation details. Let's analyze each option:

- **A)** Incorrect. Components are implementation details that developers care about, not stakeholders. Showing AuthService, PostService, LikeService, etc., would overwhelm and confuse business stakeholders.
- **B)** Correct! This structure tells the right story for stakeholders:
  - **People** (Users) — Who uses the system?
  - **Systems** (Social Media Platform, Firebase, AWS S3) — What are the major systems?
  - **Containers** (Web App, API, Database, Cache) — What are the main deployable units?
  
  This gives stakeholders enough context to understand the overall architecture without drowning in technical details.

- **C)** Incorrect. People are critical for understanding who uses the system. Without people, the diagram lacks purpose. Containers are also important for showing the structure of the main system.
- **D)** Incorrect. You need systems to show the big picture and dependencies. You need people to show who uses the system. Containers alone don't tell a complete story.

**What the diagram should look like:**

```sruja
// People (who uses the system?)
User = person "User"

// Systems (what are the major systems?)
SocialMedia = system "Social Media Platform"
Firebase = system "Firebase Push Notifications"
AWSS3 = system "AWS S3 Storage"

// Containers (what's deployable within the main system?)
SocialMedia = system "Social Media Platform" {
  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "PostgreSQL"
  Cache = datastore "Redis Cache"
}

// Key relationships
User -> SocialMedia.WebApp "Uses platform"
SocialMedia.API -> SocialMedia.Database "Stores content"
SocialMedia.API -> SocialMedia.Cache "Reads cached content"
SocialMedia.API -> Firebase "Sends push notifications"
SocialMedia.API -> AWSS3 "Stores images and videos"
```

**Key Takeaway:** Not every diagram needs every level of detail. Match your modeling to your audience:
- **Stakeholders:** People, Systems, Containers (skip components)
- **Product Managers:** People, Systems, Containers
- **Developers:** People, Systems, Containers, Components
- **DevOps:** People, Systems, Containers (with infrastructure details)

</details>

---

## How Did You Do?

Count your correct answers:

- **5 correct:** Excellent! You have a solid understanding of parts, relationships, and hierarchy. You're ready to apply these concepts in real projects.
- **4 correct:** Great work! You understand most concepts well. Review the explanation for the question you missed to solidify your understanding.
- **3 correct:** Good effort! You understand the basics but need to practice on some concepts. Re-read the relevant lessons and try the quiz again.
- **1-2 correct:** Keep learning! You're on the right track, but need to review the lessons more carefully. Focus on understanding the "why" behind each concept, not just the "how."

---

## What's Next?

Ready to move on? In [Module 3: Boundaries](../module-3-boundaries/module-overview.md), you'll learn how to define where one system ends and another begins. This is crucial for:

- Understanding dependencies between systems
- Managing complexity in large architectures
- Designing decoupled, maintainable systems
- Making architectural decisions about integration patterns

See you there!