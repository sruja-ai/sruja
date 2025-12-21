# Content Contribution Guide

Complete guide for contributing content to the Sruja website (Astro-based).

## Quick Start

```bash
# Create content using Make commands
make content-tutorial NAME="my-tutorial"
make content-blog NAME="my-blog-post"
make content-course NAME="my-course"

# Validate all content
make content-validate
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

### Blog Posts
News, updates, and articles:
- **Structure**: Single page with date
- **Use Case**: Announcements, thoughts, case studies

### Documentation Pages
Reference documentation:
- **Structure**: Single page or nested pages
- **Use Case**: API reference, concepts, guides

## Creating Content

### Using Make Commands (Recommended)

```bash
# Courses
make content-course NAME="course-name"
make content-module COURSE="course-name" NAME="module-name"
make content-lesson COURSE="course-name" MODULE="module-name" NAME="lesson-name"

# Other content
make content-tutorial NAME="tutorial-name"
make content-blog NAME="blog-post-name"
make content-doc NAME="doc-name"

# Validation
make content-validate
```

### Using Go Scripts Directly

```bash
go run scripts/content-generator/main.go <command> [args]
go run scripts/content-validator/main.go
```

## Content Structure

```
apps/website/src/content/
├── courses/
│   └── course-name/
│       └── module-name/
│           ├── module-overview.md  # Module landing page
│           └── lesson-1.md         # Lesson content
├── tutorials/
│   └── tutorial-name.md
├── blog/
│   └── YYYY-MM-DD-post-name.md
└── docs/
    └── doc-name.md
```

## Workflow Examples

### Creating a Course

```bash
# 1. Create course
make content-course NAME="system-design-301"

# 2. Create modules
make content-module COURSE="system-design-301" NAME="fundamentals"

# 3. Create lessons
make content-lesson COURSE="system-design-301" MODULE="fundamentals" NAME="lesson-1"

# 4. Edit generated files
# 5. Validate
make content-validate
```

### Adding a Blog Post

```bash
# 1. Create blog post
make content-blog NAME="announcing-sruja-v1"

# 2. Edit the generated file
# 3. Set draft: false when ready
# 4. Validate
make content-validate
```

## Frontmatter Guidelines

All content files require frontmatter at the top. Astro uses content collections with schema validation:

```yaml
---
title: "Your Title"
summary: "Brief description (1-2 sentences)"
weight: 1  # Optional: For menu ordering (lower = earlier)
---
```

### Required Fields

- `title` - Human-readable title (required by Astro schema)

### Optional Fields

**For all content:**
- `summary` - Brief description (1-2 sentences)
- `weight` - For menu ordering (lower = earlier)
- `description` - Extended description

**For blog posts:**
- `pubDate` - Publication date (auto-set to current date by generator)
- `authors` - Array of author objects
- `tags` - Array of tag strings

**For courses/tutorials:**
- `difficulty` - `beginner`, `intermediate`, or `advanced`
- `topic` - Topic category
- `estimatedTime` - Estimated reading time

Note: Astro validates frontmatter against the schema defined in `apps/website/src/content/config.ts`

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

Use standard Markdown links for internal content:

```markdown
[Link to lesson](/courses/course-name/module-name/lesson-1)
[Link to module](/courses/course-name/module-name)
[Link to tutorial](/tutorials/tutorial-name)
```

Or use Astro's `getCollection` API in components for dynamic linking.

## Validation

Always validate content before committing:

```bash
make content-validate
```

The validator checks:
- ✅ Frontmatter format
- ✅ Required fields (title, summary)
- ✅ File structure
- ⚠️  TODO placeholders (warns but doesn't fail)

## Troubleshooting

### "Course already exists"
The generator prevents overwriting. Delete the existing directory/file and recreate, or use a different name.

### "Module does not exist"
Create the parent course/module first:
```bash
make content-course NAME="my-course"
make content-module COURSE="my-course" NAME="my-module"
```

### Validation Errors

Common issues:
- Missing frontmatter: Add `---` delimiters
- Missing title: Add `title: "Your Title"` in frontmatter
- Missing summary: Add `summary: "Your summary"` in frontmatter

Run `make content-validate` to see specific errors.

## Templates

Generated content includes templates with:
- Proper frontmatter structure
- TODO placeholders for content
- Standard sections for each type

Templates located at: `scripts/content-generator/templates/`

## Tools Available

- **Content Generator**: Automatically scaffolds new content with proper structure
- **Content Validator**: Validates frontmatter and file structure
- **Templates**: Pre-configured templates for each content type
- **Make Commands**: Easy-to-use commands for common tasks

## Complete Workflow

1. Create content structure using Make commands
2. Edit generated files and replace TODO placeholders
3. Update parent `module-overview.md` files to link to new lessons (if needed)
4. Validate with `make content-validate`
5. Test locally with `cd apps/website && npm run dev`
6. Commit and push

## Additional Resources

- [Astro Content Collections](https://docs.astro.build/en/guides/content-collections/) - Astro content management
- [Astro Documentation](https://docs.astro.build/) - Astro features and APIs

