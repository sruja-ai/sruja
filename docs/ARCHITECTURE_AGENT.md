# Sruja Architecture Agent - AI-Powered Architecture Discovery

## Overview

Sruja Architecture Agent is an AI-powered approach to architecture discovery that leverages the capabilities of modern AI assistants (Claude, Cursor, Copilot, etc.) to automatically analyze codebases and generate Sruja architecture DSL.

## The Approach: Skills Over Infrastructure

Instead of building complex crawler infrastructure with custom parsers and detectors for every language and framework, Sruja uses a **skill-based approach**:

- **Traditional approach**: Build parsers, detectors, analyzers for each language/framework
- **Sruja approach**: Provide a comprehensive skill that teaches AI assistants how to discover architecture

This approach is:
- ✅ **Simpler** - No infrastructure to build or maintain
- ✅ **Universal** - Works with any language the AI understands
- ✅ **Smarter** - AI understands context and semantics
- ✅ **Maintainable** - Just update the skill file, no code changes
- ✅ **Portable** - Single markdown file distribution

## How It Works

### 1. Install the Skill

```bash
npx skills add sruja-ai/sruja --skill sruja-architecture-agent
```

### 2. Ask Your AI Assistant

```
You: Analyze the architecture of my microservices platform

AI: I'll analyze your architecture using the Sruja Agent skill.
    
    [Uses git to clone repos]
    [Reads package.json, docker-compose.yml, README.md]
    [Analyzes code structure]
    [Generates Sruja DSL]
    [Validates with sruja lint]
    
    ✓ Architecture generated and validated
```

### 3. Review and Refine

The AI presents the architecture and asks clarifying questions:
- Did I identify all services correctly?
- Are there external services I missed?
- Should I add deployment patterns?

You provide feedback, and the AI iterates until you're satisfied.

## What Gets Discovered

The AI agent can discover:

### Technologies
- **Languages**: Node.js, Python, Go, Java, Rust, Ruby, C#, etc.
- **Frameworks**: Express, Django, FastAPI, Spring Boot, Rails, Next.js, etc.
- **Databases**: PostgreSQL, MongoDB, MySQL, Redis, Elasticsearch, etc.
- **Message Queues**: RabbitMQ, Kafka, SQS, Redis Pub/Sub
- **Cloud Services**: AWS, GCP, Azure, Stripe, Twilio, etc.

### Architecture Elements
- **Systems**: Services, applications, bounded contexts
- **Containers**: APIs, workers, web apps, background jobs
- **Datastores**: Databases, caches, queues
- **External Systems**: Third-party services and APIs
- **People**: End users, personas, segments

### Relationships
- Service-to-service communication
- Database connections
- External API calls
- User interactions
- Event flows

## Tools Required

The AI assistant needs access to these tools:

1. **git** - Clone and explore repositories
2. **read** - Read files from filesystem
3. **fetch** - Fetch content from URLs (for OpenAPI specs, docs)
4. **sruja** - Validate architecture (optional but recommended)

Most modern AI assistants (Claude Desktop, Cursor, Continue) already have these capabilities.

## Use Cases

### Single Service Analysis

```
You: Analyze github.com/myorg/user-service

AI: [Clones repo, analyzes code, generates architecture]
```

### Multi-Service Architecture

```
You: Analyze my e-commerce platform: user-service, order-service, payment-service

AI: [Analyzes each service, detects relationships, generates complete architecture]
```

### Import from Specs

```
You: Import Stripe's architecture from their OpenAPI spec

AI: [Fetches spec, converts to Sruja external_system definition]
```

### Documentation-Based

```
You: Generate architecture from my docs/ folder

AI: [Reads architecture docs, extracts system design, generates architecture]
```

## Workflow

```
┌─────────────────┐
│  User Request   │
│ "Analyze my     │
│  architecture"  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  AI Agent       │
│  (Using Skill)  │
├─────────────────┤
│ • Clone repos   │
│ • Read files    │
│ • Analyze code  │
│ • Generate DSL  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Validation     │
│  sruja lint     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  User Review    │
│  & Refinement   │
└─────────────────┘
```

## Benefits

### vs. Manual Documentation
- **Faster** - Minutes instead of days
- **Accurate** - Based on actual code, not outdated docs
- **Complete** - Catches things humans miss
- **Living** - Easy to update as code changes

### vs. Building Crawler Infrastructure
- **No infrastructure** - No servers, no deployment
- **No maintenance** - No parsers to update
- **Multi-language** - Works with any language
- **Context-aware** - AI understands semantics
- **Interactive** - Can ask clarifying questions

### vs. Static Analysis Tools
- **Semantic understanding** - Understands intent, not just syntax
- **Flexible** - Handles edge cases and custom patterns
- **Conversational** - Interactive refinement
- **Holistic** - Sees the big picture

## Example Output

```sruja
system "User Service" {
  description "User management and authentication service"
  
  metadata {
    repo "github.com/myorg/user-service"
    language "Node.js"
    framework "Express 4.18"
  }
  
  api = container "REST API" {
    technology "Node.js"
    description "User management API with authentication"
    
    metadata {
      port 3000
      endpoints [
        "POST /auth/login",
        "GET /users/:id",
        "PUT /users/:id"
      ]
    }
  }
  
  db = database "User Database" {
    technology "PostgreSQL"
    description "User profiles and authentication data"
  }
  
  cache = database "Session Cache" {
    technology "Redis"
    description "Active user sessions and JWT blacklist"
  }
  
  api -> db "SQL queries"
  api -> cache "Redis protocol"
}

external_system "Stripe" {
  description "Payment gateway for premium subscriptions"
  
  container "Payment API" {
    technology "REST"
    metadata {
      usage ["subscription management", "payment processing"]
    }
  }
}

user_service.api -> stripe.payment_api "HTTPS - subscription payments"

person "End User" {
  description "Application users"
  metadata {
    segment "SaaS users"
    behaviors ["register", "login", "manage profile"]
  }
}

end_user -> user_service.api "HTTPS - authentication and profile management"
```

## Advanced Features

### End-to-End Flow Tracing

```
You: Trace the complete flow from user placing an order to payment

AI: [Traces through all services]
    
    Flow: User → Web App → Order Service → Payment Service → Stripe
    
    [Generates view showing complete path]
```

### Gap Detection

```
You: What's missing from this architecture?

AI: Architecture Completeness: 72%
    
    ✓ Complete: Services, databases, external dependencies
    ⚠ Partial: External service details (only API surface)
    ✗ Missing: End users, deployment patterns, SLAs
    
    Recommendations:
    1. Define end user segments
    2. Add deployment patterns from docs
    3. Document SLA requirements
```

### Architecture Improvements

```
You: Suggest improvements for this architecture

AI: Observations:
    
    1. Bottleneck: Order service used by 5 services
       → Consider: Decomposition or caching
    
    2. Single point of failure: One database for users
       → Consider: Read replicas
    
    3. External dependency: Critical path depends on Stripe
       → Consider: Circuit breaker pattern
```

## Comparison to Other Approaches

| Feature | Manual | Crawler | **AI Agent** |
|---------|--------|---------|--------------|
| Speed | Slow | Fast | Fast |
| Accuracy | Varies | High | High |
| Multi-language | ✓ | Complex | **✓** |
| Context awareness | ✓ | Limited | **✓** |
| Interactive | ✓ | No | **✓** |
| Infrastructure needed | No | Yes | **No** |
| Maintenance | Low | High | **Minimal** |
| Cost | Time | Infrastructure | **API calls** |

## Getting Started

### Prerequisites
- AI assistant with tool access (Claude Desktop, Cursor, etc.)
- Git credentials (for private repos)
- Sruja CLI (for validation)

### Quick Start

```bash
# 1. Install the skill
npx skills add sruja-ai/sruja --skill sruja-architecture-agent

# 2. In your AI assistant:
"Analyze the architecture of github.com/myorg/my-service"

# 3. Review and refine the generated architecture

# 4. Save to your project
# The AI will generate architecture.sruja

# 5. Validate
sruja lint architecture.sruja
```

## The Skill File

The actual skill is defined in `skills/sruja-architecture-agent/SKILL.md` and contains:

- **Role definition** - What the agent does
- **Process** - Step-by-step workflow
- **Detection patterns** - What to look for in code
- **Examples** - Real-world scenarios
- **Best practices** - How to work effectively
- **Validation rules** - Quality checklist

The skill is comprehensive (1000+ lines) and teaches the AI everything it needs to discover architecture intelligently.

## Philosophy

### External Systems: Surface Only

We don't try to discover the internal architecture of external systems (Stripe, AWS, etc.). Instead, we:
- Document their API surface (what we use)
- Note authentication methods
- Track SLA information if available
- Map our usage patterns

This is the correct approach because external systems are black boxes, and only their surface matters for integration.

### Iterative Discovery

Architecture discovery is a dialogue, not a one-shot process:
1. AI generates initial architecture
2. User reviews and provides feedback
3. AI refines based on input
4. Repeat until satisfied

You don't need 100% completeness to get value. Even partial architecture is useful.

### Code + Documentation

Architecture is expressed in both code and documentation:
- **Code**: What the software does (implementation)
- **Docs**: How it's used (deployment, integration, personas)

The AI agent analyzes both to get the complete picture.

## Future Enhancements

The skill can be improved over time:
- More detection patterns
- Better gap analysis
- Domain-specific patterns (e-commerce, fintech, etc.)
- Integration with more external specs
- Architecture improvement suggestions

All improvements are just updates to the skill file—no code changes needed.

## Learn More

- **Skill File**: `skills/sruja-architecture-agent/SKILL.md`
- **Skill README**: `skills/sruja-architecture-agent/README.md`
- **Examples**: `skills/sruja-architecture-agent/examples/`
- **Sruja Docs**: https://sruja.ai/docs
- **Discord**: https://discord.gg/VNrvHPV5

## Summary

Sruja Architecture Agent represents a new paradigm in architecture discovery:

- **No infrastructure** - Just a skill file
- **AI-powered** - Leverages modern AI capabilities
- **Universal** - Works with any language/framework
- **Interactive** - Collaborative refinement
- **Practical** - Generates usable architecture documentation

Instead of building complex crawler systems, we teach AI assistants how to discover architecture using their existing capabilities. This is simpler, smarter, and more maintainable.

---

**Sruja Architecture Agent** - AI-native architecture discovery that works with your AI assistant.