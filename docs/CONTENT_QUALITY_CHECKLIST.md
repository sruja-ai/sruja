# Content Quality Checklist

Use this checklist before publishing any course, tutorial, blog, or doc. It raises clarity, correctness, and consistency.

## Global (All Content)
- [ ] `title` is clear, specific, and audience-appropriate
- [ ] `summary` explains value in 1–2 sentences
- [ ] Structure uses clear headings; no wall-of-text sections
- [ ] Links resolve; internal slugs match site paths
- [ ] All code blocks compile or run
- [ ] Images include descriptive `alt` text
- [ ] Avoid jargon or define it in context; include a glossary link if needed
- [ ] Inclusive, concise language; active voice
- [ ] Uses consistent terminology from `docs/LANGUAGE_SPECIFICATION.md`
- [ ] Ran `make content-validate` and fixed warnings

## Courses
- [ ] Course overview explains audience, prerequisites, and outcomes
- [ ] Each module lists learning objectives and prerequisites
- [ ] Each lesson starts with objectives and ends with key takeaways
- [ ] Includes at least one hands-on exercise per module
- [ ] Provides “What’s next” guidance to related modules/tutorials
- [ ] Difficulty set consistently across modules

## Tutorials
- [ ] Clear problem statement and expected result
- [ ] Step-by-step instructions with numbered steps
- [ ] Shows before/after or visual confirmation when applicable
- [ ] Troubleshooting tips for common failure points
- [ ] “Next steps” links to deeper docs or related tutorials
- [ ] Tags added for discoverability (e.g., `modeling`, `validation`)

## Blog Posts
- [ ] Strong introduction framing the topic and relevance
- [ ] Concrete examples or case studies (screenshots ok)
- [ ] Clear conclusion with a takeaway or call-to-action
- [ ] `pubDate`, authors, and tags set appropriately
- [ ] Avoids overly promotional tone; focuses on teaching

## Documentation Pages
- [ ] Overview explains the concept in one short paragraph
- [ ] Examples are minimal, correct, and copy/pasteable
- [ ] Cross-links to related concepts and tutorials
- [ ] Notes section for edge cases and pitfalls
- [ ] Consistent terminology and naming

## Accessibility & UX
- [ ] Headings form a logical outline (H1 → H2 → H3)
- [ ] Lists are scannable; avoid long paragraphs
- [ ] Tables include headers; avoid complex tables when possible
- [ ] No reliance on color alone to convey meaning

## Definition of Done
- [ ] `make content-validate` passes
- [ ] All TODO placeholders removed
- [ ] Code blocks validated locally where applicable
- [ ] Internal links tested locally (`apps/website` dev)

Tip: Prefer concrete examples and visuals over abstract descriptions. If a reader can replicate the outcome in 5–10 minutes, it’s a great learning resource.
