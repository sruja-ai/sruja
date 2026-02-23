# AI-First Architecture Intelligence: Module Analysis

## Strategic Direction

**Primary User: AI Assistant (Claude, GPT, etc.)**
**Secondary User: Developer (occasionally views results)**

This document analyzes each of the 16 modules against this strategy.

---

## Module Analysis

### Core Layer

#### 1. sruja-language ✅ KEEP

**Purpose:** Parser and AST for .sruja DSL files

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **Yes** - AI reads/writes .sruja files |
| Critical for intelligence? | **Yes** - Intent declarations are parsed here |
| Maturity | Good - full language spec implemented |
| Changes needed | Minor - add round-trip (AST → DSL) |

**Why Keep:**
- AI needs to understand developer's declared architecture
- AI needs to generate .sruja files
- Foundation for all other modules

**Improvements:**
- Add `Program::to_dsl()` for round-trip
- Better error recovery for partial parses

---

#### 2. sruja-graph ✅ KEEP - CRITICAL

**Purpose:** Architecture Knowledge Graph - nodes, edges, decisions, policies

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **Yes** - This IS the intelligence |
| Critical for intelligence? | **Yes** - Core of the system |
| Maturity | Basic - needs enhancement |
| Changes needed | Major - add queries, temporal, provenance |

**Why Keep:**
- Central knowledge store for all architecture data
- Multi-source fusion (DSL + code + chats)
- Queryable by AI via MCP

**Current Gaps:**
```rust
// MISSING: Semantic queries
fn blast_radius(&self, node: &str) -> Vec<Impact>;  // "What breaks if I remove X?"
fn decision_chain(&self, node: &str) -> Vec<Decision>;  // "Why did we do this?"
fn similar_patterns(&self, pattern: &str) -> Vec<Pattern>;  // "Is this an anti-pattern?"
fn detect_drift(&self, declared: &Program) -> Vec<Drift>;  // "Code differs from declared"
```

**Improvements:**
- Add temporal versioning (history of changes)
- Add provenance tracking on every node/edge
- Add semantic query methods
- Add pattern detection

---

#### 3. sruja-engine ⚠️ KEEP - SIMPLIFY

**Purpose:** Rule-based validation engine (12 rules)

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **Partially** - AI can validate, but needs guidance |
| Critical for intelligence? | **No** - Useful but not core |
| Maturity | Good - well documented |
| Changes needed | Reduce from 12 to 3-5 core rules |

**Current Rules:**
```
unique_id        - KEEP (essential)
valid_ref        - KEEP (essential)
orphan           - KEEP (useful)
cycle            - KEEP (essential)
layer_violation  - MOVE to skill (opinionated)
database_isolation - MOVE to skill (opinionated)
simplicity       - MOVE to skill (opinionated)
slo_validation   - KEEP (useful)
properties_validation - KEEP (useful)
governance_validation - MOVE to skill
public_interface_documentation - MOVE to skill
scenario_validation - KEEP (useful)
```

**Recommended: Keep 5, Move 7 to Skills**

**Why Simplify:**
- AI doesn't need opinionated rules baked in
- Skills can provide domain-specific rules
- Reduces maintenance burden

---

#### 4. sruja-diagnostics ❌ MERGE INTO LANGUAGE

**Purpose:** Error types, severity, source locations

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **Indirectly** - errors from parsing/validation |
| Critical for intelligence? | **No** - Could be internal to language/engine |
| Maturity | Simple - just types |
| Changes needed | Merge into sruja-language |

**Why Merge:**
- Only used by language and engine
- No standalone value
- Reduces crate count

---

#### 5. sruja-export ❌ NOT CRITICAL

**Purpose:** Export AST to JSON, Mermaid, Markdown, PlantUML

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** - AI works with Graph, not diagrams |
| Critical for intelligence? | **No** - Output formatting |
| Maturity | Good |
| Changes needed | None if keeping, but... |

**Why Not Critical:**
- Diagrams are for humans, not AI
- AI can generate Mermaid directly from Graph
- MCP tool can expose a simple "export" if needed

**Recommendation:** Move to a single file in MCP server as a utility function, not a separate crate.

---

### Interface Layer

#### 6. sruja-mcp ✅ KEEP - CRITICAL

**Purpose:** Model Context Protocol server - AI's interface to Sruja

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **Yes** - This is how AI talks to Sruja |
| Critical for intelligence? | **Yes** - Primary interface |
| Maturity | Basic - needs more tools |
| Changes needed | Major - add more tools |

**Why Keep:**
- This IS the product for AI-first approach
- AI assistants use MCP to query architecture
- Already built, just needs enhancement

**Current Tools:**
```rust
get_architecture_summary
query_decisions
check_policies
```

**Missing Tools:**
```rust
add_node              // AI can add nodes
add_edge              // AI can add relationships
add_decision          // AI can record decisions
query_dependencies    // "What depends on X?"
query_impact          // "What breaks if I change X?"
detect_drift          // "Does code match declared?"
suggest_improvements  // AI asks AI for suggestions
get_patterns          // "What patterns match this?"
extract_from_chat     // Extract from conversation text
scan_repository       // Trigger code scan
```

**Improvements:**
- 15+ tools instead of 3
- Bidirectional (read AND write)
- Session management for conversations

---

#### 7. sruja-cli ❌ NOT CRITICAL

**Purpose:** Command-line interface for developers

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** - AI uses MCP |
| Critical for intelligence? | **No** - Developer convenience |
| Maturity | Good |
| Changes needed | None |

**Why Not Critical:**
- AI doesn't use CLI
- CLI is for developers (secondary user)
- Could be thin wrapper over MCP tools

**Recommendation:** Keep minimal CLI for developer convenience, but don't prioritize. Can even be in a separate repo.

---

#### 8. sruja-lsp ❌ NOT CRITICAL

**Purpose:** Language Server Protocol for IDE integration

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** - AI doesn't use IDEs |
| Critical for intelligence? | **No** - Developer experience |
| Maturity | Basic |
| Changes needed | None |

**Why Not Critical:**
- IDE features are for developers
- AI works through MCP, not LSP
- VS Code can use WASM directly

**Recommendation:** Defer or move to separate repo. Nice to have, not core.

---

#### 9. sruja-wasm ❌ NOT CRITICAL

**Purpose:** WebAssembly bindings for browser/extension

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** - AI uses MCP over HTTP |
| Critical for intelligence? | **No** - Browser optimization |
| Maturity | Good |
| Changes needed | None |

**Why Not Critical:**
- WASM is for browser-side execution
- AI doesn't run in browser
- MCP server handles AI requests

**Recommendation:** Keep if VS Code extension is maintained, otherwise defer.

---

#### 10. VS Code Extension ❌ NOT CRITICAL

**Purpose:** Syntax highlighting, diagnostics, diagram preview

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** |
| Critical for intelligence? | **No** |
| Maturity | Good |
| Changes needed | None |

**Why Not Critical:**
- For developers editing .sruja files
- AI doesn't use VS Code
- Could be separate product/repo

**Recommendation:** Move to separate repo. Maintain if valuable, but not core to AI-first.

---

### Intelligence Layer

#### 11. sruja-extract ✅ KEEP - CRITICAL

**Purpose:** LLM-based extraction of architecture from conversations

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **Yes** - Extract from chat history |
| Critical for intelligence? | **Yes** - Key intelligence source |
| Maturity | Basic - works but needs improvement |
| Changes needed | Major - accuracy, more extraction types |

**Why Keep:**
- Turns unstructured chat into structured knowledge
- Key differentiator vs static tools
- Enables "learn from conversations"

**Current Extraction Types:**
```rust
Decision    // Architectural decisions
Requirement // Requirements
Constraint  // Design constraints
Policy      // Governance policies
Risk        // Identified risks
Component   // System components
```

**Improvements:**
- Better prompt engineering for accuracy
- Confidence scoring
- Batch extraction (multiple items from one message)
- Extraction from meeting notes, PRs, issues
- Conflict detection (two people say different things)

---

#### 12. sruja-scan ✅ KEEP - CRITICAL

**Purpose:** Code scanner - infer architecture from source code

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **Yes** - Discover reality vs intent |
| Critical for intelligence? | **Yes** - Ground truth |
| Maturity | Basic - 5 languages |
| Changes needed | Major - more languages, more patterns |

**Why Keep:**
- Discovers what code actually does (vs what .sruja says)
- Key for drift detection
- No manual entry needed

**Current Support:**
- TypeScript
- JavaScript
- Python
- Go
- Rust

**Improvements:**
- Java/Kotlin (enterprise)
- C#/.NET (enterprise)
- Framework-specific patterns (Next.js, Django, Spring)
- Database schema extraction
- API endpoint detection
- Service mesh detection (Istio, Linkerd)

---

#### 13. sruja-chat ❌ DELETE - USE SLACK

**Purpose:** Multi-party chat system with AI agents

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** - AI doesn't need custom chat |
| Critical for intelligence? | **No** - UI, not intelligence |
| Maturity | Moderate |
| Changes needed | Replace with Slack bot |

**Why Delete:**
- Building chat is a distraction
- Developers already use Slack/Teams
- Slack has better UX than any custom chat
- Slack bot = 200 lines, custom chat = 2000+ lines

**Replacement:**
```
slack-bot/
├── src/
│   ├── app.ts         # Slack Bolt (50 lines)
│   └── mcp-client.ts  # Call MCP server
└── package.json
```

---

#### 14. sruja-watch ❌ NOT CRITICAL

**Purpose:** File watcher for change detection

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** - Can trigger on demand |
| Critical for intelligence? | **No** - Optimization |
| Maturity | Simple |
| Changes needed | None |

**Why Not Critical:**
- Real-time watching is optimization
- MCP can trigger scan on demand
- Adds complexity for marginal benefit

**Recommendation:** Delete or defer. On-demand is fine for AI.

---

### Application Layer

#### 15. sruja-app (Desktop) ❌ DELETE

**Purpose:** Desktop application with Dioxus UI

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** |
| Critical for intelligence? | **No** - UI |
| Maturity | Basic |
| Changes needed | Delete |

**Why Delete:**
- Desktop apps are distribution nightmare
- Slack bot provides same value
- Maintaining UI is expensive
- Not core to intelligence

**Replacement:** Slack bot + optional web dashboard (separate repo if needed)

---

#### 16. skill-lint ❌ NOT CRITICAL

**Purpose:** Linter for skill definitions

| Aspect | Assessment |
|--------|------------|
| AI needs it? | **No** - Internal tool |
| Critical for intelligence? | **No** - Developer tool |
| Maturity | Good |
| Changes needed | None |

**Why Not Critical:**
- Only used when writing skills
- Not runtime intelligence
- Can be separate repo or simple script

---

## Summary: Keep vs Delete

### ✅ KEEP (5 Critical Modules)

| Module | Role | Priority |
|--------|------|----------|
| **sruja-graph** | Knowledge store | P0 - Enhance |
| **sruja-mcp** | AI interface | P0 - Expand tools |
| **sruja-extract** | LLM extraction | P0 - Improve accuracy |
| **sruja-scan** | Code discovery | P0 - More languages |
| **sruja-language** | DSL parser | P1 - Add round-trip |

### ⚠️ KEEP BUT SIMPLIFY (2 Modules)

| Module | Action |
|--------|--------|
| **sruja-engine** | Reduce 12 rules → 5 core rules, rest to skills |
| **sruja-diagnostics** | Merge into sruja-language |

### ❌ DELETE / DEFER (9 Modules)

| Module | Action | Reason |
|--------|--------|--------|
| **sruja-chat** | DELETE | Replace with Slack bot |
| **sruja-app** | DELETE | Replace with Slack bot |
| **sruja-watch** | DELETE | On-demand is fine |
| **sruja-export** | MOVE | Single file in MCP, not crate |
| **sruja-cli** | DEFER | Keep minimal, separate repo |
| **sruja-lsp** | DEFER | Separate repo if needed |
| **sruja-wasm** | DEFER | Only if VS Code needed |
| **VS Code** | DEFER | Separate repo |
| **skill-lint** | DEFER | Separate repo |

---

## What's Missing

### 1. Slack Bot Integration 🔴 CRITICAL

```typescript
// slack-bot/src/app.ts - ~100 lines
import { App } from '@slack/bolt';
import { MCPClient } from './mcp-client';

const app = new App({ /* config */ });
const mcp = new MCPClient('http://localhost:3000');

app.event('app_mention', async ({ event, say }) => {
  const response = await mcp.query(event.text);
  await say(response);
});
```

**Why Critical:**
- Primary user interface for AI-first
- Developers already in Slack
- Zero distribution friction

### 2. Enhanced MCP Tools 🔴 CRITICAL

Current: 3 tools
Needed: 15+ tools

```
READ Tools:
- get_architecture_summary ✓
- query_decisions ✓
- check_policies ✓
+ query_dependencies(node)
+ query_impact(node)
+ query_history(node)
+ detect_drift()
+ search_patterns(query)

WRITE Tools:
+ add_node(kind, label, props)
+ add_edge(source, target, kind)
+ add_decision(title, context, decision)
+ update_node(id, props)
+ delete_node(id)

EXTRACTION Tools:
+ extract_from_text(text)
+ extract_from_slack(channel, ts)
+ scan_repository(path)
```

### 3. Pattern Library 🟡 IMPORTANT

```yaml
# patterns/microservices.yml
name: microservices
description: Microservices architecture pattern
indicators:
  - multiple_databases
  - async_communication
  - independent_deployability
anti_patterns:
  - distributed_monolith
  - shared_database
  - synchronous_everything
```

**Why Important:**
- AI needs to recognize patterns
- Enables "Is this an anti-pattern?" queries
- Skills can reference patterns

### 4. Temporal Analysis 🟡 IMPORTANT

```rust
impl KnowledgeGraph {
    fn history(&self, node_id: &str) -> Vec<GraphSnapshot>;
    fn diff(&self, t1: DateTime, t2: DateTime) -> GraphDiff;
    fn rollback(&mut self, to: DateTime);
}
```

**Why Important:**
- "How did we get here?"
- "What changed last week?"
- ADR evolution tracking

### 5. Drift Detection 🟡 IMPORTANT

```rust
impl KnowledgeGraph {
    fn detect_drift(&self, declared: &Program) -> DriftReport;
}

struct DriftReport {
    missing_in_code: Vec<Node>,      // Declared but not found
    missing_in_dsl: Vec<Node>,       // Found but not declared
    relationship_mismatch: Vec<Edge>,
}
```

**Why Important:**
- Documentation vs reality
- Key intelligence insight
- "Your architecture is lying"

---

## What Needs Improvement

### sruja-graph (Major)

| Gap | Solution |
|-----|----------|
| No semantic queries | Add query methods |
| No history | Add temporal versioning |
| Weak provenance | Source tracking on every element |
| No pattern detection | Add pattern matching |

### sruja-mcp (Major)

| Gap | Solution |
|-----|----------|
| Only 3 tools | Add 12+ tools |
| Read-only | Add write tools |
| No sessions | Add conversation tracking |

### sruja-extract (Moderate)

| Gap | Solution |
|-----|----------|
| Accuracy varies | Better prompts |
| Single extraction | Batch extraction |
| No conflict detection | Add dedup/conflict |

### sruja-scan (Moderate)

| Gap | Solution |
|-----|----------|
| 5 languages | Add Java, C#, Kotlin |
| Basic patterns | Add framework detection |
| No DB schema | Add SQL/NoSQL parsing |

### sruja-language (Minor)

| Gap | Solution |
|-----|----------|
| One-way (parse) | Add round-trip (to_dsl) |
| | |

---

## New Architecture (5 Modules + Slack)

```
┌─────────────────────────────────────────────────────────────┐
│                     AI-FIRST SRUJA                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐                                            │
│  │  Slack Bot  │◄── Developer asks questions                │
│  │  (~200 LOC) │                                            │
│  └──────┬──────┘                                            │
│         │                                                   │
│         ▼                                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   sruja-mcp                          │   │
│  │         (15+ tools for AI to use)                   │   │
│  └──────────────────────┬──────────────────────────────┘   │
│                         │                                   │
│         ┌───────────────┼───────────────┐                  │
│         ▼               ▼               ▼                  │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│  │   Graph    │  │  Extract   │  │   Scan     │           │
│  │ (knowledge)│  │  (LLM)     │  │ (code)     │           │
│  └─────┬──────┘  └────────────┘  └────────────┘           │
│        │                                                    │
│        ▼                                                    │
│  ┌────────────┐                                             │
│  │  Language  │  (.sruja files)                            │
│  └────────────┘                                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘

5 crates + 1 Slack bot (vs 16 crates + desktop app)
```

---

## Migration Path

### Phase 1: Stabilize Core (2 weeks)
1. Enhance MCP with 10+ tools
2. Add basic Slack bot
3. Improve extraction accuracy

### Phase 2: Enhance Intelligence (4 weeks)
1. Add temporal to Graph
2. Add drift detection
3. Add pattern library
4. Expand scan languages

### Phase 3: Cleanup (2 weeks)
1. Merge diagnostics into language
2. Reduce engine rules
3. Move export to MCP
4. Archive: chat, app, watch, cli, lsp, wasm, extension

---

## Final Count

| Before | After |
|--------|-------|
| 16 Rust crates | 5 Rust crates |
| Desktop app | Slack bot |
| Multiple interfaces | One interface (MCP) |
| ~15,000 LOC | ~8,000 LOC |

**Focus: AI-first, single interface, maximum intelligence.**
