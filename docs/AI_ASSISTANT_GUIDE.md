# Guide: Using AI Code Assistants with Sruja

This guide explains how to configure your environment so AI code assistants (like GitHub Copilot, Cursor, Claude, ChatGPT, etc.) can effectively help you write Sruja code.

## Quick Setup

### 1. Reference the Language Specification

When asking an AI assistant to help with Sruja code, provide context by referencing:

```
Use the Sruja language specification. Sruja is an architecture-as-code DSL.
Documentation: https://sruja.ai
Language spec: See docs/LANGUAGE_SPECIFICATION.md in the Sruja repository
```

### 2. Include Examples

Share example Sruja files with the assistant:

```
Here's an example Sruja file:
[paste example from examples/ directory]
```

### 3. Point to Documentation

Reference the official documentation:

```
Reference: https://sruja.ai/docs
Examples: https://github.com/sruja-ai/sruja/tree/main/examples
```

## Recommended Prompt Template

Use this template when asking AI assistants for help:

```
I'm working with the Sruja architecture-as-code language. 

Language Documentation: https://sruja.ai
Language Specification: [link to LANGUAGE_SPECIFICATION.md]

Here's what I'm trying to model:
[describe your architecture]

Can you help me write the Sruja code for this?
```

## What AI Assistants Need

### Essential Information

1. **Language Grammar**: The complete syntax and structure
   - Location: `docs/LANGUAGE_SPECIFICATION.md`
   - Also: `pkg/language/ast.go` (for detailed grammar)

2. **Examples**: Real-world usage examples
   - Location: `examples/` directory
   - Contains: 40+ example files covering all features

3. **Documentation**: User-facing docs
   - Location: https://sruja.ai
   - Contains: Tutorials, concepts, getting started guide

### Optional but Helpful

4. **Language Server**: For IDE integration (future)
   - Would provide: Autocomplete, syntax highlighting, validation
   - Status: Not yet implemented

5. **JSON Schema**: For structure validation (future)
   - Would provide: Schema-based validation and autocomplete
   - Status: Not yet implemented

## Current Capabilities

✅ **What Works Now:**
- AI assistants can understand Sruja syntax when provided with:
  - Language specification document
  - Example files
  - Documentation links

✅ **What You Can Do:**
- Copy-paste the language spec into your prompt
- Share example files as context
- Reference the documentation site

## Future Enhancements

### Language Server Protocol (LSP)

A future LSP implementation would enable:
- Real-time autocomplete in IDEs
- Syntax validation as you type
- Go-to-definition for elements
- Hover documentation
- Error highlighting

### JSON Schema

A JSON Schema would enable:
- Schema-based validation
- Better autocomplete in JSON-aware editors
- Integration with tools like JSON Schema validators

### VS Code Extension

A VS Code extension would provide:
- Syntax highlighting
- IntelliSense
- Integrated validation
- Quick fixes

## Best Practices for AI-Assisted Development

1. **Start with Examples**: Show the AI assistant a similar example first
2. **Be Specific**: Describe what you want to model clearly
3. **Iterate**: Start simple, then add complexity
4. **Validate**: Always run `sruja lint` on generated code
5. **Format**: Use `sruja fmt` to ensure consistent formatting

## Example Workflow

```bash
# 1. Ask AI to generate initial code
# 2. Save to file
cat > my-architecture.sruja << 'EOF'
[AI-generated code]
EOF

# 3. Validate
sruja lint my-architecture.sruja

# 4. Format
sruja fmt my-architecture.sruja

# 5. Export to visualize
sruja export svg my-architecture.sruja > my-architecture.svg
```

## Resources

- **Language Spec**: `docs/LANGUAGE_SPECIFICATION.md`
- **Examples**: `examples/` directory
- **Documentation**: https://sruja.ai
- **GitHub Repo**: https://github.com/sruja-ai/sruja

## Contributing

If you find ways to improve AI assistant support, please:
1. Update this guide
2. Add examples to the `examples/` directory
3. Improve the language specification
4. Open an issue or PR













