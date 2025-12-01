# Simplified Change Workflow: ADRs + Studio + External Systems

## Core Principle

**Keep the language simple. Use external systems for collaboration.**

- ✅ **ADRs** for decision tracking (already in language)
- ✅ **Studio** for creating changes visually
- ✅ **GitHub/Cloud Studio** for discussions and reviews
- ❌ **No proposal workflow** in the language itself

## Workflow

### 1. Create Change (Studio or CLI)

**Option A: Visual Studio**
- Open Studio (`sruja studio`)
- Create/modify architecture visually
- Save as change file

**Option B: CLI**
- Create change file directly
- Or use `sruja change create` for simple changes

### 2. Track Decision (ADR)

Use ADR to document the decision:

```sruja
// decisions/adr-001-analytics-api.sruja
adr "ADR-001" "Use REST API for analytics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  status "decided"
  context "Need to provide analytics data to external systems"
  decision "Use REST API with OAuth2 authentication"
  consequences [
    "Standard protocol - easy integration"
    "Requires authentication layer"
  ]
}
```

### 3. Create Change

**Simple change (CLI)**:
```bash
sruja change create add-analytics --requirement "REQ-123"
```

**Or create change file directly**:
```sruja
// changes/001-add-analytics.sruja
change "001-add-analytics" {
  version "v1.1.0"
  requirement "REQ-123"
  adr "ADR-001"
  
  metadata {
    owner: "alice@example.com"
    stakeholders: ["bob@example.com", "charlie@example.com", "Platform Team"]
  }
  
  add {
    system ShopSystem {
      container AnalyticsAPI "Analytics API" {
        component MetricsCollector {}
      }
    }
    relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
  }
}
```

### 4. Review & Discussion (External Systems)

**Option A: GitHub PR**
- Create PR with change file
- Use GitHub comments for discussions
- Use GitHub reviews for approvals
- Link ADR in PR description

**Option B: Cloud Studio** (future)
- Create proposal in cloud studio
- Use cloud studio discussions
- Use cloud studio reviews
- Auto-generate PR on approval

### 5. Commit Change

Once approved (via GitHub PR or cloud studio):
```bash
# Apply change
sruja change apply

# Then commit to Git (external)
git add changes/001-add-analytics.sruja current.sruja
git commit -m "Add analytics change"
```

## What We Remove

### ❌ Proposal Workflow (Task 1.6)

**Removed**:
- `sruja proposal create`
- `sruja proposal comment`
- `sruja proposal respond`
- `sruja proposal action-item`
- `sruja proposal review`
- `sruja proposal modify`
- `sruja proposal conflicts`
- `sruja proposal approve`
- `sruja proposal commit`

**Why**:
- Too complex for language core
- Duplicates ADR functionality (decisions)
- Comments don't belong in files
- External systems (GitHub, cloud studio) handle this better

### ✅ What We Keep

1. **Change Commands** (Task 1.5)
   - `sruja change create`
   - `sruja change apply`
   - `sruja snapshot create`
   - `sruja diff`

2. **ADRs** (already in language)
   - Track decisions
   - Link to elements via tags
   - Status tracking (pending/decided/rejected)

3. **Studio** (Task 4.6 simplified)
   - Create changes visually
   - Export to change files
   - Visual diff

4. **External Systems**
   - GitHub PRs for reviews
   - Cloud Studio for collaboration (future)
   - Comments/reviews in external systems

## Simplified Studio Integration

### Studio Creates Changes

Studio can:
- Create changes visually (drag-and-drop)
- Save to change file
- Link to ADR
- Link to requirement/user story

**No need for**:
- Proposal metadata in Studio
- Discussion threads in Studio
- Action items in Studio
- Review workflow in Studio

**Use external systems for**:
- Discussions (GitHub comments, cloud studio)
- Reviews (GitHub reviews, cloud studio)
- Approvals (GitHub approvals, cloud studio)

## ADR as Decision Tracker

ADRs handle decision tracking:

```sruja
// Pending decision
adr "ADR-002" "Choose database for analytics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  status "pending"
  context "Need to store large volumes of analytics data"
  options [
    "PostgreSQL - familiar, good performance"
    "ClickHouse - optimized for analytics"
  ]
}

// Decided (after discussion in GitHub/cloud studio)
adr "ADR-002" "Choose database for analytics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  status "decided"
  decision "PostgreSQL - team familiarity outweighs performance benefits"
  context "Need to store large volumes of analytics data"
  consequences [
    "Easier onboarding for team"
    "May need optimization later for scale"
  ]
}
```

## Benefits

✅ **Simpler language**: No proposal workflow complexity  
✅ **Better separation**: Decisions in ADRs, discussions external  
✅ **Familiar tools**: Use GitHub PRs (everyone knows them)  
✅ **Flexible**: Can use GitHub, cloud studio, or other systems  
✅ **Maintainable**: Less code, less complexity  

## Migration Path

1. **Remove Task 1.6** (Proposal Commands)
2. **Simplify Task 4.6** (Studio Proposals → Studio Changes)
3. **Enhance ADR support** (better status tracking, linking)
4. **Document workflow** (ADRs + GitHub PRs + Studio)

## Updated Task List

### Go Tasks
- ✅ Task 1.1: JSON Exporter
- ✅ Task 1.2: JSON to AST Converter
- ✅ Task 1.3: CLI Commands
- ✅ Task 1.4: Modularization
- ✅ Task 1.5: Change Commands (keep)
- ❌ Task 1.6: Proposal Commands (remove)

### TypeScript Tasks
- ✅ Task 3.1-3.8: Viewer Library
- ✅ Task 4.1-4.5: Studio Core
- 🔄 Task 4.6: Studio Changes (simplified - no proposals)

## Example Workflow

### Developer Workflow

1. **Create change in Studio**:
   ```bash
   sruja studio
   # Visually add AnalyticsAPI container
   # Save as change file
   ```

2. **Create ADR for decision**:
   ```sruja
   adr "ADR-001" "Use REST API" {
     tags [container "ShopSystem.AnalyticsAPI"]
     status "decided"
   }
   ```

3. **Create PR in GitHub**:
   - PR includes change file + ADR
   - Discussion happens in GitHub comments
   - Reviews happen in GitHub reviews

4. **After approval, commit**:
   ```bash
   sruja change apply
   ```

### Team Workflow (Future: Cloud Studio)

1. **Create change in Cloud Studio**
2. **Create ADR in Cloud Studio**
3. **Discussion in Cloud Studio** (threads, mentions)
4. **Review in Cloud Studio** (approvals)
5. **Auto-generate PR** on approval
6. **Merge PR** → change committed

## Conclusion

**Simpler is better**. Use:
- **ADRs** for decisions (already in language)
- **Studio** for creating changes (visual editor)
- **External systems** for collaboration (GitHub, cloud studio)

This keeps the language focused on architecture, not project management.

