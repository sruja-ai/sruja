---
title: "Module 4 - AI Editor Integration"
weight: 4
summary: "Set up MCP server, integrate with AI editors, and use context-driven development."
---

# Module 4 - AI Editor Integration

**MCP server setup, AI editor configuration, and context-driven development.**

## Overview

This module teaches how to integrate Sruja with AI code editors (Cursor, Trae, Copilot, Cline, Windsurf) for context-aware AI assistance.

## Lessons

### Lesson 1: [MCP Server Setup](lesson-1.md)
_Configure Sruja as an MCP server for AI editors_

- MCP protocol fundamentals
- Installing and configuring Sruja MCP
- Editor configuration (Cursor, VS Code, etc.)
- Testing the connection

### Lesson 2: [Context-Driven AI Assistance](lesson-2.md)
_Use architecture context in AI conversations_

- Sruja context commands
- Focus mode for task-specific context
- Cross-repo context building
- Context score optimization

### Lesson 3: [AI Pair Programming Patterns](lesson-3.md)
_Effective patterns for AI-assisted architecture work_

- Asking architecture-aware questions
- Using AI for impact analysis
- AI-assisted code review
- Wrapping AI output with Sruja validation

## Learning Outcomes

- ✅ Configure Sruja MCP server for AI editors
- ✅ Build and optimize architecture context for AI
- ✅ Use AI editors with architecture awareness
- ✅ Validate AI suggestions against Sruja

## Prerequisites

- Completed Agentic AI Modules 1-3
- Familiarity with AI code editors (Cursor, Copilot, etc.)
- Understanding of Sruja CLI basics

## Estimated Time

1-2 hours

---

## Raw Thoughts from Analysis

### v0.23.0+ Features to Cover:
- `sruja mcp` command (MCP stdio server)
- `sruja context` command for building context
- `sruja focus` for task-specific briefing
- `sruja context-score` for AI-readiness scoring
- Editor integrations (Cursor, GitHub Copilot, Cline, Windsurf)

### Related Concepts:
- Model Context Protocol (MCP)
- AI-editor context optimization
- Cross-repo architecture context
- Context-driven development
- AI pair programming

### Key Commands:
- `sruja mcp -r .`
- `sruja context -r repoA -r repoB`
- `sruja focus --file <path>`
- `sruja context-score -r .`
- `sruja why <component-id>`
- `sruja impact <component-id>`

### Editor Setup:
```json
// Cursor: .cursor/mcp.json
{
  "mcpServers": {
    "sruja": {
      "command": "sruja",
      "args": ["mcp", "-r", "."]
    }
  }
}
```

### Lesson Ideas:

1. **MCP Server Setup**
   - What is MCP and why it matters
   - Installing/configuring Sruja MCP
   - Editor-specific setup (Cursor, Copilot, etc.)
   - Testing with `sruja mcp -r .`

2. **Context-Driven Assistance**
   - Building context with `sruja context`
   - Using `sruja focus` for task briefing
   - Cross-repo context (`sruja context -r repoA -r repoB`)
   - Optimizing context score

3. **AI Pair Programming Patterns**
   - Asking "what depends on X?" before changing
   - Using `sruja impact` to understand blast radius
   - Validating AI suggestions with `sruja lint`
   - Using AI to generate Sruja from conversations