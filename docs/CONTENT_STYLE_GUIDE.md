# Content Style Guide

Consistent voice and structure across courses, tutorials, blogs, and docs.

## Voice & Tone
- Friendly, direct, and practical; avoid hype
- Prefer active voice and short sentences
- Teach through concrete examples first, theory second

## Structure
- Start with context: problem, audience, outcome
- Use H2/H3 headings to group ideas; keep sections short
- End with summary and “what’s next” links

## Code Blocks
- Use fenced blocks with language tags: ` ```sruja `, ` ```bash `, ` ```ts `
- Keep blocks minimal; one idea per block
- Ensure copy/paste works; no ellipses or placeholders in runnable code

## Sruja DSL Conventions
- Use descriptive names for `architecture`, `system`, `container`, `component`
- Prefer consistent casing and hyphenated IDs
- Show relationships with meaningful labels

## Writing Patterns
- Learning objectives up front for lessons
- Step-by-step for tutorials (numbered steps)
- Examples → explanation → takeaway for docs
- Introductions → examples → conclusion for blogs

## Naming & Terminology
- Use consistent terms from `docs/LANGUAGE_SPECIFICATION.md`
- Prefer plain English over acronyms; define acronyms on first use

## Admonitions
- Use callouts sparingly: Note, Tip, Warning
- Keep them short (≤3 lines) and actionable

## Links
- Prefer relative site paths (e.g., `/tutorials/...`) over external when possible
- Link to next logical step at end of each piece

## Do / Don’t
- Do: show clear outcomes, small steps, real-world context
- Don’t: overuse abstract descriptions, large unscannable paragraphs

## Accessibility
- Use descriptive `alt` text
- Headings form a logical outline
- Avoid relying on color alone

## Review Process
- Run `make content-validate`
- Use `docs/CONTENT_QUALITY_CHECKLIST.md` before publish
