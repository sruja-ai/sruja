# Course Quizzes

This directory contains interactive quizzes for Sruja courses using the [mdbook-quiz](https://github.com/cognitive-engineering-lab/mdbook-quiz) preprocessor.

## Directory Structure

```
quizzes/
├── README.md                           # This file
└── system-design-101/                  # Course-specific directories
    ├── module-1-fundamentals/
    │   ├── lesson-1-quiz.toml         # Quiz for Module 1, Lesson 1
    │   └── lesson-2-quiz.toml         # Quiz for Module 1, Lesson 2
    ├── module-2-building-blocks/
    │   └── (quiz files for Module 2)
    └── ...
```

## Creating a New Quiz

### 1. Choose a Question Type

mdbook-quiz supports three question types:

#### **Short Answer**
User provides a one-line text answer.

```toml
[[questions]]
type = "ShortAnswer"
prompt.prompt = "What is the keyword for declaring a variable in Rust?"
answer.answer = "let"
answer.alternatives = ["let keyword", "variable"]
context = "For example, you can write: `let x = 1`"
```

**Fields:**
- `prompt.prompt`: The question text (Markdown)
- `answer.answer`: The exact correct answer
- `answer.alternatives`: List of alternative acceptable answers (optional)
- `context`: Explanation shown after answering (optional, Markdown)

#### **Multiple Choice**
User selects one option from a list.

```toml
[[questions]]
type = "MultipleChoice"
prompt.prompt = "What does it mean if a variable `x` is immutable?"
prompt.distractors = [
  "`x` is stored in the immutable region of memory.",
  "After being defined, `x` can be changed at most once.",
  "You cannot create a reference to `x`."
]
answer.answer = "`x` cannot be changed after being assigned to a value."
context = "Immutable means 'not mutable', or not changeable."
```

**Fields:**
- `prompt.prompt`: The question text (Markdown)
- `prompt.distractors`: Array of incorrect answers (Markdown)
- `prompt.answerIndex`: If set, don't randomize and put answer at this index (optional)
- `answer.answer`: The correct answer (Markdown)
- `context`: Explanation shown after answering (optional, Markdown)

#### **Tracing**
User predicts how a program will execute (or fail to compile).

```toml
[[questions]]
type = "Tracing"
prompt.program = """
fn main() {
  let x = 1;
  println!("{x}");
  x += 1;
  println!("{x}");
}
"""
answer.doesCompile = false
context = "This is a compiler error because line 4 tries to mutate `x` when `x` is not marked as `mut`."
```

**Fields:**
- `prompt.program`: The source code to trace
- `answer.doesCompile`: `true` if program should compile, `false` if not
- `answer.stdout`: Expected output if `doesCompile = true` (optional)
- `context`: Explanation shown after answering (optional, Markdown)

### 2. Create a Quiz File

Create a new `.toml` file in the appropriate directory:

```bash
# Example: Create quiz for Module 1, Lesson 3
touch sruja/book/src/quizzes/system-design-101/module-1-fundamentals/lesson-3-quiz.toml
```

### 3. Add Questions

Add multiple questions to the quiz file:

```toml
# Quiz for System Design 101 - Module 1, Lesson 3: The C4 Model

[[questions]]
type = "ShortAnswer"
prompt.prompt = "In the C4 model, what diagram shows the highest-level view of a software system in its environment?"
answer.answer = "Context"
answer.alternatives = ["System Context", "Context diagram"]
context = "The System Context diagram shows your system as a black box in the center, surrounded by users and external systems."

[[questions]]
type = "MultipleChoice"
prompt.prompt = "Which C4 diagram shows how containers within a system interact with each other?"
prompt.distractors = [
  "System Context diagram",
  "Component diagram",
  "Deployment diagram"
]
answer.answer = "Container diagram"
context = "The Container diagram zooms into a single system to show the containers (applications, databases, etc.) and how they interact."
```

### 4. Reference Quiz in Lesson

Add the quiz to the corresponding lesson file:

```markdown
## Quiz: Test Your Knowledge

Ready to apply what you've learned? Take the interactive quiz for this lesson!

{{#quiz ../../quizzes/system-design-101/module-1-fundamentals/lesson-3-quiz.toml}}

This quiz covers:
- System Context diagrams
- Container diagrams
- Component diagrams
- Practical diagramming scenarios
```

**Important:** The path is relative to the lesson file. Use `../` to navigate up to the `quizzes` directory.

## Installation & Setup

### 1. Install mdbook-quiz

```bash
# Install from crates.io
cargo install mdbook-quiz --locked

# Or install a specific version (recommended for stability)
cargo install mdbook-quiz --locked --version 0.4.0
```

### 2. Verify Installation

```bash
mdbook-quiz -V
```

### 3. Build the Book

```bash
# Navigate to the book directory
cd sruja/book

# Build with quizzes
mdbook build

# Or serve locally with live reload
mdbook serve
```

## Configuration

The mdbook-quiz preprocessor is configured in `book.toml`:

```toml
[preprocessor.quiz]
```

Additional configuration options can be added:

```toml
[preprocessor.quiz]
# Make quizzes take up full screen
fullscreen = false

# Cache answers in localStorage
cache-answers = true

# Run spellchecker on quiz text
spellcheck = false
```

## Best Practices

### 1. **Quiz Length**
- **5-15 questions** per quiz is optimal
- Too many questions can overwhelm learners
- Focus on key concepts rather than comprehensive coverage

### 2. **Question Variety**
- Mix **Short Answer**, **Multiple Choice**, and **Tracing** questions
- Balance **recall** (Short Answer) and **application** (Multiple Choice)
- Include **real-world scenarios** and **case studies**

### 3. **Quality Explanations**
- Always provide **context** for each question
- Explain **why** the answer is correct
- Add **links** to relevant sections or external resources
- Use **examples** from real companies or systems

### 4. **Practical Focus**
- Base questions on **real-world scenarios**
- Include **specific numbers and metrics** from production systems
- Test **decision-making** skills, not just memorization
- Use **trade-off scenarios** that require reasoning

### 5. **Progressive Difficulty**
- Start with **easier questions** to build confidence
- Include **challenging scenarios** for advanced learners
- Cover **foundational concepts** before advanced topics

### 6. **Real-World Examples**

**Good:**
```toml
prompt.prompt = "Netflix moved from a single datacenter to cloud infrastructure in 2011. What was the primary reason for this architectural change?"
```

**Less Good:**
```toml
prompt.prompt = "What is the benefit of horizontal scaling?"
```

**Why:** The specific example helps learners understand the context and practical application.

## Naming Conventions

### Quiz Files
- Format: `lesson-{number}-quiz.toml`
- Example: `lesson-1-quiz.toml`, `lesson-2-quiz.toml`

### Directory Structure
```
quizzes/
└── {course-name}/           # kebab-case: system-design-101, agentic-ai
    └── module-{name}/       # kebab-case: module-1-fundamentals, module-2-patterns
        └── lesson-{number}-quiz.toml
```

### Question Organization
Group related questions together:
```toml
# Section: Scaling Concepts
[[questions]]
type = "MultipleChoice"
...

[[questions]]
type = "MultipleChoice"
...

# Section: Real-World Scenarios
[[questions]]
type = "ShortAnswer"
...
```

## Testing Quizzes

### 1. Local Testing
```bash
cd sruja/book
mdbook serve
# Open http://localhost:3000 in your browser
```

### 2. Verify All Quizzes
```bash
# Build the book to check for syntax errors
mdbook build
```

### 3. User Testing
- Have colleagues take the quizzes
- Gather feedback on question clarity
- Adjust difficulty based on completion rates
- Update explanations based on common misconceptions

## Troubleshooting

### Quiz Not Rendering
**Problem:** Quiz appears as raw markdown `{{#quiz ...}}`

**Solution:**
1. Verify mdbook-quiz is installed: `mdbook-quiz -V`
2. Check `book.toml` has `[preprocessor.quiz]`
3. Build fresh: `mdbook build`

### Path Not Found
**Problem:** Quiz file not found error

**Solution:**
1. Verify the quiz file exists
2. Check the path in the lesson file is correct (relative to lesson)
3. Use `../` to navigate up directories

### Answer Not Accepted
**Problem:** Correct answer marked as wrong

**Solution:**
1. Check for typos in `answer.answer`
2. Add more `alternatives` for acceptable variations
3. For Short Answer, case matters unless configured otherwise

## Contributing

When adding quizzes:

1. **Test thoroughly** - Take the quiz yourself multiple times
2. **Get feedback** - Have peers review questions
3. **Document context** - Ensure every question has a good explanation
4. **Real-world focus** - Use actual industry examples and metrics
5. **Progressive difficulty** - Balance easy and hard questions

## Resources

- [mdbook-quiz Documentation](https://github.com/cognitive-engineering-lab/mdbook-quiz)
- [mdbook Documentation](https://rust-lang.github.io/mdBook/)
- [Quiz Schema](https://github.com/cognitive-engineering-lab/mdbook-quiz/blob/main/mdbook-quiz.schema.json)

## Example Quizzes

For reference, see the existing quizzes:

- `system-design-101/module-1-fundamentals/lesson-1-quiz.toml` - Functional vs Non-functional requirements, trade-offs
- `system-design-101/module-1-fundamentals/lesson-2-quiz.toml` - Scaling strategies, latency vs throughput

These examples demonstrate:
- Mix of question types
- Real-world case studies (Netflix, Healthcare.gov, Instagram)
- Practical scenarios and decision-making
- Comprehensive explanations and context
- Progressive difficulty