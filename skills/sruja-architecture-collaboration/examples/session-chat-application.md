# Real-Time Chat Application - Collaborative Architecture Session

This example demonstrates a full multi-agent collaboration session for designing a real-time chat application.

## Session Log

### @facilitator: Session Start

```
Starting architecture session: "Real-Time Chat Application"

Participants: @analyst, @architect, @reviewer, @validator
Goal: Design architecture for real-time chat with messaging, groups, and file attachments
Timebox: 2 hours
```

---

### @analyst: Requirements Analysis

```markdown
## Architecture Analysis Report

### Functional Requirements
- FR-001: Real-time messaging with < 100ms delivery
- FR-002: Group chats with up to 100 members
- FR-003: Message history and search
- FR-004: File attachments up to 25MB
- FR-005: Typing indicators and presence

### Non-Functional Requirements
- NFR-001: 99.9% uptime
- NFR-002: Support 100K concurrent users
- NFR-003: End-to-end encryption for messages
- NFR-004: GDPR compliant (data residency, right to erasure)

### Constraints
- Mobile-first (must work on unreliable networks)
- Team expertise: Node.js, PostgreSQL, Redis
- Budget: Use existing AWS infrastructure
- Timeline: MVP in 3 months

### Current State
- No existing chat infrastructure
- User authentication exists (Auth0 integration)
- File storage exists (S3 bucket)

### Key Concerns
1. WebSocket connectivity on mobile networks
2. Message ordering guarantees
3. Encryption key management
4. Cost at scale (100K users)
```

---

### @architect: Initial Proposal

```sruja
system "Chat Application" {
  description "Real-time chat platform with messaging, groups, and file sharing"
  
  metadata {
    status "proposed"
    author "solution-architect"
    version "1.0.0"
    session "real-time-chat-2025-01-15"
  }
  
  // Connection Layer
  connection = container "Connection Manager" {
    technology "Node.js + Socket.io"
    description "WebSocket connections with HTTP long-polling fallback"
    
    metadata {
      addresses ["FR-001", "NFR-002"]
      fallback "HTTP long-polling for mobile networks"
      scaling "Horizontal with sticky sessions"
    }
  }
  
  // Message Layer
  message_api = container "Message API" {
    technology "Node.js"
    description "REST API for message operations and history"
    
    metadata {
      addresses ["FR-003"]
      endpoints ["POST /messages", "GET /messages", "GET /search"]
    }
  }
  
  message_worker = container "Message Worker" {
    technology "Node.js"
    description "Background processing for message delivery and encryption"
    
    metadata {
      addresses ["FR-001", "NFR-003"]
      responsibilities ["encryption", "delivery", "notifications"]
    }
  }
  
  // Group Layer
  group_api = container "Group API" {
    technology "Node.js"
    description "Group management and membership"
    
    metadata {
      addresses ["FR-002"]
      max_members 100
    }
  }
  
  // File Layer
  file_api = container "File API" {
    technology "Node.js"
    description "File upload and attachment handling"
    
    metadata {
      addresses ["FR-004"]
      max_size "25MB"
      storage "S3"
    }
  }
  
  // Presence Layer
  presence = container "Presence Service" {
    technology "Node.js + Redis Pub/Sub"
    description "Online status and typing indicators"
    
    metadata {
      addresses ["FR-005"]
      heartbeat_interval "30s"
    }
  }
  
  // Data Layer
  message_db = datastore "Message Database" {
    technology "PostgreSQL"
    description "Message storage with encryption at rest"
    
    metadata {
      addresses ["FR-003", "NFR-003", "NFR-004"]
      encryption "pgcrypto"
      retention "7 years"
    }
  }
  
  group_db = datastore "Group Database" {
    technology "PostgreSQL"
    description "Group and membership data"
    
    metadata {
      addresses ["FR-002"]
    }
  }
  
  cache = datastore "Session Cache" {
    technology "Redis"
    description "Active sessions, presence, and hot data"
    
    metadata {
      addresses ["FR-001", "FR-005", "NFR-002"]
      use_cases ["presence", "typing_indicators", "message_cache"]
    }
  }
  
  queue = datastore "Message Queue" {
    technology "Redis Streams"
    description "Message delivery queue with ordering guarantees"
    
    metadata {
      addresses ["FR-001"]
      ordering "per-conversation FIFO"
    }
  }
  
  file_storage = datastore "File Storage" {
    technology "AWS S3"
    description "File attachments with virus scanning"
    
    metadata {
      addresses ["FR-004"]
      existing true
    }
  }
  
  // Internal Relationships
  connection -> message_api "REST - history queries"
  connection -> presence "Redis Pub/Sub - presence updates"
  connection -> queue "Redis Streams - consume messages"
  
  message_api -> message_db "SQL - message CRUD"
  message_api -> group_api "REST - group validation"
  message_api -> cache "Redis - message cache"
  
  message_worker -> queue "Redis Streams - process messages"
  message_worker -> message_db "SQL - persist messages"
  message_worker -> connection "WebSocket - push delivery"
  
  group_api -> group_db "SQL - group management"
  group_api -> cache "Redis - group membership cache"
  
  file_api -> file_storage "S3 - file upload/download"
  file_api -> message_api "REST - create attachment message"
  
  presence -> cache "Redis - presence state"
}

external_system "Auth0" {
  description "Existing authentication service"
  
  container "OAuth Provider" {
    technology "OAuth 2.0"
  }
}

external_system "AWS CloudFront" {
  description "CDN for static assets and file delivery"
  
  container "CDN" {
    technology "CloudFront"
  }
}

// Cross-system relationships
chat_application.connection -> auth0.oauth_provider "JWT validation"
chat_application.file_api -> chat_application.file_storage "S3 SDK"
chat_application.file_storage -> aws_cloudfront.cdn "S3 origin"

// End Users
person "Chat User" {
  description "End user sending and receiving messages"
  
  metadata {
    segment "mobile-first"
    behaviors ["send messages", "join groups", "share files", "search history"]
  }
}

person "Group Admin" {
  description "User managing group membership"
  
  metadata {
    segment "power-user"
    behaviors ["create groups", "manage members", "set permissions"]
  }
}

chat_user -> chat_application.connection "WebSocket/HTTPS - real-time messaging"
chat_user -> chat_application.message_api "HTTPS - history search"
chat_user -> aws_cloudfront.cdn "HTTPS - file downloads"

group_admin -> chat_application.group_api "HTTPS - group management"
```

---

### @reviewer: Architecture Review

```markdown
## Architecture Review Report

**Proposal**: Chat Application v1.0.0
**Reviewer**: architecture-reviewer
**Date**: 2025-01-15
**Recommendation**: APPROVE WITH CONDITIONS

### Summary
Well-structured real-time chat architecture with appropriate separation of concerns.
Mobile connectivity addressed with fallback. Some concerns around encryption and scaling.

### Strengths
1. ✅ Clear service boundaries aligned with capabilities
2. ✅ WebSocket + HTTP fallback addresses mobile constraint
3. ✅ Redis Streams for ordered message delivery
4. ✅ Leveraging existing Auth0 and S3
5. ✅ Presence service separated for independent scaling

### Concerns

#### HIGH: Encryption Key Management Not Addressed
- **Issue**: NFR-003 requires E2E encryption, but key management not specified
- **Impact**: Cannot guarantee true E2E encryption without key strategy
- **Suggestion**: Add key management service or document client-side encryption approach

#### HIGH: Single Point of Failure in Connection Manager
- **Issue**: All WebSocket connections go through connection manager
- **Impact**: If connection manager fails, all users disconnected
- **Suggestion**: Deploy with multi-AZ, add connection draining, document reconnection logic

#### MEDIUM: Message Ordering Across Partitions
- **Issue**: Redis Streams per-conversation ordering may have race conditions
- **Impact**: Messages could appear out of order in high-velocity conversations
- **Suggestion**: Add client-side message ordering or use monotonically increasing IDs

#### MEDIUM: Cost at Scale Not Analyzed
- **Issue**: 100K concurrent users not costed
- **Impact**: May exceed budget constraint
- **Suggestion**: Add cost estimation worksheet

#### LOW: Missing Observability
- **Issue**: No metrics, tracing, or logging strategy
- **Impact**: Difficult to debug production issues
- **Suggestion**: Add OpenTelemetry instrumentation

### Anti-Patterns Detected
- None detected ✅

### Alignment with Principles
- ✅ Separation of concerns
- ✅ Independent scaling
- ✅ Mobile-first design
- ⚠️ Resilience (needs HA for connection manager)
- ⚠️ Security (key management unclear)

### Conditions for Approval
1. Document encryption key management approach
2. Add HA strategy for Connection Manager
3. Clarify message ordering guarantees

### Validated With
```bash
sruja lint chat-application.sruja
# ✓ No errors found
```
```

---

### @architect: Addressing Feedback

```sruja
system "Chat Application" {
  description "Real-time chat platform with messaging, groups, and file sharing"
  
  metadata {
    status "revised"
    author "solution-architect"
    version "1.1.0"
    session "real-time-chat-2025-01-15"
    changes "Addressed HIGH feedback from review"
  }
  
  // ... [previous components remain the same] ...
  
  // NEW: Key Management
  key_service = container "Key Service" {
    technology "Node.js + AWS KMS"
    description "Encryption key management for E2E encryption"
    
    metadata {
      addresses ["NFR-003"]
      approach "Client-generated keys, KMS-wrapped for storage"
      key_rotation "90 days"
    }
  }
  
  // ... [other components] ...
  
  // Internal Relationships (updated)
  message_worker -> key_service "AWS KMS - unwrap keys for processing"
  key_service -> message_db "Encrypted key references"
}
```

**Revision Notes**:

1. **Encryption Key Management** (Addressing HIGH #1)
   - Added Key Service using AWS KMS
   - Client-side encryption: Client generates keys, encrypts messages
   - Server stores wrapped keys, never sees plaintext
   - Key rotation every 90 days via KMS

2. **Connection Manager HA** (Addressing HIGH #2)
   - Deploy across 3 AZs with auto-scaling
   - Use Redis for session state (already in design)
   - Client SDK handles reconnection with exponential backoff
   - Connection draining on 30-second timeout

3. **Message Ordering** (Addressing MEDIUM #1)
   - Server-assigned monotonically increasing IDs per conversation
   - Client sorts by server ID, not timestamp
   - Redis Streams consumer groups ensure single delivery

---

### @validator: Final Validation

```markdown
## Validation Report

### Syntax Validation
```bash
sruja lint chat-application.sruja
# ✓ No errors found
```

### Completeness Score: 100%

| Category | Status |
|----------|--------|
| Functional Requirements | 5/5 ✅ |
| Non-Functional Requirements | 4/4 ✅ |
| Components | All described ✅ |
| Relationships | All labeled ✅ |
| Technologies | All specified ✅ |
| Metadata | Complete ✅ |

### Constraint Compliance: 100%

| Constraint | Status | Evidence |
|------------|--------|----------|
| Mobile-first | ✅ | WebSocket + HTTP fallback |
| Team expertise | ✅ | Node.js, PostgreSQL, Redis |
| AWS infrastructure | ✅ | S3, KMS, CloudFront |
| 3-month timeline | ✅ | Standard stack, existing auth/storage |

### Requirements Traceability

| Requirement | Component | Decision | Status |
|-------------|-----------|----------|--------|
| FR-001 Real-time | connection, message_worker | WebSocket + fallback | ✅ |
| FR-002 Groups | group_api, group_db | PostgreSQL | ✅ |
| FR-003 History | message_api, message_db | PostgreSQL + full-text search | ✅ |
| FR-004 Files | file_api, file_storage | S3 | ✅ |
| FR-005 Presence | presence, cache | Redis Pub/Sub | ✅ |
| NFR-001 Uptime | All | Multi-AZ, HA | ✅ |
| NFR-002 Scale 100K | connection, cache | Horizontal scaling | ✅ |
| NFR-003 Encryption | key_service, message_worker | AWS KMS, E2E | ✅ |
| NFR-004 GDPR | message_db | Data residency, retention | ✅ |

### Quality Gate

| Gate | Threshold | Result | Status |
|------|-----------|--------|--------|
| Syntax | 100% | 100% | ✅ PASS |
| Completeness | >= 90% | 100% | ✅ PASS |
| Constraints | 100% | 100% | ✅ PASS |
| Requirements | 100% | 100% | ✅ PASS |

### Issues Addressed

| Issue | Severity | Status |
|-------|----------|--------|
| Key management | HIGH | ✅ Fixed |
| Connection HA | HIGH | ✅ Fixed |
| Message ordering | MEDIUM | ✅ Fixed |

**Overall Status**: ✅ APPROVED
```

---

### @facilitator: Session Summary

```markdown
## Session Complete

### Summary
- **Architecture**: chat-application.sruja v1.1.0
- **Duration**: 1 hour 45 minutes
- **Outcome**: ✅ APPROVED

### Key Decisions
1. **ADR-001**: WebSocket + HTTP long-polling for mobile reliability
2. **ADR-002**: Redis Streams for ordered message delivery
3. **ADR-003**: Client-side E2E encryption with AWS KMS key wrapping
4. **ADR-004**: PostgreSQL for message persistence (GDPR compliance)
5. **ADR-005**: Horizontal scaling for connection managers

### Artifacts Created
- `chat-application.sruja` - Architecture definition
- `ADR-001.md` through `ADR-005.md` - Decision records
- `review-report.md` - Architecture review
- `validation-report.md` - Validation report

### Next Steps
1. Tech Lead approval (pending)
2. Security review for encryption approach
3. Cost estimation for 100K users
4. Create implementation tickets
5. Schedule sprint planning

### Session Archive
Archived to: `sessions/2025-01-15-real-time-chat/`
```
