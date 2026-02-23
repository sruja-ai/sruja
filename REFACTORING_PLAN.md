# Sruja Refactoring Plan

> **DEPRECATED:** This plan has been superseded by [architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md](architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md), which defines the current Architecture Intelligence direction, adoption-first strategy, and module decisions. Kept for historical reference only.

---

## Product Vision

**Sruja = Collaborative Architectural Cognition Layer**

Multi-party architecture conversations that:
1. Capture decisions as they happen
2. Auto-extract ADRs
3. Build queryable knowledge graph
4. Answer "why" questions

```
┌──────────────────────────────────────────────────────┐
│                   MULTI-PARTY CHAT                    │
│  ┌──────────────────────────────────────────────────┐ │
│  │ Alice: We should use Kafka for events           │ │
│  │ Bob: What about RabbitMQ?                       │ │
│  │ Alice: Kafka handles replay better              │ │
│  │ Carol: +1, Kafka for event sourcing             │ │
│  └──────────────────────────────────────────────────┘ │
│                         │                             │
│                         ▼                             │
│  ┌──────────────────────────────────────────────────┐ │
│  │ EXTRACTION ENGINE                                │ │
│  │ → Decision detected (confidence: 0.92)          │ │
│  │ → ADR-007 draft generated                       │ │
│  │ → Knowledge graph updated                       │ │
│  └──────────────────────────────────────────────────┘ │
│                         │                             │
│          ┌──────────────┼──────────────┐              │
│          ▼              ▼              ▼              │
│     ┌─────────┐   ┌──────────┐   ┌───────────┐       │
│     │ Chat UI │   │ MCP/API  │   │  Editors  │       │
│     └─────────┘   └──────────┘   └───────────┘       │
└──────────────────────────────────────────────────────┘
```

---

## Phase 1: Cleanup

### 1.1 Crates to DELETE

```
crates/sruja-proposal/    # Conversations replace markdown proposals
crates/sruja-enrich/      # Generic enrichment - wrong abstraction
```

### 1.2 Crates to KEEP

| Crate | Why |
|-------|-----|
| `sruja-language` | Parse .sruja architecture definitions |
| `sruja-diagnostics` | Error reporting |
| `sruja-engine` | Validation rules (policy reasoning) |
| `sruja-export` | Export to formats |
| `sruja-lsp` | Editor integration (consumes graph) |
| `sruja-wasm` | Web support for Chat UI |
| `sruja-scan` | Phase 2: Drift detection |
| `sruja-diff` | Phase 2: Graph comparison |
| `skill-lint` | Skills validation |

### 1.3 CLI Commands to REMOVE

```rust
// DELETE these commands from sruja-cli
Init { ... }
Generate { ... }
Change { ... }
Score { ... }
Skills { ... }
Review { ... }  // depends on deleted sruja-proposal
```

### 1.4 Directories/Files to DELETE

```
index.html               # Static demo
internal-docs/           # Move useful bits to docs/
test-outputs/            # Generated artifacts
log.log, run.log, outdated.log   # Log files
.playwright-mcp/         # Not related
```

### 1.5 Directories to KEEP

```
book/                    # mdBook documentation - KEEP for product docs
docs/                    # Language spec, guides
```

### 1.5 Examples to CONSOLIDATE

Keep ~10 examples, delete the rest:
```
examples/
├── basic-system.sruja
├── microservices.sruja
├── ecommerce.sruja
└── tutorial/
    ├── 01-system.sruja
    ├── 02-containers.sruja
    └── 03-databases.sruja
```

---

## Phase 2: New Crates

### 2.1 `sruja-chat` - Multi-Party Conversation System

**Purpose:** Real-time architecture discussions

```rust
pub struct ChatSession {
    pub id: SessionId,
    pub topic: String,
    pub participants: Vec<Participant>,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub status: SessionStatus,  // Active, Closed
}

pub struct Message {
    pub id: MessageId,
    pub session_id: SessionId,
    pub author: Participant,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub extractions: Vec<ExtractionId>,  // Link to extracted items
}

pub struct Participant {
    pub id: String,
    pub name: String,
    pub role: ParticipantRole,  // Owner, Contributor, Observer
}
```

**API:**
```rust
impl ChatServer {
    pub async fn create_session(&self, topic: &str) -> SessionId;
    pub async fn join_session(&self, session: SessionId, participant: Participant);
    pub async fn send_message(&self, session: SessionId, message: NewMessage);
    pub async fn get_history(&self, session: SessionId) -> Vec<Message>;
}
```

### 2.2 `sruja-extract` - Architecture Extraction Engine

**Purpose:** Extract structured knowledge from conversations

```rust
pub enum Intent {
    Decision,      // "We should use Kafka"
    Requirement,   // "The system must handle 10k rps"
    Constraint,    // "Frontend can't call database directly"
    Policy,        // "All APIs need authentication"
    Risk,          // "This creates a single point of failure"
    Question,      // "Why are we using Redis?"
    Tradeoff,      // "We're trading latency for consistency"
}

pub struct Extraction {
    pub id: ExtractionId,
    pub intent: Intent,
    pub confidence: f32,
    pub content: ExtractedContent,
    pub source: SourceReference,  // Which message
    pub status: ExtractionStatus, // Draft, Confirmed, Rejected
}

pub enum ExtractedContent {
    Decision {
        title: String,
        context: String,
        decision: String,
        alternatives: Vec<String>,
        consequences: Vec<String>,
    },
    Constraint {
        source: String,       // "Frontend"
        target: String,       // "Database"
        constraint: String,   // "cannot call directly"
    },
    // ... other variants
}

// Detect if a decision is ratified
pub fn detect_ratification(messages: &[Message], extraction: &Extraction) -> RatificationStatus {
    // Look for +1, "agreed", "approved", "sounds good", etc.
}
```

**Extraction Pipeline:**
```
Message → Intent Classifier → Content Extractor → Confidence Score → Ratification Check
                                                                              │
                                                                              ▼
                                                                    Knowledge Graph Update
```

### 2.3 `sruja-graph` - Architecture Knowledge Graph

**Purpose:** Store and query architecture knowledge

```rust
pub struct KnowledgeGraph {
    // Architecture elements
    pub systems: HashMap<SystemId, System>,
    pub services: HashMap<ServiceId, Service>,
    pub databases: HashMap<DatabaseId, Database>,
    pub external_apis: HashMap<ApiId, ExternalApi>,
    
    // Decisions
    pub decisions: HashMap<DecisionId, Decision>,
    
    // Policies & Constraints
    pub policies: HashMap<PolicyId, Policy>,
    pub constraints: HashMap<ConstraintId, Constraint>,
    
    // Requirements
    pub requirements: HashMap<RequirementId, Requirement>,
    
    // Relationships
    pub edges: Vec<Edge>,
}

pub struct Decision {
    pub id: DecisionId,
    pub title: String,
    pub status: DecisionStatus,  // Proposed, Accepted, Deprecated, Superseded
    pub context: String,
    pub decision: String,
    pub alternatives: Vec<String>,
    pub consequences: String,
    pub created_at: DateTime<Utc>,
    pub ratified_at: Option<DateTime<Utc>>,
    pub source: SourceReference,  // Conversation, ADR file, etc.
    pub affects: Vec<String>,     // What components this impacts
}

pub enum SourceReference {
    Conversation { session_id: String, message_ids: Vec<String> },
    AdrFile { path: String },
    DslFile { path: String, line: u32 },
}

// Query interface
impl KnowledgeGraph {
    pub fn query(&self, question: &str) -> QueryResult;
    pub fn get_decision(&self, id: &DecisionId) -> Option<&Decision>;
    pub fn get_component_decisions(&self, component: &str) -> Vec<&Decision>;
    pub fn find_policy_violations(&self) -> Vec<PolicyViolation>;
}
```

### 2.4 `sruja-mcp` - MCP Server

**Purpose:** Expose knowledge graph to editors, agents, CI

```rust
// MCP Tools
pub enum SrujaTool {
    GetArchitecture,
    GetDecision { id: String },
    GetDecisions,
    GetPolicyConflicts,
    Query { question: String },
    GetComponent { id: String },
}

// Endpoints
GET  /architecture          # Full graph
GET  /decisions             # All decisions
GET  /decision/{id}         # Specific decision
GET  /policies              # All policies
GET  /policy/conflicts      # Violations
POST /query                 # Natural language query
```

---

## Phase 3: Chat UI (Web)

### Tech Stack
- React + TypeScript
- WebSocket for real-time
- Tailwind CSS
- Graph visualization (D3/Cytoscape)

### Key Views
1. **Chat View** - Multi-party conversation
2. **Decisions View** - Extracted ADRs with status
3. **Graph View** - Visual architecture
4. **Query View** - Ask questions

### Features
- Real-time message sync
- Decision highlighting in chat
- Draft ADR sidebar
- Confidence indicators
- Manual confirmation/rejection of extractions

---

## Updated Cargo.toml

```toml
[workspace]
members = [
    # Core (keep)
    "crates/sruja-diagnostics",
    "crates/sruja-language",
    "crates/sruja-engine",
    "crates/sruja-export",
    "crates/sruja-lsp",
    "crates/sruja-cli",
    "crates/sruja-scan",
    "crates/sruja-diff",
    "crates/sruja-wasm",
    "crates/skill-lint",
    # New (build)
    "crates/sruja-chat",
    "crates/sruja-extract",
    "crates/sruja-graph",
    "crates/sruja-mcp",
]
```

---

## Implementation Order

### Sprint 1: Cleanup (1 day)
- [ ] Delete `sruja-proposal`, `sruja-enrich`
- [ ] Remove CLI commands: init, generate, change, score, skills, review
- [ ] Delete `book/`, `index.html`, `internal-docs/`, `test-outputs/`
- [ ] Update `Cargo.toml`
- [ ] Verify build passes

### Sprint 2: Knowledge Graph (3 days)
- [ ] Create `sruja-graph` crate
- [ ] Define schema (System, Service, Decision, Policy, etc.)
- [ ] Implement CRUD operations
- [ ] JSON serialization
- [ ] Query interface stub

### Sprint 3: Extraction Engine (5 days)
- [ ] Create `sruja-extract` crate
- [ ] Intent classification (pattern-based first)
- [ ] Decision extractor
- [ ] Constraint extractor
- [ ] Ratification detection
- [ ] Integration with `sruja-graph`

### Sprint 4: Chat System (4 days)
- [ ] Create `sruja-chat` crate
- [ ] WebSocket server
- [ ] Session management
- [ ] Message storage
- [ ] Hook into extraction pipeline

### Sprint 5: MCP Server (2 days)
- [ ] Create `sruja-mcp` crate
- [ ] Implement endpoints
- [ ] MCP protocol compliance

### Sprint 6: Chat UI (5 days)
- [ ] React app skeleton
- [ ] WebSocket client
- [ ] Chat view
- [ ] Decisions sidebar
- [ ] Basic styling

---

## Success Criteria

### MVP Ready When:
- [ ] Multi-user chat works in browser
- [ ] Decisions are auto-detected from conversation
- [ ] ADR drafts appear in sidebar
- [ ] Users can confirm/reject extractions
- [ ] Query "Why did we choose Kafka?" returns answer

### PMF Signal:
- [ ] Team uses Sruja for architecture discussions
- [ ] ADRs are created without manual writing
- [ ] New team members can query architecture history
