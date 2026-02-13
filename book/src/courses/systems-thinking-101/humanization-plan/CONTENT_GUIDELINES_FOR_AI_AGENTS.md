# Content Guidelines for Humanizing Course Content

**Purpose:** This document provides comprehensive guidelines for humanizing AI-generated course content to create engaging, conversational, and effective learning materials.

**Audience:** AI agents tasked with rewriting technical course content to sound human-written.

**Context:** These guidelines are based on successful humanization of Systems Thinking 101 course (8 lessons in Module 1).

---

## Core Principles

### 1. Logical Order: Concept First, Then Implementation

**Rule:** Always explain the concept thoroughly before showing how to implement it.

**Why:** Learners need to understand WHAT and WHY before learning HOW.

**Structure:**
```
1. Hook/Opening → 2. Explain concept (what, why, types, examples) → 3. Show implementation (Sruja syntax) → 4. Practical examples → 5. Summary
```

**Don't do:**
- Jump straight into code examples
- Explain Sruja syntax before explaining the concept
- Mix concept and implementation randomly

**Do:**
- Dedicate 60-70% of lesson to explaining the concept
- Dedicate 30-40% to showing implementation
- Make clear separation between concept and implementation

---

### 2. Conversational Tone

**Rule:** Write like a human mentor talking to a colleague over coffee.

**Why:** Learners feel engaged when content feels personal and conversational.

**Voice characteristics:**
- Use "you" consistently
- Use "we" when appropriate (shared experiences)
- Ask rhetorical questions
- Use first-person occasionally ("Let me share a story...", "I've seen...")
- Be encouraging ("You've got this!", "Here's the key insight...")
- Be relatable ("Think of a time when...", "Have you ever felt...")

**Don't do:**
- Use third-person ("The learner will...", "Students should...")
- Be academic or overly formal
- Use AI-typical phrases ("In conclusion," "Furthermore," "It's important to note")
- Sound like a textbook or lecture

**Do:**
- Write in second person ("you'll learn...", "you'll notice...")
- Use conversational transitions ("Let's see...", "Here's the thing...")
- Add personal touches when natural
- Be encouraging and supportive

---

### 3. Varied Structure and Tone

**Rule:** Every lesson should feel different, not formulaic.

**Why:** AI-generated content uses identical structure for every lesson. Humans vary naturally.

**How to vary:**
- **Headings:** Don't always use "What is/What Are [Topic]?" 
  - Use alternatives: "Understanding [Topic]", "[Topic] in Practice", "How [Topic] Works", "Getting Started with [Topic]"
- **Section length:** Some sections longer, some shorter
- **Opening:** Sometimes a question, sometimes a story, sometimes a scenario
- **Examples:** Mix everyday analogies, technical examples, personal stories
- **Summary style:** Some lessons use "What to Remember", some use "Key Insights", some use narrative paragraphs

**Don't do:**
- Use identical heading format for every lesson
- Make every section the same length
- Always start with "What is/What Are [Topic]?"
- Use perfect symmetry in structure
- Repeat "Key Takeaways" in every lesson

**Do:**
- Vary heading styles naturally
- Mix section lengths based on importance
- Use different opening hooks
- Change summary style occasionally
- Feel organic, not manufactured

---

### 4. Mix Paragraphs and Bullets

**Rule:** Write in paragraphs primarily, use bullets only when truly listing items.

**Why:** Over-reliance on bullet points makes content feel robotic. Paragraphs feel more human.

**Ratio:** Aim for 60-70% paragraphs, 30-40% bullet points.

**When to use paragraphs:**
- Explaining concepts
- Providing context or background
- Walking through examples step-by-step
- Explaining WHY something matters
- Telling stories or anecdotes

**When to use bullets:**
- Listing multiple distinct items (stakeholder types, boundary types)
- Enumerating options or choices
- Quick reference lists
- DO/DON'T comparisons

**Don't do:**
- Write everything as bullet points
- Use bullets for every explanation
- Create long lists that should be paragraphs

**Do:**
- Write explanatory content in paragraphs
- Use bullets for actual lists
- Mix both naturally throughout lesson

---

## Section-by-Section Guidelines

### Frontmatter

**Purpose:** Set expectations and engage reader before lesson content.

**Required elements:**
```markdown
---
title: "Lesson X: [Topic]"
weight: X
summary: "[Engaging hook question or compelling statement, not just description]"
time: "2 minutes"
---
```

**Summary guidelines:**
- Make it engaging with a hook question, not just descriptive
- Example: "Why do same bugs keep appearing?" instead of "Understanding patterns"
- Keep it under 100 characters

**Title guidelines:**
- Can be direct ("[Topic]") or engaging ("[Engaging Title]")
- Avoid formulaic "What is/What Are [Topic]?" pattern in title

---

### Title and Learning Goals

**Title:**
```markdown
# Lesson X: [Topic]
```

**Guidelines:**
- Can be direct or engaging
- Don't repeat formulaic "What is/What Are" pattern
- Keep it clear and concise

**Learning Goals:**
```markdown
## Learning Goals

By the end of this lesson, you'll be able to:
- [Action-oriented goal 1]
- [Action-oriented goal 2]
- [Action-oriented goal 3]
```

**Guidelines:**
- Use "Learning Goals" (plural), never "Learning Goal" (singular)
- Use conversational framing: "By the end of this lesson, you'll be able to:"
- Make goals action-oriented: "Identify," "Recognize," "Apply," "Model"
- Be specific: Don't say "Understand concept," say "Identify [specific thing]"
- 3-5 goals is ideal

**Don't do:**
- Use singular "Learning Goal"
- Use passive voice: "The learner will understand..."
- Be vague: "Learn about systems"
- Have too many goals (more than 6)

**Do:**
- Use plural "Learning Goals"
- Use conversational framing
- Be specific and action-oriented
- Keep it to 3-5 goals

---

### Opening Hook

**Purpose:** Engage reader, make topic relatable, connect to their experience.

**Structure:**
```markdown
## [Engaging Heading]

[Question or story connecting to reader's experience]

[Paragraph explaining why this matters]

[Transitional sentence introducing topic]
```

**Guidelines:**
- Start with a question or relatable scenario
- Make it personal ("Have you ever...", "Think of a time when...")
- Connect to reader's work or experience
- Explain why this topic matters to them
- Transition to the main topic naturally

**Example opening:**
```markdown
## Understanding Systems Thinking

Have you ever fixed a bug, tested it thoroughly, and celebrated—only to have three new bugs appear the next day? 

These aren't just frustrating experiences. They're symptoms of looking at problems at the surface level without understanding deeper patterns and structures.

The iceberg model gives us a way to look deeper.
```

**Don't do:**
- Start with a definition
- Start with "What is [Topic]?"
- Make it abstract or theoretical
- Jump straight into content without context

**Do:**
- Start with a relatable question or story
- Make it personal and engaging
- Explain why it matters
- Transition naturally

---

### Concept Explanation Sections

**Purpose:** Thoroughly explain the concept before showing implementation.

**Structure:**
```markdown
## [Engaging Heading]

[Conversational introduction]

[Key insight or definition explained conversationally]

[Real-world example]

[Another example or detail]

[Why this matters section]
```

**Guidelines:**
- Use engaging headings (not "What is [Topic]?")
- Write conversationally
- Use "you" and "your" throughout
- Include 2-3 examples from real world
- Explain WHY it matters, not just WHAT it is
- Use analogies when helpful

**Types of concept sections:**
- "Understanding [Topic]" - general introduction
- "[Topic] in Practice" - how it's used
- "How [Topic] Works" - mechanism
- "Why [Topic] Matters" - importance
- "Types of [Topic]" - variations

**Don't do:**
- Use formulaic headings ("What is X?", "What are Y?")
- Write in academic tone
- Use abstract examples only
- Define without explaining significance

**Do:**
- Use varied, engaging headings
- Write conversationally
- Include concrete examples
- Explain significance

---

### Implementation Sections (Sruja Code)

**Purpose:** Show how to implement the concept in Sruja.

**Structure:**
```markdown
## [Action-Oriented Heading]

[Short introduction to implementation]

[Sruja code example]

[Analysis explaining what the code shows]

[Practical tips or variations]
```

**Guidelines:**
- Always come AFTER concept is fully explained
- Use action-oriented headings ("Modeling [Topic] in Sruja", "How to Model [Topic]")
- Keep code accurate and well-formatted
- Add explanation of what code shows
- Include practical tips or variations

**Example:**
```markdown
## Modeling Flows in Sruja

Let's see how to model flows using scenarios.

```sruja
scenario OrderFlow "User checkout process" {
  Customer -> WebApp "Submits order"
  WebApp -> API "Sends data"
  API -> Database "Saves order"
}
```

This scenario shows the complete user journey from submission to storage. Each arrow represents an action in the sequence.

### Practical Tips

Use descriptive labels: Instead of `API -> DB "uses"`, try `API -> DB "PostgreSQL/reads"`. This makes flows more actionable.
```

**Don't do:**
- Show code before explaining concept
- Jump straight into code without introduction
- Show code without explaining what it demonstrates
- Use complex examples when simple ones would suffice

**Do:**
- Always explain concept first
- Introduce implementation clearly
- Explain what code demonstrates
- Keep examples simple and clear

---

### Best Practices / Practical Tips Sections

**Purpose:** Provide actionable guidance for applying the concept.

**Structure:**
```markdown
## [Conversational Heading]

[Introduction to why these tips matter]

[Tip 1 with explanation]

[Tip 2 with explanation]

[Tip 3 with explanation]
```

**Guidelines:**
- Use conversational headings: "Practical Tips," "Best Practices," "How to Apply"
- Avoid rigid "Good/Bad" or "✅/❌" format
- Explain WHY each tip matters
- Provide concrete examples
- Write in paragraphs, not just bullet lists
- Use first-person when natural ("I've seen many teams...")

**Example:**
```markdown
## Practical Tips for Better Diagrams

After working with many teams, I've noticed patterns that consistently create clearer, more useful diagrams.

Be specific with relationships
Instead of writing `API → DB "uses"`, try `API → DB "PostgreSQL/reads"`. The difference might seem small, but it's huge in practice. Specific labels mean developers can look at your diagram and immediately understand what's actually happening.

Mark external systems clearly
When you depend on something outside your control, make that visible. This isn't just for documentation—it helps everyone understand risk and plan accordingly.
```

**Don't do:**
- Use rigid "✅ Good" / "❌ Bad" format
- Make lists without explanations
- Use abstract tips
- Be prescriptive without context

**Do:**
- Write conversationally
- Explain WHY each tip matters
- Provide examples
- Mix paragraphs with short bullets when helpful

---

### Summary Sections

**Purpose:** Reinforce key takeaways in a conversational way.

**Structure:**
```markdown
## [Varied Heading]

[Paragraph summarizing key insight]

[Another paragraph with example or application]

[Final encouraging statement]
```

**Guidelines:**
- Use varied headings: "What to Remember," "Key Insights," "The Main Point," "Bringing It Home"
- Don't use formulaic "Key Takeaway" every time
- Write as paragraphs, not bullet lists
- Make it conversational and encouraging
- End with an encouraging statement

**Example:**
```markdown
## What to Remember

The distinction between parts and relationships might seem subtle, but it's crucial. Parts tell you what exists in your system—users, systems, containers, components. Relationships tell you how things work together—data flows, API calls, event streams.

A simple rule of thumb: If you can't describe how two components interact, you don't really understand the system yet.

When you model in Sruja, focus on getting both right: identify the parts accurately, then label their relationships with specific protocols and actions. That's when diagrams go from informative to actionable.
```

**Don't do:**
- Use formulaic "Key Takeaway" heading
- Write as bullet list: "1. Point 1. 2. Point 2."
- Be abstract or theoretical
- End abruptly

**Do:**
- Use varied headings
- Write as conversational paragraphs
- Make it encouraging
- Connect to reader's work

---

### Quiz Sections

**Purpose:** Test understanding with varied, engaging questions.

**Structure:**
```markdown
## Check Your Understanding

Let's see if this is clicking.

### Quick Check

[Question 1 - varied format]

[Question 2 - varied format]

---

### Answers & Discussion

[Answer 1 with explanation]

[Answer 2 with explanation]
```

**Guidelines:**
- Always have exactly 2 questions per lesson
- Vary question formats between questions
- Include at least one scenario-based question
- Include at least one open-ended/reflection question
- Write explanations that teach, not just state answer
- Use "Answers & Discussion" section instead of "Check Answer" buttons
- Make explanations conversational and informative

**Question types to use:**
- Scenario-based ("You're [situation]. What should [action]?")
- Multiple choice with varied phrasing
- Open-ended reflection ("Think about [topic]...")
- Application questions ("In your current project...")

**Example:**
```markdown
## Check Your Understanding

Let's see if flows make sense to you.

### Quick Check

**1. You're troubleshooting a slow user registration process. Should you start with a static diagram or a flow, and why?**

[ ] A. Static diagram - shows components and connections
[ ] B. Flow - shows sequence of operations and where delays occur
[ ] C. Both are equally valid
[ ] D. Neither - just read the code

**2. Think about your current project. Can you identify one flow that's critical to understanding the system? Maybe a user registration flow, or a payment processing flow? What steps are involved?**

Take a moment to sketch it out mentally. There's no right answer here—the important part is practicing the skill of seeing flows.

---

### Answers & Discussion

**1. B. Flow - shows sequence of operations and where delays occur** – Static diagrams help you understand what components exist and how they're connected. But they don't show timing, sequences, or bottlenecks. Flows show the complete journey and where time is spent. To troubleshoot performance, you need to see the sequence of operations and identify where delays occur.

**2. (Your flow)** – There's no wrong answer here! The important part is starting to notice flows in your own work. If you're not sure where to start, pick something a user does (sign up, buy something, search) and trace it through your system.
```

**Don't do:**
- Have more than 2 questions
- Use identical format for all questions
- Use "Check Answer" button format
- Provide answers without explanations
- Be overly technical in questions

**Do:**
- Exactly 2 questions per lesson
- Vary question formats
- Include at least one open-ended question
- Provide detailed explanations
- Make questions test understanding, not memorization

---

### "What's Next" Sections

**Purpose:** Provide closure and momentum for next lesson.

**Structure:**
```markdown
## What's Next

Now that you understand [current topic], let's explore [next topic]. This will help you [benefit].
```

**Guidelines:**
- Always include at the end of every lesson
- Preview the next lesson's topic
- Explain why the next topic matters
- Show how it connects to current lesson
- Keep it brief (2-3 sentences)

**Example:**
```markdown
## What's Next

Now that you understand flows and how to model them, let's explore how systems respond and adapt over time. In the next lesson, we'll dive into feedback loops—how systems have natural cycles that enable self-regulation, learning, and adaptation.

This will help you understand why some systems improve over time while others spiral down, and how to design systems that get better rather than worse.
```

**Don't do:**
- Skip this section
- Be too long or detailed
- Don't explain the connection
- Sound like a teaser

**Do:**
- Always include
- Preview next topic
- Explain the connection
- Keep it brief and helpful

---

## Heading Variations

**Never use:** "What is/What Are [Topic]?" for every lesson heading.

**Alternatives to use:**

For introducing concepts:
- "Understanding [Topic]"
- "[Topic] in Practice"
- "Getting Started with [Topic]"
- "How [Topic] Works"
- "[Topic]: The Basics"
- "The Big Picture: [Topic]"

For diving deeper:
- "[Topic] in Action"
- "Putting [Topic] to Work"
- "When to Use [Topic]"
- "Common [Topic] Patterns"
- "Mastering [Topic]"

For examples:
- "Real-World Example"
- "[Topic] in Everyday Life"
- "A Concrete Example"
- "[Topic] at Work"
- "Let's See It in Action"

For summaries:
- "What to Remember"
- "Key Insights"
- "The Main Point"
- "Bringing It Home"
- "[Topic] in a Nutshell"

---

## Transition Phrases

**Use these to make sections flow naturally:**

**Between sections:**
- "Now that we've covered [X], let's look at [Y]"
- "This raises an important question: [Y]"
- "Here's where [X] gets interesting:"
- "Let's see this in practice with an example"
- "So far, we've focused on [X]. Now, consider [Y]"
- "Building on this, let's explore..."
- "This brings us to another important concept"

**Before examples:**
- "To make this concrete, let's look at..."
- "Here's a real-world example:"
- "Consider this scenario:"
- "Let's apply this to something you've probably seen:"
- "An example will help clarify this:"
- "I've seen this pattern play out many times:"

**After examples:**
- "This example shows..."
- "What's happening here?"
- "Notice how..."
- "The key insight is..."
- "This pattern is common because..."
- "Here's why this matters:"

**Before activities:**
- "Let's try this:"
- "Here's something to think about:"
- "Time to apply what you've learned:"
- "Your turn:"
- "Let's put this into practice:"

---

## Quiz Formatting Guidelines

**Always follow these rules for quizzes:**

### 1. Question Count
- Exactly 2 questions per lesson
- Never more, never less
- No questions marked as "[REMOVED]"

### 2. Question Formats
Vary formats between the two questions:

**Format A - Scenario-based:**
```markdown
**1. You're [situation]. Based on this lesson, what should [action]?**

[ ] A. Option 1
[ ] B. Option 2
[ ] C. Option 3
[ ] D. Option 4
```

**Format B - Application:**
```markdown
**2. In your current project, [application question].**

[ ] A. Option 1
[ ] B. Option 2
[ ] C. Option 3
[ ] D. Option 4
```

**Format C - Concept check:**
```markdown
**1. Which of the following best describes [concept]?**

[ ] A. Description 1
[ ] B. Description 2
[ ] C. Description 3
[ ] D. Description 4
```

### 3. Include Open-Ended Questions
At least one of the two questions should be open-ended or reflection-based:

```markdown
**2. Think about a system you've worked on recently. Can you identify [something]?**

Take a moment to reflect. There's no single right answer—this is about building awareness and intuition.
```

### 4. Answer Format
Always use "Answers & Discussion" section:

```markdown
---

### Answers & Discussion

**1. [Letter]. [Answer]** – [Detailed explanation that teaches the concept, not just states the answer].
```

**Guidelines for explanations:**
- Start with answer letter
- Provide detailed explanation
- Explain WHY the answer is correct
- Explain WHY other options are incorrect
- Make it conversational and informative
- Teach the concept, don't just give the answer

### 5. Don't Use Button Format
Never use the "Check Answer" button format:
```markdown
❌ Don't do this:
<button class="check-answer-btn" data-correct="b">Check Answer</button>
```

Always use "Answers & Discussion" section instead:
```markdown
✅ Do this:
---

### Answers & Discussion

**1. B. [Answer]** – [Explanation]
```

---

## Common Pitfalls to Avoid

### 1. Starting with Definitions
**Pitfall:** Jumping straight into "What is X?" or defining concepts without context.

**Solution:** Always start with a hook—question, story, or scenario.

**Example of pitfall:**
```markdown
❌ Don't do this:
## What is Systems Thinking?

Systems thinking is a holistic approach to understanding how components interact...
```

**Example of solution:**
```markdown
✅ Do this instead:
## Understanding Systems Thinking

Have you ever fixed a bug, tested it thoroughly, and celebrated—only to have three new bugs appear the next day?

These aren't just frustrating experiences. They're symptoms of looking at problems the wrong way...

Systems thinking is about understanding how components interact holistically.
```

---

### 2. Using "What is/What Are" Headings
**Pitfall:** Every lesson uses identical "What is X?" or "What Are Y?" heading.

**Solution:** Use varied, engaging headings.

**Examples of varied headings:**
- "Understanding X"
- "X in Practice"
- "How X Works"
- "Getting Started with X"
- "X: The Basics"
- "The Big Picture: X"

---

### 3. Over-Using Bullet Points
**Pitfall:** Writing everything as bullet lists.

**Solution:** Write explanatory content in paragraphs, use bullets for actual lists.

**Example of pitfall:**
```markdown
❌ Don't do this:
## Why It Matters

Systems thinking helps you:
- See the whole system
- Understand relationships
- Find root causes
- Improve design

It also helps you:
- Avoid local optimization
- Consider context
- Plan for failure
```

**Example of solution:**
```markdown
✅ Do this instead:
## Why It Matters

Systems thinking helps you see the whole system, not just parts. When you understand relationships and context, you can find root causes of problems instead of just treating symptoms. This leads to better architectural decisions that work in practice, not just in theory.

The key insight is avoiding local optimization. When you optimize one component in isolation, you might make the whole system worse. Systems thinking forces you to consider the whole picture every time.
```

---

### 4. Using "Key Takeaways" Every Time
**Pitfall:** Every lesson ends with "## Key Takeaway" followed by bullet points.

**Solution:** Use varied headings and write as paragraphs.

**Examples of varied headings:**
- "What to Remember"
- "Key Insights"
- "The Main Point"
- "Bringing It Home"
- "[Topic] in a Nutshell"

**Example of pitfall:**
```markdown
❌ Don't do this:
## Key Takeaways

- Relationships matter more than parts
- Design for the whole system
- Consider edge cases and failures
- Think in flows and feedback
```

**Example of solution:**
```markdown
✅ Do this instead:
## What to Remember

The core idea from this lesson is simple but powerful: Never design in isolation.

Every application exists in a context—people who use it, dependencies it relies on, processes that keep it running. When you ignore these layers, you build fragile systems that work in theory but fail in practice.

The key is: see systems holistically—all the layers together—or you'll keep solving the wrong problems.
```

---

### 5. Over-Structuring with "Good/Bad"
**Pitfall:** Using rigid "✅ Good" / "❌ Bad" format for best practices.

**Solution:** Write conversationally with explanations.

**Example of pitfall:**
```markdown
❌ Don't do this:
## Best Practices

✅ Good: Label relationships with specific protocols
❌ Bad: Use vague labels like "uses"

✅ Good: Show external dependencies
❌ Bad: Assume everything is internal
```

**Example of solution:**
```markdown
✅ Do this instead:
## Practical Tips

Be specific with relationships
Instead of writing `API → DB "uses"`, try `API → DB "PostgreSQL/reads"`. The difference might seem small, but it's huge in practice. Specific labels mean developers can look at your diagram and immediately understand what's actually happening—they don't have to guess or ask questions.

Mark external systems clearly
When you depend on something outside your control, make that visible. This isn't just for documentation—it helps everyone understand risk and plan accordingly.
```

---

### 6. Having Too Many Quiz Questions
**Pitfall:** Lessons have 3, 4, or 5+ questions.

**Solution:** Always have exactly 2 questions per lesson.

**Why:**
- 2 questions is enough to test understanding
- More feels tedious and repetitive
- Forces you to choose the most important concepts to test
- Keeps lessons focused

**Exception:** End-of-module quizzes can have 4-5 questions covering all lessons in the module.

---

## Quality Checklist

Before marking a lesson complete, verify:

### Structure
- [ ] Frontmatter summary is engaging (hook question, not just description)
- [ ] Uses "Learning Goals" (plural)
- [ ] Main heading varies (not "What is/What Are [Topic]?")
- [ ] Section headings vary appropriately
- [ ] Has conversational transitions between major sections
- [ ] Mixes paragraphs and bullet points naturally
- [ ] Section lengths vary (not perfectly symmetric)
- [ ] Has "What's Next" section linking to next lesson

### Content
- [ ] Starts with a hook (question, story, or relatable scenario)
- [ ] Explains concept fully before showing Sruja code
- [ ] Uses concrete, real-world examples
- [ ] Explains *why* something matters, not just *what* it is
- [ ] Avoids over-structuring simple explanations in code blocks
- [ ] Sruja code examples are accurate

### Voice & Tone
- [ ] Uses second person ("you") consistently
- [ ] Has 3+ rhetorical questions
- [ ] Has conversational tone (not academic or overly formal)
- [ ] Avoids AI-typical phrases ("In conclusion," "Furthermore," "It's important to note")
- [ ] Uses first-person occasionally ("Let me share...", "I've seen...")
- [ ] Has encouraging language

### Quiz
- [ ] Exactly 2 questions
- [ ] Question formats vary
- [ ] Includes at least one open-ended/reflection question
- [ ] Uses "Answers & Discussion" section
- [ ] Explanations feel natural, not robotic
- [ ] Answers provide context, not just the letter
- [ ] Tests understanding, not memorization

### Overall
- [ ] Would a human mentor speak this way?
- [ ] Does it feel like a conversation, not a lecture?
- [ ] Is there variety in structure and tone?
- [ ] Would I want to learn from this?

---

## Final Notes

**Goal:** Create content that feels like a human mentor explaining concepts, not an AI generating facts.

**Remember:**
- Variety is good—natural human writing isn't perfectly consistent
- Voice matters more than format—write like you're teaching a colleague
- Examples and stories make concepts stick—use them liberally
- Questions engage readers—ask them often
- Transitions make flow smooth—connect your sections naturally
- Clarity trumps cleverness—be clear first, clever second

**The difference between AI-generated and human-written:**
- AI: Perfect structure, identical patterns, no personality
- Human: Varied structure, natural flow, personal touches

**Your job as a humanizer:** Take the good technical content and give it a human voice. Don't change the facts—change the delivery.

**You've got this!** Happy writing!