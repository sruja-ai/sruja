# Content Contribution Guide

Complete guide for contributing content to the Sruja docs site (mdBook in `book/`).

## Quick Start

```bash
# Install mdBook (once)
make book-deps   # or: cargo install mdbook mdbook-mermaid

# Add or edit content under book/src/ (see Content Structure below)
# Then build and verify
make book-build
make book-serve  # optional: preview at http://localhost:3000
```

## Content Types

### Courses
Structured learning paths with modules and lessons:
- **Structure**: Course → Modules → Lessons
- **Use Case**: Comprehensive guides on specific topics

### Tutorials
Step-by-step how-to guides:
- **Structure**: Single page with steps
- **Use Case**: "How to do X" guides

### Documentation Pages
Reference and concepts:
- **Structure**: Single page or nested under `book/src/docs/`
- **Use Case**: API reference, concepts, adoption guides

### Challenges
Hands-on exercises:
- **Structure**: Single page with goal and optional hints
- **Location**: `book/src/challenges/`

## Content Structure (mdBook)

All user-facing content lives under **`book/src/`**:

```
book/src/
├── courses/
│   └── course-name/
│       ├── course-overview.md
│       └── module-N-name/
│           ├── module-overview.md
│           └── lesson-1.md, lesson-2.md, ...
├── tutorials/
│   ├── overview.md
│   ├── basic/
│   │   └── tutorial-name.md
│   └── advanced/
│       └── tutorial-name.md
├── docs/
│   ├── concepts/
│   ├── reference/
│   └── ... (intro, getting-started, adoption, etc.)
├── challenges/
│   ├── overview.md
│   └── challenge-name.md
├── reference/
│   └── cli.md, language.md, language-spec.md
└── SUMMARY.md   # ← Add new pages here for navigation
```

## Creating Content

### Adding a Tutorial

1. Create a new `.md` file under `book/src/tutorials/basic/` or `book/src/tutorials/advanced/`.
2. Add frontmatter (e.g. `title`, `weight`, `summary`, `tags`) and content.
3. Add an entry in `book/src/SUMMARY.md` under the Tutorials section.
4. Optionally add a link in `book/src/tutorials/overview.md`.
5. Run `make book-build` to verify.

### Adding a Course or Module

1. Create a directory under `book/src/courses/<course-name>/` (e.g. `module-2-new-topic/`).
2. Add `module-overview.md` and `lesson-1.md`, `lesson-2.md`, etc.
3. Add entries in `book/src/SUMMARY.md` under the course.
4. Link from the course’s `course-overview.md` and from the previous module’s “Next” section.
5. Run `make book-build` to verify.

### Adding a Challenge

1. Create `book/src/challenges/challenge-name.md`.
2. Add to `book/src/SUMMARY.md` and to `book/src/challenges/overview.md`.
3. Run `make book-build` to verify.

## Frontmatter Guidelines

Book pages can use YAML frontmatter for title and metadata (mdBook does not require a fixed schema):

```yaml
---
title: "Your Title"
weight: 10
summary: "Brief description (1-2 sentences)."
tags: ["cli", "getting-started"]
difficulty: "beginner"
---
```

### Suggested Fields

- `title` - Page title
- `summary` - Short description (used in some themes)
- `weight` - Ordering (lower = earlier in sidebar)
- `tags` - For discoverability (e.g. in tutorials overview)
- `difficulty` - For tutorials/courses: `beginner`, `intermediate`, `advanced`

## Content Best Practices

### Naming Conventions

- Use kebab-case for file names (e.g., `lesson-1.md`, `system-design-101.md`)
- Use descriptive names that reflect the content
- Keep names concise but clear

### Content Structure

**Courses:**
- Start with engaging introduction
- List learning objectives
- Provide clear module descriptions
- Include prerequisites

**Lessons:**
- Begin with learning objectives
- Use clear headings and sections
- Include examples and code snippets
- Add "Sruja Perspective" section when relevant
- End with summary or key takeaways

**Tutorials:**
- Use step-by-step format
- Include code examples
- Provide clear instructions
- Add troubleshooting tips

**Blog Posts:**
- Write engaging introductions
- Use clear structure (headings, lists)
- Include relevant examples
- End with conclusion

### Linking Content

Use relative Markdown links from `book/src/`. Paths are relative to the current file or book root:

```markdown
[Beginner path](../docs/beginner-path.md)
[CLI basics](basic/cli-basics.md)
[Module 1](module-1-fundamentals/module-overview.md)
```

See existing tutorials and course lessons for examples.

## Validation

Before committing, build the book to catch broken links and errors:

```bash
make book-build
```

Optionally serve locally to check navigation and links:

```bash
make book-serve
```

## Troubleshooting

### Broken links in mdBook

Check built output in `book/book/`. Use consistent relative paths (e.g. from `tutorials/basic/`, use `../advanced/deployment-modeling.md` for another tutorial).

### Page not showing in sidebar

Every new page must be listed in `book/src/SUMMARY.md` in the correct section.

### Build fails

Run `mdbook build` from the `book/` directory to see the exact error (often a broken link or invalid Markdown).

## Complete Workflow

1. Add or edit files under `book/src/` (tutorials, courses, docs, challenges).
2. Update `book/src/SUMMARY.md` so new pages appear in the sidebar.
3. Update overview or course-overview pages with links to new content.
4. Run `make book-build` and fix any errors.
5. Optionally run `make book-serve` to preview.
6. Commit and push.

## Additional Resources

- [mdBook Documentation](https://rust-lang.github.io/mdBook/) - mdBook format and commands
- [CONTENT_STYLE_GUIDE.md](CONTENT_STYLE_GUIDE.md) - Voice, structure, and Sruja conventions
- [CONTENT_QUALITY_CHECKLIST.md](CONTENT_QUALITY_CHECKLIST.md) - Checklist before publishing

