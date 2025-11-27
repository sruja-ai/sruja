# MCP Integration

This document describes how Sruja integrates with AI assistants via the Model Context Protocol (MCP).

[← Back to Documentation Index](../README.md)

## Overview

MCP (Model Context Protocol) is a standardized API that allows external agents, particularly LLMs, to interact with the architecture model in a structured way. This makes Sruja "AI-ready" and enables powerful AI-driven features.

## Why MCP is Critical

MCP integration is the **killer feature** that differentiates Sruja from other architecture tools. It enables:

- **AI-powered architecture generation** - Generate architecture from requirements
- **Architecture-aware code generation** - AI generates code that follows architecture
- **Automated validation** - AI checks architecture against best practices
- **Intelligent suggestions** - AI suggests improvements and alternatives
- **Reverse engineering** - Generate architecture from existing codebases

## MCP Endpoint Design

The platform exposes an MCP endpoint that provides a set of tools for AI agents.

### Core Tools

#### 1. `read_model`
Retrieve the current state of the architecture model.

**Request:**
```json
{
  "tool": "read_model",
  "params": {
    "project": "optional-project-id",
    "format": "json" | "dsl"
  }
}
```

**Response:**
```json
{
  "model": { /* JSON architecture model */ },
  "version": "1.0.0",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### 2. `update_model`
Propose changes to the model (e.g., add a new component, change a relationship).

**Request:**
```json
{
  "tool": "update_model",
  "params": {
    "changes": [
      {
        "action": "add_component",
        "component": {
          "id": "payment-service",
          "type": "Service",
          "name": "Payment Service",
          "technology": "Go"
        }
      }
    ]
  }
}
```

**Response:**
```json
{
  "success": true,
  "updated_model": { /* updated JSON model */ },
  "validation_errors": []
}
```

#### 3. `query_model`
Ask questions about the model.

**Request:**
```json
{
  "tool": "query_model",
  "params": {
    "query": "What are the dependencies of the Payment Service?",
    "format": "natural" | "structured"
  }
}
```

**Response:**
```json
{
  "answer": "The Payment Service depends on: User Service, Order Service, and Payment Gateway",
  "dependencies": [
    { "from": "payment-service", "to": "user-service", "type": "uses" },
    { "from": "payment-service", "to": "order-service", "type": "uses" },
    { "from": "payment-service", "to": "payment-gateway", "type": "uses" }
  ]
}
```

#### 4. `get_architecture_context`
Get full context for a component (VHLD, HLD, LLD, requirements, ADRs).

**Request:**
```json
{
  "tool": "get_architecture_context",
  "params": {
    "component": "BillingService"
  }
}
```

**Response:**
```json
{
  "component": {
    "id": "billing-service",
    "vhld": { /* very high-level design context */ },
    "hld": { /* high-level design context */ },
    "lld": { /* low-level design context */ },
    "requirements": [ /* linked requirements */ ],
    "adrs": [ /* linked ADRs */ ],
    "interfaces": [ /* API contracts */ ],
    "constraints": [ /* architectural constraints */ ]
  }
}
```

#### 5. `validate_generated_code`
Validate generated code against architecture rules.

**Request:**
```json
{
  "tool": "validate_generated_code",
  "params": {
    "code_path": "/billing/src",
    "component": "BillingService"
  }
}
```

**Response:**
```json
{
  "valid": false,
  "violations": [
    {
      "rule": "no-direct-db-access",
      "message": "Service directly accesses database instead of using repository pattern",
      "severity": "error",
      "location": "billing/src/db.go:42"
    }
  ]
}
```

#### 6. `generate_scaffold`
Generate code templates aligned with architecture.

**Request:**
```json
{
  "tool": "generate_scaffold",
  "params": {
    "component": "GDPRDataStore",
    "language": "go",
    "template": "default"
  }
}
```

**Response:**
```json
{
  "scaffold": {
    "files": [
      {
        "path": "gdpr/store.go",
        "content": "/* Generated scaffold code */"
      }
    ],
    "instructions": "This scaffold implements GDPR-compliant data store pattern"
  }
}
```

#### 7. `explain_why`
Get ADR lineage and history for a component.

**Request:**
```json
{
  "tool": "explain_why",
  "params": {
    "component": "AuthService"
  }
}
```

**Response:**
```json
{
  "explanation": "AuthService uses OAuth2 because...",
  "adrs": [
    {
      "id": "ADR-003",
      "title": "Use OAuth2 for authentication",
      "context": "...",
      "decision": "...",
      "consequences": "..."
    }
  ],
  "history": [ /* evolution timeline */ ]
}
```

## Use Cases

### 1. Architecture Generation from Requirements

An LLM can automatically generate a high-level architecture diagram from a set of requirements or user stories.

**Flow:**
1. User provides requirements in natural language
2. LLM calls `read_model` to understand current architecture
3. LLM calls `update_model` to add new components based on requirements
4. Platform validates and renders the updated architecture

### 2. Architecture-Aware Code Generation

AI code assistants can generate code that follows the architecture model.

**Flow:**
1. Developer asks AI to implement a feature
2. AI calls `get_architecture_context` to understand architectural constraints
3. AI generates code following architecture patterns
4. AI calls `validate_generated_code` to check compliance
5. Code is generated with architecture compliance

### 3. Automated Architecture Review

LLMs can check architecture against best practices and compliance rules.

**Flow:**
1. Architecture model is updated
2. AI calls `read_model` to get current state
3. AI analyzes against best practices (security, performance, etc.)
4. AI calls `update_model` to add recommendations or flags issues
5. Issues are displayed in the UI

### 4. Reverse Engineering from Code

Generate architecture models from existing codebases.

**Flow:**
1. User imports a codebase
2. AI analyzes code structure
3. AI calls `update_model` to create initial architecture model
4. Model is refined and validated
5. Architecture diagram is generated

## Implementation in Go

### MCP Server Structure

```go
// pkg/mcp/server.go
type MCPServer struct {
    modelEngine *engine.ModelEngine
    validator   *engine.Validator
}

func (s *MCPServer) HandleRequest(req MCPRequest) MCPResponse {
    switch req.Tool {
    case "read_model":
        return s.handleReadModel(req.Params)
    case "update_model":
        return s.handleUpdateModel(req.Params)
    case "query_model":
        return s.handleQueryModel(req.Params)
    // ... other tools
    }
}
```

### Integration Points

1. **CLI Integration**: `sruja mcp` command starts MCP server
2. **HTTP Endpoint**: REST API for MCP tools
3. **JSON-RPC**: Standard MCP protocol support
4. **WebSocket**: Real-time MCP communication (future)

## Security Considerations

- **Authentication**: MCP endpoints should require authentication
- **Authorization**: Different tools may require different permissions
- **Rate Limiting**: Prevent abuse of AI endpoints
- **Validation**: All model updates must be validated
- **Audit Logging**: Track all MCP tool usage

## Future Enhancements

- **Streaming Responses**: For large model queries
- **Batch Operations**: Multiple tool calls in one request
- **Webhook Support**: Notify external systems of model changes
- **Custom Tools**: Allow organizations to define custom MCP tools
- **Tool Marketplace**: Share custom tools across organizations

---

## Complete Tool Reference

For a comprehensive list of all available MCP tools, see:

- **[MCP Tools Reference](../notebooks/mcp-tools.md)** - Complete tool definitions for notebook/kernel integration

The notebook integration provides additional tools for:
- Snapshots and variants
- Diagram generation
- Event simulation
- LSP features
- Code alignment

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io)
- [MCP Server Implementation](../../pkg/mcp/server.go)
- [MCP Tools Reference](../notebooks/mcp-tools.md) - Complete tool definitions
- [Architecture Model Schema](../specs/model-schema.md)

