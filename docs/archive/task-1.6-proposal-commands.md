# Task 1.6: Proposal Commands and Workflow

**Priority**: 🟡 High (Enables collaborative change management)
**Technology**: Go
**Estimated Time**: 3-4 days
**Dependencies**: Task 1.5 (Change Commands), Teams model

## Overview

CLI commands for managing architecture proposals: create, review, modify, approve, and commit proposals. Supports parallel proposals and team-based ownership.

## Commands

### 1. Create Proposal

```bash
# Create new proposal
sruja proposal create <title> \
  --team <team-name> \
  --requirement <req-id> \
  --user-story <story-id>

# Example
sruja proposal create "Add Analytics Dashboard" \
  --team "Analytics Team" \
  --requirement "REQ-123"
# Creates: proposals/PROP-001-add-analytics.sruja
```

**Output**: Creates proposal file with template:
```sruja
proposal "PROP-001" "Add Analytics Dashboard" {
  team "Analytics Team"
  requirement "REQ-123"
  status "draft"
  
  add {
    // Proposed additions
  }
  
  modify {
    // Proposed modifications
  }
}
```

### 2. List Proposals

```bash
# List all proposals
sruja proposal list

# Filter by team
sruja proposal list --team "Analytics Team"

# Filter by status
sruja proposal list --status "in-review"

# Filter by element
sruja proposal list --element "ShopSystem"
```

### 3. View Proposal

```bash
# View proposal details
sruja proposal view PROP-001

# Show proposed changes (diff)
sruja proposal diff PROP-001

# Show proposal with current architecture
sruja proposal preview PROP-001
```

### 4. Review Proposal (Comments, Questions, Concerns)

```bash
# Add comment/question/concern
sruja proposal comment PROP-001 \
  --reviewer <email> \
  --team <team-name> \
  --type <question|comment|concern|suggestion> \
  --topic <topic> \
  --message <message>

# Example: Ask a question
sruja proposal comment PROP-001 \
  --reviewer "alice@example.com" \
  --team "Platform Team" \
  --type "question" \
  --topic "API Protocol" \
  --message "What protocol are you planning to use? REST or GraphQL?"

# Example: Raise a concern
sruja proposal comment PROP-001 \
  --reviewer "bob@example.com" \
  --team "Security Team" \
  --type "concern" \
  --topic "Authentication" \
  --message "How will authentication be handled? Need OAuth2 compliance"

# Respond to a comment/question
sruja proposal respond PROP-001 \
  --discussion-id DISC-001 \
  --author <email> \
  --message <response>

# Example
sruja proposal respond PROP-001 \
  --discussion-id DISC-001 \
  --author "diana@example.com" \
  --message "We'll use GraphQL as suggested by Platform Team"
```

### 4.1. Action Items

```bash
# Create action item from discussion
sruja proposal action-item PROP-001 \
  --create \
  --description <description> \
  --assigned-to <email> \
  --due-date <date> \
  --linked-discussion <discussion-id>

# Example
sruja proposal action-item PROP-001 \
  --create \
  --description "Add authentication layer with OAuth2" \
  --assigned-to "diana@example.com" \
  --due-date "2025-01-20" \
  --linked-discussion DISC-002

# Update action item status
sruja proposal action-item PROP-001 \
  --update AI-001 \
  --status <pending|in-progress|completed|blocked>

# List action items
sruja proposal action-items PROP-001
```

### 4.2. Formal Review

```bash
# Add formal review (summary/recommendation)
sruja proposal review PROP-001 \
  --reviewer <email> \
  --team <team-name> \
  --status <approved|rejected|changes-requested> \
  --comment <comment>

# Example
sruja proposal review PROP-001 \
  --reviewer "alice@example.com" \
  --team "Platform Team" \
  --status "approved" \
  --comment "Looks good, all concerns addressed"
```

### 5. Modify Proposal (Create New Iteration)

```bash
# Modify proposal during review (creates new iteration)
sruja proposal modify PROP-001 \
  --description <description> \
  --change <change-description> \
  --triggered-by <reviewer-email>

# Modify technology choice (ADR) - tracks decision evolution
sruja proposal modify PROP-001 \
  --adr "ADR-003" \
  --choice "GraphQL API" \
  --rationale "Better for analytics queries" \
  --changed-by <reviewer-email> \
  --reason <reason-for-change>

# Example
sruja proposal modify PROP-001 \
  --adr "ADR-003" \
  --choice "GraphQL API" \
  --rationale "Better for analytics queries, suggested by Platform Team" \
  --changed-by "alice@example.com" \
  --reason "Platform Team prefers GraphQL for consistency"
```

**Modifications create new iterations**:
- Each modification creates a new iteration version
- Tracks what changed and why
- Tracks who suggested the change
- Tracks decision evolution (how choices changed)
- Modifications can include:
  - Changing relations
  - Modifying technology choices (ADRs)
  - Adjusting proposed elements
  - Updating scope

### 6. Check Conflicts

```bash
# Check for conflicts between proposals
sruja proposal conflicts PROP-001

# Check all proposals for conflicts
sruja proposal conflicts --all

# Check conflicts with specific proposal
sruja proposal conflicts PROP-001 --with PROP-002
```

### 7. Approve Proposal

```bash
# Approve proposal (all reviews must be approved)
sruja proposal approve PROP-001

# Force approve (override)
sruja proposal approve PROP-001 --force
```

### 8. Commit Proposal

```bash
# Commit approved proposal as change (change is linked to proposal)
sruja proposal commit PROP-001 --change-name "001-add-analytics"

# This creates: changes/001-add-analytics.sruja
# The change file references PROP-001 and includes iteration history
```

### 9. Reject/Close Proposal

```bash
# Reject proposal
sruja proposal reject PROP-001 --reason <reason>

# Close proposal (withdraw)
sruja proposal close PROP-001 --reason <reason>
```

## File Structure

```
architecture/
  ├── proposals/
  │   ├── PROP-001-add-analytics.sruja
  │   ├── PROP-001-add-analytics.md          # Optional: Proposal document
  │   ├── PROP-002-refactor-payment.sruja
  │   └── PROP-003-add-inventory.sruja
  ├── changes/
  │   ├── 001-add-analytics.sruja            # Committed from PROP-001
  │   └── 002-refactor-payment.sruja         # Committed from PROP-002
  ├── current.sruja
  └── .sruja-proposals.json                  # Proposal metadata
```

## Proposal Metadata File

```json
{
  "proposals": [
    {
      "id": "PROP-001",
      "title": "Add Analytics Dashboard",
      "team": "Analytics Team",
      "status": "in-review",
      "created": "2025-01-10T10:00:00Z",
      "updated": "2025-01-15T14:30:00Z",
      "requirement": "REQ-123",
      "userStory": "US-456",
      "file": "proposals/PROP-001-add-analytics.sruja",
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
          "message": "What protocol are you planning to use? REST or GraphQL?",
          "status": "answered",
          "responses": [...]
        }
      ],
      "actionItems": [
        {
          "id": "AI-001",
          "description": "Add authentication layer with OAuth2",
          "assignedTo": "diana@example.com",
          "team": "Analytics Team",
          "status": "pending",
          "linkedToDiscussion": "DISC-002"
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
      ]
    }
  ]
}
```

## Implementation

### Create Proposal

```go
func CreateProposal(title string, team string, requirement string) error {
    // Generate proposal ID (next in sequence)
    proposalNum := getNextProposalNumber()
    proposalID := fmt.Sprintf("PROP-%03d", proposalNum)
    
    // Create proposal file with template
    template := generateProposalTemplate(proposalID, title, team, requirement)
    
    // Write to proposals/ directory
    return writeProposalFile(proposalID, template)
}
```

### Check Conflicts

```go
func CheckConflicts(proposalID string) ([]Conflict, error) {
    currentProposal := loadProposal(proposalID)
    allProposals := loadAllProposals()
    
    conflicts := []Conflict{}
    for _, otherProposal := range allProposals {
        if otherProposal.ID == proposalID {
            continue
        }
        
        // Check if proposals modify same elements
        if hasElementOverlap(currentProposal, otherProposal) {
            conflicts = append(conflicts, Conflict{
                Proposal1: currentProposal.ID,
                Proposal2: otherProposal.ID,
                Elements: findOverlappingElements(currentProposal, otherProposal),
            })
        }
    }
    
    return conflicts, nil
}
```

### Commit Proposal

```go
func CommitProposal(proposalID string, changeName string) error {
    proposal := loadProposal(proposalID)
    
    // Validate proposal is approved
    if proposal.Status != "approved" {
        return fmt.Errorf("proposal %s is not approved", proposalID)
    }
    
    // Convert proposal to change (includes proposal link and iteration history)
    change := convertProposalToChange(proposal)
    change.Proposal = proposalID
    change.IterationHistory = proposal.Iterations
    change.DecisionEvolution = proposal.DecisionEvolution
    
    // Write change file
    return writeChangeFile(changeName, change)
}
```

## Workflow Integration

Proposals integrate with:
- **Change system**: Approved proposals become changes (changes are linked to proposals)
- **Change visualization**: Proposals shown in timeline with iteration history
- **Team ownership**: Proposals respect team ownership
- **Review process**: Multiple reviewers, comments, modifications tracked as iterations
- **Decision tracking**: Decision evolution captured through iterations

## Acceptance Criteria

* [ ] `proposal create` creates proposal file with template
* [ ] `proposal list` shows all proposals with filters
* [ ] `proposal view` shows proposal details
* [ ] `proposal diff` shows proposed changes
* [ ] `proposal comment` adds comments/questions/concerns to discussions
* [ ] `proposal respond` allows responding to discussions
* [ ] `proposal action-item` creates and manages action items
* [ ] `proposal review` adds formal review (after discussions resolved)
* [ ] `proposal modify` updates proposal during review (creates iterations)
* [ ] `proposal conflicts` detects conflicts between proposals
* [ ] `proposal approve` approves proposal (requires all reviews, tracks action items)
* [ ] `proposal commit` converts approved proposal to change (change linked to proposal)
* [ ] Proposal iterations are tracked
* [ ] Decision evolution is captured through iterations
* [ ] Changes reference their originating proposal
* [ ] Proposal metadata file is maintained
* [ ] Team ownership is validated
* [ ] Parallel proposals are supported
* [ ] Conflict detection works correctly
