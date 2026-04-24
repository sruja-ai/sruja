# Grill Me - Shared Understanding Skill

This skill ensures AI and human share a design concept before writing code.

## Description

Use this skill when planning new features or significant changes. The AI will interview you relentlessly to reach shared understanding.

## Usage

```bash
# Activate skill
@skill:grill-me

# Or use in prompt
"Grill me on this plan until we reach shared understanding."
```

## Process

1. You describe what you want to build
2. AI asks questions to clarify:
   - What problem does this solve?
   - Who is the user?
   - What are the success criteria?
   - What are the dependencies?
   - What could go wrong?
   - How do you measure success?
3. Continue until AI has no more questions
4. Document the shared understanding as PRD or issues

## Questions AI Will Ask

### Understanding Phase
- What exactly are you trying to build?
- Why is this important?
- Who will use this?
- What happens if we don't build it?

### Requirements Phase
- What are the acceptance criteria?
- What are the edge cases?
- What are the error conditions?
- What are the performance requirements?

### Architecture Phase
- What does this depend on?
- What will depend on this?
- Where does this fit in the architecture?
- What are the boundary conditions?

### Implementation Phase
- How will you verify this works?
- What could go wrong in production?
- What is the rollback plan?
- How do you measure success?

## Example

```
You: I want to add a feature that shows code complexity
AI: What problem does this solve?
You: Developers don't understand their code quality
AI: What specifically do you want to measure?
You: Cyclomatic complexity
AI: What threshold indicates "too complex"?
You: 10
AI: What happens when it exceeds threshold?
AI: [continues until full understanding]
```

## Output

After grilling, produce:
- Product Requirements Document (PRD)
- Or direct to issues/tasks
- Never start coding until shared understanding reached

## Why This Matters

From the talks: "Me and the AI don't share a design concept."
- Without shared understanding, AI produces garbage
- Re-running compiler just produces more garbage
- Solution: reach shared understanding first