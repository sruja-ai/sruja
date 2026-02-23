# Sruja Architecture Agent

AI-powered architecture discovery skill that helps you understand and document your software architecture.

## What It Does

The Sruja Architecture Agent skill enables AI assistants (Claude, Cursor, Copilot, etc.) to:

- 📊 **Analyze codebases** - Single repos or multiple microservices
- 🔍 **Detect technologies** - Languages, frameworks, databases, external services
- 🌐 **Discover dependencies** - External APIs, message queues, cloud services
- 📝 **Generate Sruja DSL** - Valid, validated architecture definitions
- 🔗 **Trace end-to-end flows** - From users to backend services
- 📚 **Import from specs** - OpenAPI, GraphQL, AsyncAPI
- ✅ **Validate & iterate** - Collaborative refinement

## Installation

### Option 1: Using skills.sh (Recommended)

```bash
npx skills add sruja-ai/sruja --skill sruja-architecture-agent
```

### Option 2: Manual Installation

1. Download the skill file:
   ```bash
   mkdir -p ~/.skills/sruja-architecture-agent
   curl -o ~/.skills/sruja-architecture-agent/SKILL.md \
     https://raw.githubusercontent.com/sruja-ai/sruja/main/skills/sruja-architecture-agent/SKILL.md
   ```

2. Configure your AI assistant to use the skill

### Option 3: Project-Level

Add to your project's `.cursor/skills.json` or similar:

```json
{
  "skills": [
    "sruja-architecture-agent"
  ]
}
```

## Usage

### In Claude Desktop

```
You: Analyze the architecture of my microservices platform

Claude: I'll analyze your architecture using the Sruja Agent skill.
        
        [Analyzes repositories, generates architecture]
        
        I've detected 3 services:
        - User Service (Node.js, PostgreSQL)
        - Order Service (Python, MongoDB)  
        - Payment Service (Go, Redis)
        
        [Shows complete architecture]
```

### In Cursor IDE

```
You: @skill:sruja-architecture-agent analyze this repository

Cursor: [Analyzes code, generates architecture.sruja]
```

### In VS Code + Continue

```
You: Use the sruja-architecture-agent skill to understand my architecture

Continue: [Follows skill instructions, generates architecture]
```

## Examples

### Example 1: Single Service

```
You: Analyze github.com/myorg/user-service

Agent: 
1. Clones repository
2. Reads package.json, docker-compose.yml, README.md
3. Detects Node.js API with PostgreSQL
4. Generates architecture.sruja
5. Validates with sruja lint

✓ Architecture generated and validated
```

### Example 2: Microservices

```
You: Analyze my e-commerce platform with 3 services

Agent:
1. Analyzes each service separately
2. Detects cross-service communication
3. Identifies shared dependencies (Stripe, AWS S3)
4. Generates master architecture with imports
5. Traces end-to-end flows

✓ Complete architecture with service relationships
```

### Example 3: From Documentation

```
You: Generate architecture from my docs/ folder

Agent:
1. Reads architecture.md, deployment.md, api.md
2. Extracts system design from documentation
3. Generates architecture based on docs
4. Notes gaps and ambiguities

✓ Architecture generated from documentation
```

## What Gets Detected

### Technologies
- **Languages**: Node.js, Python, Go, Java, Rust, Ruby, etc.
- **Frameworks**: Express, Django, FastAPI, Spring Boot, Rails, etc.
- **Databases**: PostgreSQL, MongoDB, MySQL, Redis, Elasticsearch
- **Message Queues**: RabbitMQ, Kafka, SQS, Redis Pub/Sub

### Architecture Elements
- **Systems**: Services, applications, bounded contexts
- **Containers**: APIs, workers, web apps, background jobs
- **Datastores**: Databases, caches, queues
- **External Services**: Stripe, AWS, Twilio, etc.
- **End Users**: Personas, segments, behaviors

### Relationships
- Service-to-service communication
- Database connections
- External API calls
- User interactions
- Event flows

## Workflow

1. **Ask** - Tell the AI what you want to analyze
2. **Analyze** - AI uses the skill to examine your codebase
3. **Generate** - AI produces Sruja DSL architecture
4. **Validate** - Architecture is validated automatically
5. **Refine** - Collaborate to add missing details

## What You Get

- ✅ `architecture.sruja` - Valid Sruja DSL file
- ✅ Service definitions with technologies
- ✅ Database and external dependency mapping
- ✅ End-to-end flow documentation
- ✅ Exportable to Markdown, Mermaid, JSON

## Advanced Features

### Import from OpenAPI

```
You: Import Stripe's architecture from their OpenAPI spec at https://api.stripe.com/v1/openapi.json

Agent: [Fetches spec, converts to Sruja external_system]
```

### Trace End-to-End Flows

```
You: Trace the complete flow from user login to order completion

Agent: [Traces path through all services, generates flow diagram]
```

### Gap Analysis

```
You: What's missing from my architecture documentation?

Agent: [Analyzes completeness, suggests improvements]
```

## Requirements

The AI assistant needs access to these tools:
- **git** - Clone repositories
- **read** - Read files from filesystem  
- **fetch** - Fetch content from URLs
- **sruja** - Validate architecture (optional but recommended)

## Tips

### Better Results

1. **Be specific** - "Analyze my Node.js API" vs "Analyze code"
2. **Provide context** - Mention if it's microservices, monolith, etc.
3. **Point to key files** - "Focus on the services/ directory"
4. **Iterate** - Start high-level, then add detail

### Common Patterns

```bash
# Analyze single repo
"Analyze the architecture of github.com/myorg/service-name"

# Analyze multiple repos
"Analyze my microservices: user-service, order-service, payment-service"

# Focus on specific aspect
"What external dependencies does my service use?"

# Generate specific output
"Create a Mermaid diagram of my architecture"

# Fill gaps
"Add deployment patterns to my architecture"
```

## Output Format

The agent generates output in this format:

```markdown
## Architecture Analysis

### Summary
- Services: 3 services detected
- Technologies: Node.js, Python, Go
- External Dependencies: Stripe, AWS S3
- End Users: E-commerce shoppers

### Services

#### User Service
- Technology: Node.js, Express
- Description: User management API
- Components: REST API, PostgreSQL, Redis

### Generated Architecture

```sruja
system "User Service" {
  api = container "REST API" {
    technology "Node.js"
  }
  
  db = database "PostgreSQL" {
    technology "PostgreSQL"
  }
  
  api -> db "SQL"
}
```

✓ Validated with sruja lint
```

## Troubleshooting

### Agent can't access repository

**Solution**: Provide a local path or ensure git credentials are configured

```bash
# Use local path
"Analyze the architecture in ./my-service"

# Or clone first, then point to directory
```

### Missing technologies detected

**Solution**: Point to specific files or provide hints

```
"My API uses FastAPI (Python). Analyze the src/ directory."
```

### Incomplete architecture

**Solution**: Ask for gap analysis and fill in manually

```
"What's missing from this architecture?"
```

## Contributing

Found an issue or have suggestions?

1. Open an issue: https://github.com/sruja-ai/sruja/issues
2. Submit improvements to the SKILL.md file
3. Share example architectures

## Learn More

- **Sruja Documentation**: https://sruja.ai/docs
- **Language Specification**: docs/LANGUAGE_SPECIFICATION.md
- **Examples**: examples/ directory
- **Discord**: https://discord.gg/VNrvHPV5

## License

Apache 2.0

---

**Sruja Architecture Agent** - AI-native architecture discovery that works with your AI assistant.