# Teams, Ownership, and Proposal Workflow

## Overview

Architecture changes are proposed by teams who own systems/containers. Multiple proposals can exist in parallel, go through review/modification, and eventually get committed as architecture changes.

## Core Concepts

### 1. Teams and Ownership

**Teams own architectural elements** (systems, containers, components):
- Teams are defined in metadata
- Ownership is tracked via metadata on elements
- Teams can propose changes to elements they own
- Multiple teams can collaborate on changes

### 2. Proposals

**Proposals are pending architecture changes**:
- Multiple proposals can exist in parallel
- Each proposal is owned by a team
- Proposals go through review/modification cycle
- Once approved, proposals are committed as changes

### 3. Proposal Workflow

1. **Create Proposal** - Team creates proposal for changes
2. **Review** - Other teams/stakeholders review (comments, questions, concerns)
3. **Discuss** - Discussions capture questions/concerns, action items track tasks
4. **Modify** - Proposal can be modified during review (relations, tech choices)
5. **Approve** - Stakeholders approve proposal (all action items addressed)
6. **Commit** - Approved proposal becomes architecture change (linked to proposal)

### 4. Discussions and Action Items Live in Proposals

**Why discussions/action items live in proposals**:
- **Context**: Full discussion history is part of proposal evolution
- **Traceability**: Shows why decisions were made and how concerns were addressed
- **State tracking**: Easy to see what's open, what's resolved, what's pending
- **Historical record**: When proposal becomes a change, the full context is preserved
- **Action items**: Track what needs to be done to move proposal forward

**What's captured**:
- **Discussions**: Comments, questions, concerns from stakeholders
- **Responses**: Answers and clarifications
- **Action items**: Tasks to address concerns/questions
- **Status tracking**: Open/answered/resolved for discussions, pending/completed for action items

## Teams and Ownership Model

### Team Ownership

Teams own elements via metadata on each element (simple, no duplication):

```sruja
architecture "E-commerce Platform" {
  system ShopSystem {
    metadata {
      team "Platform Team"
    }
    // ...
  }
  
  container PaymentSystem.PaymentAPI {
    metadata {
      team "Payment Team"
    }
    // ...
  }
}
```

**Note**: Teams can optionally be documented in architecture metadata for reference, but ownership is tracked directly on elements. In JSON, we only store team ID on elements (no duplication).

### Ownership via Metadata

Elements have team ownership in metadata (no duplication - just team ID):

```json
{
  "architecture": {
    "systems": [
      {
        "id": "ShopSystem",
        "metadata": {
          "team": "Platform Team"
        }
      }
    ],
    "containers": [
      {
        "id": "PaymentSystem.PaymentAPI",
        "metadata": {
          "team": "Payment Team"
        }
      }
    ]
  }
}
```

**No root-level teams array** - team ownership is tracked directly on elements via metadata.

## Proposal Structure

### Proposal File

Proposals are stored as separate files:

```
architecture/
  ├── proposals/
  │   ├── PROP-001-add-analytics.md          # Proposal document
  │   ├── PROP-001-add-analytics.sruja       # Proposed changes
  │   ├── PROP-002-refactor-payment.md
  │   └── PROP-002-refactor-payment.sruja
  ├── changes/
  │   ├── 001-add-analytics.sruja            # Committed from PROP-001
  │   └── 002-refactor-payment.sruja         # Committed from PROP-002
  └── current.sruja
```

### Proposal DSL Syntax

```sruja
proposal "PROP-001" "Add Analytics Dashboard" {
  team "Analytics Team"
  requirement "REQ-123"
  userStory "US-456"
  
  status "in-review"  // draft, in-review, approved, rejected, committed
  
  // Proposed changes
  add {
    system ShopSystem {
      container AnalyticsAPI "Analytics API" {
        component Dashboard "Dashboard"
        component MetricsCollector "Metrics Collector"
      }
    }
  }
  
  modify {
    system ShopSystem {
      // Add new relation
      WebApp -> AnalyticsAPI "Sends analytics data"
    }
  }
  
  // Technology choices (can be modified during review)
  decisions {
    adr "ADR-003" "Use React for dashboard UI" {
      status "proposed"
      rationale "Team expertise in React"
    }
  }
  
  // Impact on other teams
  impacts {
    system "ShopSystem" {
      team "Platform Team"
      changes "New API endpoint, new container"
    }
  }
  
  // Iteration history - tracks how proposal evolved
  iterations [
    {
      version 1
      date "2025-01-10"
      description "Initial proposal"
      changes [
        "Added AnalyticsAPI container"
        "Proposed REST API"
      ]
    }
    {
      version 2
      date "2025-01-15"
      description "Updated based on review feedback"
      changes [
        "Changed from REST to GraphQL API"
        "Added authentication layer"
      ]
      triggeredBy {
        reviewer "alice@example.com"
        team "Platform Team"
        comment "Suggest using GraphQL instead of REST"
      }
    }
  ]
  
  // Decision evolution - tracks how choices changed
  decisionEvolution [
    {
      decision "ADR-003" "API Protocol Choice"
      iterations [
        {
          version 1
          date "2025-01-10"
          choice "REST API"
          rationale "Team expertise in REST"
          status "proposed"
        }
        {
          version 2
          date "2025-01-15"
          choice "GraphQL API"
          rationale "Better for analytics queries, suggested by Platform Team"
          status "approved"
          changedBy {
            reviewer "alice@example.com"
            reason "Platform Team prefers GraphQL for consistency"
          }
        }
      ]
    }
  ]
  
  // Discussion threads - comments, questions, concerns
  discussions [
    {
      id "DISC-001"
      reviewer "alice@example.com"
      team "Platform Team"
      date "2025-01-12"
      type "question"  // question, comment, concern, suggestion
      topic "API Protocol"
      message "What protocol are you planning to use for the analytics API? REST or GraphQL?"
      status "answered"  // open, answered, resolved
      resolvedDate "2025-01-15"
      resolvedInIteration 2
      responses [
        {
          author "diana@example.com"
          team "Analytics Team"
          date "2025-01-15"
          message "We'll use GraphQL as suggested by Platform Team for consistency"
        }
      ]
    }
    {
      id "DISC-002"
      reviewer "bob@example.com"
      team "Security Team"
      date "2025-01-13"
      type "concern"
      topic "Authentication"
      message "How will authentication be handled? Need to ensure OAuth2 compliance"
      status "open"
      actionItems [
        {
          id "AI-001"
          description "Add authentication layer with OAuth2"
          assignedTo "diana@example.com"
          status "pending"
        }
      ]
    }
  ]
  
  // Action items - tasks to address
  actionItems [
    {
      id "AI-001"
      description "Add authentication layer with OAuth2"
      assignedTo "diana@example.com"
      team "Analytics Team"
      created "2025-01-13"
      due "2025-01-20"
      status "pending"  // pending, in-progress, completed, blocked
      linkedToDiscussion "DISC-002"
      completedInIteration 2
    }
    {
      id "AI-002"
      description "Update API documentation"
      assignedTo "diana@example.com"
      team "Analytics Team"
      created "2025-01-15"
      status "pending"
    }
  ]
  
  // Review comments/feedback (summary/formal reviews)
  reviews [
    {
      reviewer "alice@example.com"
      team "Platform Team"
      status "approved"
      comment "Looks good, suggest using GraphQL instead of REST"
      date "2025-01-15"
      iteration 2  // Which iteration this review refers to
      actionItemsAddressed ["AI-001"]
    }
  ]
  
  // Link to committed change (when approved)
  change "001-add-analytics"
}
```

### Proposal Metadata

```json
{
  "id": "PROP-001",
  "title": "Add Analytics Dashboard",
  "team": "Analytics Team",
  "status": "in-review",
  "created": "2025-01-10T10:00:00Z",
  "updated": "2025-01-15T14:30:00Z",
  "requirement": "REQ-123",
  "userStory": "US-456",
  "proposedChanges": {
    "add": {...},
    "modify": {...},
    "remove": {...}
  },
  "impacts": [
    {
      "element": "ShopSystem",
      "team": "Platform Team",
      "description": "New API endpoint, new container"
    }
  ],
  "discussions": [
    {
      "id": "DISC-001",
      "reviewer": "alice@example.com",
      "team": "Platform Team",
      "date": "2025-01-12T10:00:00Z",
      "type": "question",
      "topic": "API Protocol",
      "message": "What protocol are you planning to use for the analytics API? REST or GraphQL?",
      "status": "answered",
      "resolvedDate": "2025-01-15T14:00:00Z",
      "resolvedInIteration": 2,
      "responses": [
        {
          "author": "diana@example.com",
          "team": "Analytics Team",
          "date": "2025-01-15T14:00:00Z",
          "message": "We'll use GraphQL as suggested by Platform Team for consistency"
        }
      ]
    },
    {
      "id": "DISC-002",
      "reviewer": "bob@example.com",
      "team": "Security Team",
      "date": "2025-01-13T09:00:00Z",
      "type": "concern",
      "topic": "Authentication",
      "message": "How will authentication be handled? Need to ensure OAuth2 compliance",
      "status": "open"
    }
  ],
  "actionItems": [
    {
      "id": "AI-001",
      "description": "Add authentication layer with OAuth2",
      "assignedTo": "diana@example.com",
      "team": "Analytics Team",
      "created": "2025-01-13T09:00:00Z",
      "due": "2025-01-20T00:00:00Z",
      "status": "pending",
      "linkedToDiscussion": "DISC-002",
      "completedInIteration": null
    }
  ],
  "reviews": [
    {
      "reviewer": "alice@example.com",
      "team": "Platform Team",
      "status": "approved",
      "comment": "Looks good, suggest using GraphQL instead of REST",
      "date": "2025-01-15T12:00:00Z",
      "actionItemsAddressed": ["AI-001"]
    }
  ],
  "decisions": [
    {
      "adr": "ADR-003",
      "title": "Use React for dashboard UI",
      "status": "proposed",
      "canModify": true
    }
  ],
  "iterations": [
    {
      "version": 1,
      "date": "2025-01-10T10:00:00Z",
      "description": "Initial proposal",
      "changes": [
        "Added AnalyticsAPI container",
        "Proposed REST API"
      ]
    },
    {
      "version": 2,
      "date": "2025-01-15T14:30:00Z",
      "description": "Updated based on review feedback",
      "changes": [
        "Changed from REST to GraphQL API",
        "Added authentication layer"
      ],
      "triggeredBy": {
        "reviewer": "alice@example.com",
        "team": "Platform Team",
        "comment": "Suggest using GraphQL instead of REST"
      }
    }
  ],
  "decisionEvolution": [
    {
      "decision": "ADR-003",
      "title": "API Protocol Choice",
      "iterations": [
        {
          "version": 1,
          "date": "2025-01-10T10:00:00Z",
          "choice": "REST API",
          "rationale": "Team expertise in REST",
          "status": "proposed"
        },
        {
          "version": 2,
          "date": "2025-01-15T14:30:00Z",
          "choice": "GraphQL API",
          "rationale": "Better for analytics queries, suggested by Platform Team",
          "status": "approved",
          "changedBy": {
            "reviewer": "alice@example.com",
            "reason": "Platform Team prefers GraphQL for consistency"
          }
        }
      ]
    }
  ],
  "change": "001-add-analytics"  // Link to committed change
}
```

## Proposal Workflow

### 1. Create Proposal

Team creates proposal for changes to elements they own:

```bash
sruja proposal create "Add Analytics Dashboard" \
  --team "Analytics Team" \
  --requirement "REQ-123"
```

Creates: `proposals/PROP-001-add-analytics.sruja`

### 2. Review Proposal

Other teams review proposals that impact their elements:

```bash
sruja proposal review PROP-001 \
  --reviewer "alice@example.com" \
  --team "Platform Team" \
  --status "approved" \
  --comment "Looks good, suggest using GraphQL instead of REST"
```

### 3. Modify Proposal

During review, proposals can be modified:

```bash
sruja proposal modify PROP-001 \
  --change "Use GraphQL instead of REST" \
  --update-file proposals/PROP-001-add-analytics.sruja
```

Modifications can include:
- Changing relations
- Modifying technology choices (ADRs)
- Adjusting proposed elements
- Updating scope

### 4. Approve Proposal

Once all reviews are approved:

```bash
sruja proposal approve PROP-001
```

### 5. Commit Proposal

Approved proposal is committed as change (change is linked back to proposal):

```bash
sruja proposal commit PROP-001 --change-name "001-add-analytics"
```

Creates: `changes/001-add-analytics.sruja` from approved proposal. The change file references the proposal ID.

## Parallel Proposals

Multiple proposals can exist in parallel:

```
proposals/
  ├── PROP-001-add-analytics.sruja      # Analytics Team
  ├── PROP-002-refactor-payment.sruja   # Payment Team
  ├── PROP-003-add-inventory.sruja      # Platform Team
  └── PROP-004-improve-search.sruja     # Platform Team
```

**Conflict Detection**:
- Proposals modifying same elements are flagged
- Visual diff shows conflicts
- Teams coordinate to resolve conflicts

## JSON Structure for Teams

**Simplified**: No root-level teams array - team ownership is tracked directly on elements:

```json
{
  "metadata": {
    "name": "E-commerce Platform"
  },
  "architecture": {
    "systems": [
      {
        "id": "ShopSystem",
        "metadata": {
          "team": "Platform Team"
        }
      },
      {
        "id": "PaymentSystem",
        "metadata": {
          "team": "Payment Team"
        }
      }
    ],
    "containers": [
      {
        "id": "PaymentSystem.PaymentAPI",
        "metadata": {
          "team": "Payment Team"
        }
      }
    ]
  }
}
```

**Benefits**:
- No duplication - team ownership is only on elements
- Simple - just team ID in element metadata
- Easy to query - filter elements by team

## Proposal Commands

### Create Proposal
```bash
sruja proposal create <title> \
  --team <team-name> \
  --requirement <req-id> \
  --user-story <story-id>
```

### List Proposals
```bash
sruja proposal list
sruja proposal list --team "Analytics Team"
sruja proposal list --status "in-review"
```

### View Proposal
```bash
sruja proposal view PROP-001
sruja proposal diff PROP-001  # Show proposed changes
```

### Review Proposal
```bash
sruja proposal review PROP-001 \
  --reviewer <email> \
  --team <team-name> \
  --status <approved|rejected|changes-requested> \
  --comment <comment>
```

### Modify Proposal
```bash
sruja proposal modify PROP-001 \
  --change <description> \
  --update-file <file>
```

### Approve Proposal
```bash
sruja proposal approve PROP-001
```

### Commit Proposal
```bash
sruja proposal commit PROP-001 --change-name <name>
```

### Check Conflicts
```bash
sruja proposal conflicts PROP-001
sruja proposal conflicts --all  # Check all proposals
```

## Integration with Change Visualization

Proposals are visualized:
- Show pending proposals in timeline
- Visual diff for each proposal
- Highlight conflicts between proposals
- Show team ownership on elements
- Filter by team
 - Build proposals visually in Studio and export/import with CLI

## Acceptance Criteria

* [ ] Teams can be defined in architecture metadata
* [ ] Elements have team ownership in metadata
* [ ] Proposals can be created by teams
* [ ] Multiple proposals can exist in parallel
* [ ] Proposals can be reviewed by other teams
* [ ] Proposals can be modified during review
* [ ] Technology choices (ADRs) in proposals can be modified
* [ ] Conflict detection works for parallel proposals
* [ ] Approved proposals can be committed as changes
* [ ] Team ownership visualized in diagrams
* [ ] Proposals visible in change timeline
