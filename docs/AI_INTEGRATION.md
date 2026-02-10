# Sruja AI Integration Guide

Guide for integrating Sruja's architecture DSL with AI code editors and assistants.

## Quick Start for AI Assistants

### 1. Understanding the DSL

Sruja defines software architecture using a declarative DSL with the following structure:

```sruja
architecture "Project Name" {
  system "System Name" {
    container "Component Name" {
      technology "Technology"
      description "Description"
    }
  }

  component1 -> component2 "relationship"
}
```

### 2. Prompt Templates

#### Generate from Natural Language

```
Create a Sruja architecture DSL for:

[Describe your system]

Include:
- All systems and containers
- Technology stack for each component
- Data flow relationships
- Appropriate descriptions

Follow these rules:
1. Define all referenced components before use
2. Use double quotes for strings
3. Include technology field for containers
4. Use -> for relationships with descriptive labels
5. Run sruja lint to validate
```

#### Refactor Existing Architecture

```
Review and refactor this Sruja architecture to [goal]:

[PASTE DSL]

Focus on:
- [specific improvements needed]
- Maintaining all relationships
- Valid component definitions
- Proper nesting

Validate the result with sruja lint.
```

#### Add Feature to Architecture

```
Add the following feature to this Sruja architecture:

Feature: [describe feature]
Components: [list components]
Relationships: [list data flow]

[PASTE EXISTING DSL]

Update the architecture to include the new feature,
ensuring all components are defined and relationships
are properly established.
```

#### Convert from Other Format

```
Convert this architecture description to Sruja DSL:

[PASTE DESCRIPTION]

Generate valid Sruja code with:
- Proper component definitions
- Technology specifications
- All relationships
- Descriptions for clarity

Validate with sruja lint after generation.
```

### 3. Common Patterns for AI

#### Pattern: Generate from Diagram Description

```
Generate Sruja DSL from this diagram description:
- Frontend web app
- API gateway
- Microservices: Auth, Orders, Payments
- Databases: Users DB, Orders DB
- Message queue: RabbitMQ
- Cache: Redis

Include all relationships with appropriate protocols.
```

#### Pattern: Fix Validation Errors

```
Fix this Sruja DSL that has validation errors:

[PASTE DSL WITH ERRORS]

Errors to fix:
- [list errors from sruja lint output]

Ensure the fixed DSL:
- Passes sruja lint
- Maintains the original architecture
- Has all required fields
```

#### Pattern: Migration Scenario

```
Generate Sruja architecture for migrating from [old] to [new]:

Current architecture: [describe]
Target architecture: [describe]
Migration approach: [describe]

Include both current and target states,
marking migration phases if applicable.
```

### 4. System Prompts for AI Agents

#### Generic AI Agent

```
You are an expert in Sruja architecture DSL.

When generating Sruja code:
1. Start with architecture "[name]" block
2. Define all component types (kind) first
3. Define all components before relationships
4. Include technology field for all containers
5. Use descriptive labels for relationships
6. Add descriptions for all components
7. Validate structure (no circular dependencies, no orphans)

Always verify the generated code follows the Sruja DSL syntax.
```

#### Code Review Assistant

```
You are a code reviewer specializing in Sruja architecture.

When reviewing Sruja DSL:
1. Check all components are defined before use
2. Verify no orphan components
3. Ensure no circular dependencies
4. Validate proper nesting (systems contain containers)
5. Check technology fields are present for containers
6. Verify relationship labels are descriptive
7. Ensure descriptions explain component purpose

Provide specific, actionable feedback.
```

### 5. Training Data Format

#### Example for Fine-Tuning

```json
{
  "input": "Create an e-commerce system with web frontend, backend API, and payment service",
  "output": "architecture \"E-Commerce\" {\n  system \"Application\" {\n    web = container \"Web Frontend\" {\n      technology \"React\"\n      description \"User-facing web application\"\n    }\n    api = container \"API Service\" {\n      technology \"Node.js\"\n      description \"RESTful API for business logic\"\n    }\n    payment = container \"Payment Service\" {\n      technology \"Python\"\n      description \"Payment processing service\"\n    }\n  }\n\n  web -> api \"HTTPS\"\n  api -> payment \"REST API\"\n}",
  "validation": "success"
}
```

### 6. OpenAI Function Calling

#### Tool Definition

```json
{
  "type": "function",
  "function": {
    "name": "generate_sruja_architecture",
    "description": "Generate Sruja architecture DSL from natural language description",
    "parameters": {
      "type": "object",
      "properties": {
        "description": {
          "type": "string",
          "description": "Natural language description of the architecture"
        },
        "technology_stack": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Technology stack to use (e.g., ['React', 'Node.js', 'PostgreSQL'])"
        }
      },
      "required": ["description"]
    }
  }
}
```

### 7. Anthropic Tool Use

#### Tool Definition

```xml
<tool_description>
  <tool_name>generate_sruja_dsl</tool_name>
  <description>Generate Sruja architecture DSL from architecture description</description>
  <input_schema>
    <type>object</type>
    <properties>
      <property>
        <name>architecture_description</name>
        <type>string</type>
        <description>Detailed description of the software architecture</description>
      </property>
      <property>
        <name>components</name>
        <type>array</type>
        <items>
          <type>string</type>
        </items>
          <description>List of components to include</description>
      </property>
    </properties>
    <required>
      <item>architecture_description</item>
    </required>
  </input_schema>
</tool_description>
```

### 8. Integration with Popular AI Editors

#### Cursor AI

- Place `.cursorrules` in project root (already created)
- Cursor uses LSP - ensure Sruja LSP is running
- Enable "Context-aware AI" for best results

#### GitHub Copilot

- Place `.copilot-instructions.md` in project root (already created)
- Use Copilot Chat with file context: "Generate architecture for [description]"
- Enable Copilot Labs for code suggestions

#### Continue.dev

- Configure in `.continue/config.json`:

```json
{
  "contextFiles": [".cursorrules", ".copilot-instructions.md"],
  "customRules": ["Follow .cursorrules when generating Sruja DSL"]
}
```

#### Zed

- Zed uses LSP natively - configure Zed to use Sruja LSP
- Place prompt templates in `prompts/` directory
- Use Zed's AI assistant with these templates

### 9. CI/CD Integration

#### GitHub Action for AI-Generated DSL

```yaml
name: Validate Sruja DSL

on:
  pull_request:
    paths:
      - "**.sruja"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Sruja
        run: cargo install sruja-cli --git https://github.com/sruja-ai/sruja --locked
      - name: Lint all Sruja files
        run: find . -name '*.sruja' -exec sruja lint {} \;
```

### 10. Best Practices for AI Generation

#### For AI Model Developers

1. **Provide context**: Always include technology stack and system boundaries
2. **Be specific**: Describe data flow, protocols, and interactions
3. **Request validation**: Ask AI to validate generated code
4. **Iterate**: Review and refine generated architectures
5. **Use examples**: Reference existing architectures in prompts

#### For Code Review

1. **Check validation**: Run `sruja lint` on all AI-generated code
2. **Review relationships**: Ensure data flow makes sense
3. **Verify completeness**: No missing components or technologies
4. **Check naming**: Use consistent, descriptive names
5. **Document decisions**: Add comments for architectural decisions

### 11. Error Handling

#### Common AI Generation Errors

- **Undefined references**: Component used but not defined
  - Solution: Define all components before relationships

- **Circular dependencies**: System A depends on B, B on A
  - Solution: Extract common dependencies

- **Missing technology**: Container without technology field
  - Solution: Add technology specification

- **Orphan components**: Component with no relationships
  - Solution: Add appropriate relationships or remove if not needed

### 12. Resources

- **Documentation**: https://sruja.ai
- **Language Spec**: docs/LANGUAGE_SPECIFICATION.md
- **Examples**: examples/
- **GitHub**: https://github.com/sruja-ai/sruja
- **Discord**: https://discord.gg/VNrvHPV5

### 13. Contributing

To improve AI integration:

1. Add more examples to `examples/`
2. Document new patterns in this guide
3. Create prompt templates for common scenarios
4. Share successful prompts with the community
5. Report issues with AI-generated code

---

**Next Steps:**

1. Copy `.cursorrules` to your project
2. Copy `.copilot-instructions.md` to your project
3. Try the prompt templates with your AI assistant
4. Review and validate AI-generated code with `sruja lint`
