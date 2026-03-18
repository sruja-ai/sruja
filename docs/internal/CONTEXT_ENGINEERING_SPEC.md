# Context Engineering for AI Coding Agents

## 1. Executive Summary

### Problem
AI coding agents (Claude, Cursor, Copilot, etc.) are powerful at generating code, but unreliable at staying correct inside a specific repo's constraints. Even strong agents repeatedly fail in predictable ways:
- They pick the wrong files and miss existing patterns
- They violate boundaries (layering, forbidden dependencies, ownership)
- They ignore team conventions that are not encoded in code
- They require multiple loops (search → guess → build/test → fix) to converge

### Solution
**Sruja as Context Engine**: A deterministic context and validation layer that makes AI coding agents cheaper to use and safer to trust, without embedding an LLM.

### Core Insight (from Claude Code docs)
> **Context window is the fundamental constraint.** Everything optimizes around keeping it clean and relevant.

### User-First Value
Sruja's value is not "smarter generation". It is "fewer retries and fewer surprises":
- **Fast grounding**: give the agent architecture context (modules, boundaries, dependencies)
- **Deterministic guardrails**: validate changes against architecture rules
- **CI integration**: catch violations before merge
- **Cross-agent consistency**: same source-of-truth for Cursor/Claude/Copilot

---

## 2. Current State: What Exists Today

### 2.1 Implemented Commands

| Command | Status | Purpose |
|---------|--------|---------|
| `sruja init -r .` | ✅ Done | Initialize `.sruja/` directory |
| `sruja sync -r .` | ✅ Done | Refresh evidence + write `.sruja/context.json` + drift |
| `sruja discover --context -r . --format json` | ✅ Done | Repo structure, tech, areas summary |
| `sruja lint file.sruja` | ✅ Done | Validate DSL |
| `sruja drift -r .` | ✅ Done | Detect violations (cycles, orphans, layers) |
| `sruja explain <element>` | ✅ Done | Show dependencies/dependents |
| `sruja context -r . --format cursor-rules` | ✅ Done | Export AI context |
| `sruja export mermaid/markdown/json` | ✅ Done | Output formats |
| `sruja publish/compose` | ✅ Done | Multi-repo federation |

### 2.2 Generated Artifacts

| File | Purpose |
|------|---------|
| `.sruja/context.json` | Evidence + truth status + baseline path + git commit |
| `.sruja/graph.json` | Full scan graph for progressive discovery |
| `repo.sruja` | Architecture as code (C4 model) |

### 2.3 NOT Implemented (Spec Only)

| Feature | Status | Notes |
|---------|--------|-------|
| `sruja pack --for claude --task` | ❌ Not implemented | Context pack generation |
| `sruja query --for-file` | ❌ Not implemented | File-scoped context |
| `sruja validate --check conventions` | ❌ Not implemented | Convention checking |
| `.sruja/context.yaml` | ❌ Not implemented | User-declared conventions |
| MCP tools | ❌ Not implemented | Direct agent integration |
| `sruja-agent` crate | ❌ Not implemented | Would contain new features |

---

## 3. End-to-End User Workflow

### Phase 1: Setup (One-time)

```bash
# 1. Install Sruja CLI
curl -fsSL https://sruja.ai/install.sh | bash

# 2. Install the AI skill in your editor
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture

# 3. Initialize project
cd your-project
sruja init -r . --prompt   # Creates .sruja/ and init_prompt.txt
```

### Phase 2: Create Architecture Baseline

In your AI editor (Cursor, Claude, etc.):

```
Use sruja-architecture skill. Run sruja discover --context -r . --format json,
gather evidence, ask targeted questions if needed, generate repo.sruja,
then run sruja lint and fix until it passes.
```

**Output:** `repo.sruja` — your architecture as code

### Phase 3: Generate AI Context

```bash
# Export context for your AI tool
sruja context -r . --format cursor-rules -o .cursorrules

# Or for Copilot
sruja context -r . --format copilot-instructions -o .github/copilot-instructions.md

# Or JSON for custom use
sruja context -r . --format json -o .context/architecture.json
```

### Phase 4: Daily Development Loop

```
┌─────────────────────────────────────────────────────────┐
│  BEFORE CODING (Get Context)                            │
│  "Read @repo.sruja and @.sruja/context.json.            │
│   I'm working on [feature]. What components affected?"  │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  DURING CODING                                          │
│  "Implement [feature] following patterns in             │
│   @src/existing_example.rs. After: run tests."          │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  BEFORE COMMIT (Validate)                               │
│  $ sruja drift -r . --violations-only                   │
│  If architecture changed: $ sruja sync -r .             │
└────────────────────┴────────────────────────────────────┘
```

### Phase 5: Code Review Workflow

```bash
# Check for architectural drift
sruja drift -r . -a repo.sruja
```

In AI editor (fresh session for unbiased review):

```
Review my changes for:
1. Boundary violations (check @repo.sruja)
2. Bugs and edge cases
3. Missing tests
Output: file:line, severity, description
```

### Phase 6: Keep Architecture in Sync

```bash
# Weekly or after major changes
sruja sync -r .              # Refresh evidence
sruja drift -r .             # Check for drift
sruja lint repo.sruja        # Validate
```

If drift found, in AI editor:

```
Run sruja drift -r . and update repo.sruja to reflect the changes,
or list open questions if unclear.
```

### CI/CD Integration

```yaml
# .github/workflows/architecture.yml
name: Architecture Check
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: curl -fsSL https://sruja.ai/install.sh | bash
      - run: sruja lint repo.sruja
      - run: sruja drift -r . --violations-only --fail-on all
```

---

## 4. Best Practices (from Research)

### 4.1 Context Window Management

The #1 constraint. Key practices:

| Practice | Why |
|----------|-----|
| `/clear` between unrelated tasks | Prevents context pollution |
| Keep CLAUDE.md under 200 lines | Longer files reduce adherence |
| Use subagents for research | Isolated context, summary returns |
| Specific > vague prompts | Reduces back-and-forth |

### 4.2 CLAUDE.md / .cursorrules Pattern

The most effective approach for persistent context:

```markdown
# CLAUDE.md (keep under 200 lines)

## Build & Test
- Build: `cargo build --release`
- Test: `cargo test --workspace`
- Lint: `cargo clippy -- -D warnings`

## Architecture
- Workspace pattern: core crates + product crates
- API handlers live in `src/api/handlers/`
- Never import from `sruja-cli` into other crates

## Code Style
- Use `Result<T, Box<dyn std::error::Error>>` for errors
- Group imports: external → internal
- Run `cargo fmt` before committing
```

**What to include:**

| ✅ Include | ❌ Exclude |
|------------|------------|
| Bash commands AI can't guess | Standard language conventions |
| Code style that differs from defaults | Detailed API docs (link instead) |
| Testing instructions | Information that changes frequently |
| Architectural decisions | Self-evident practices |
| Environment quirks | File-by-file codebase descriptions |

### 4.3 Verification Criteria (Highest Leverage)

Always provide ways for AI to self-verify:

| Before | After |
|--------|-------|
| "implement email validation" | "write validateEmail(). Tests: user@example.com → true, invalid → false, user@.com → false. Run tests after." |

### 4.4 Explore → Plan → Execute Pattern

Don't let AI jump straight to coding:

```
Phase 1 (Plan Mode): "Read src/auth and understand session handling"
Phase 2: "Create a plan for adding OAuth"
Phase 3 (Normal Mode): "Implement the plan, write tests, run them"
Phase 4: "Commit and create PR"
```

### 4.5 Writer/Reviewer Pattern

Use separate sessions for unbiased review:

| Session A (Writer) | Session B (Reviewer) |
|--------------------|----------------------|
| "Implement rate limiter" | |
| | "Review @src/middleware/rateLimiter.ts for edge cases, race conditions" |
| "Fix issues from review" | |

Fresh context = better review. No bias toward code just written.

---

## 5. Implementation Roadmap

### Phase 1: Enhance Existing Commands (v0.19.x)

**Goal**: Improve what exists before adding new features

**Deliverables**:
- [ ] Enhance `sruja context` output with more detail from `repo.sruja`
- [ ] Add `--task` flag to `sruja context` for focused context
- [ ] Improve evidence collection in `sruja sync`
- [ ] Better layer inference from paths
- [ ] Document best practices in `sruja help`

### Phase 2: Convention Support (v0.20.x)

**Goal**: Support user-declared conventions

**Deliverables**:
- [ ] `.sruja/context.yaml` schema and loading
- [ ] `sruja validate` command
- [ ] Convention checking engine
- [ ] Boundary violation detection improvements

### Phase 3: MCP Server (v0.21.x)

**Goal**: Direct agent integration

**Deliverables**:
- [ ] MCP server implementation
- [ ] Tool definitions (query, validate, evidence)
- [ ] Integration tests with Claude/Cursor

### Phase 4: Advanced Features (v0.22.x)

**Goal**: Task-specific context

**Deliverables**:
- [ ] `sruja query --for-file` command
- [ ] Example similarity matching
- [ ] `sruja watch` file watcher
- [ ] Performance optimization

---

## 6. Data Structures

### Current: `.sruja/context.json`

```json
{
  "schema_version": 1,
  "updated_at": "2025-03-17T10:30:00Z",
  "git_commit": "abc1234",
  "truth_status": "reviewed",
  "baseline_path": "repo.sruja",
  "repo": ".",
  "scan_scope": { ... },
  "components": 42,
  "edges": 87,
  "primary_language": "Rust",
  "framework": null,
  "architecture_style": "monolith",
  "suggested_areas": ["crates", "extension", "skills"]
}
```

### Planned: `.sruja/context.yaml` (User-Declared)

```yaml
# .sruja/context.yaml
version: 1

conventions:
  error_handling:
    style: "Result<T, AppError>"
    examples:
      - "src/utils/errors.rs"
    
  testing:
    location: "same directory"
    pattern: "*_test.rs"
    framework: "cargo test"

architecture:
  default_layer_order: ["api", "service", "repository", "models"]

forbidden:
  - pattern: "unwrap()"
    reason: "Use expect() with message or proper error handling"
    severity: "warning"
```

---

## 7. MCP Tool Definitions (Planned)

```json
{
  "tools": [
    {
      "name": "sruja_query_context",
      "description": "Get architecture context for a file or task",
      "inputSchema": {
        "type": "object",
        "properties": {
          "file_path": { "type": "string" },
          "intent": { "type": "string", "enum": ["add-feature", "refactor", "fix-bug", "add-test"] }
        }
      }
    },
    {
      "name": "sruja_validate_code",
      "description": "Validate code against architecture boundaries",
      "inputSchema": {
        "type": "object",
        "properties": {
          "code": { "type": "string" },
          "file_path": { "type": "string" }
        },
        "required": ["code", "file_path"]
      }
    },
    {
      "name": "sruja_get_evidence",
      "description": "Get raw observations about code",
      "inputSchema": {
        "type": "object",
        "properties": {
          "path": { "type": "string" }
        },
        "required": ["path"]
      }
    }
  ]
}
```

---

## 8. Trade-offs and Decisions

### Decision: No LLM in Sruja

| Pro | Con |
|-----|-----|
| Zero marginal cost | Cannot interpret natural language |
| Works offline | Limited "understanding" |
| Universal agent support | Structured queries only |
| Deterministic (testable) | |
| No API keys needed | |

**Verdict**: Correct. Let AI agents provide intelligence.

### Decision: Enhance Existing Before Adding New

| Pro | Con |
|-----|-----|
| Ships value faster | Less "new" to market |
| Builds on proven code | |
| Reduces maintenance burden | |
| User feedback on real usage | |

**Verdict**: Correct. Current commands need polish before expansion.

### Decision: Declarative Conventions

| Pro | Con |
|-----|-----|
| Honest about capabilities | Manual setup required |
| User has control | Can drift from code |
| Language-agnostic | |
| No false positives | |

**Verdict**: Correct. Auto-detection would be noisy.

---

## 9. Success Metrics

| Metric | Target |
|--------|--------|
| Context generation time | <5s for standard repo |
| Drift detection accuracy | >95% correct violations |
| User setup time to first value | <5 minutes |
| CLAUDE.md/.cursorrules usefulness | User-reported >4/5 |
| CI integration adoption | 50% of users in 6 months |

---

## 10. Quick Reference: User Commands

| Task | Command |
|------|---------|
| Initialize | `sruja init -r .` |
| Generate architecture | "Use sruja-architecture skill..." |
| Export AI context | `sruja context -r . --format cursor-rules` |
| Check drift | `sruja drift -r .` |
| Sync evidence | `sruja sync -r .` |
| Impact analysis | `sruja explain ComponentName` |
| Validate DSL | `sruja lint repo.sruja` |
| View status | `sruja status -r .` |

---

## 11. References

- [Sruja Design Philosophy](../DESIGN_PHILOSOPHY.md)
- [Sruja Language Specification](../LANGUAGE_SPECIFICATION.md)
- [Claude Code Best Practices](https://code.claude.com/docs/en/best-practices.md)
- [Claude Code Memory/CLAUDE.md](https://code.claude.com/docs/en/memory.md)
- [MCP Specification](https://modelcontextprotocol.io/)

---

**Document Status**: Updated  
**Created**: 2025-03-17  
**Updated**: 2025-03-17
