# Multi-Repo Federation: Setup Guide

Complete step-by-step guide to test Sruja's multi-repo federation with real repositories.

## What This Guide Does

Walks you through setting up federation across multiple repositories, from individual repo setup to system-wide composition.

**Time to complete:** 15-20 minutes

## Overview

Federation lets you:
- Keep separate `repo.sruja` files in each repository
- Export each repo's architecture as a `repo.bundle.json`
- Combine all bundles into a single `system.index.json` with canonical IDs
- Detect conflicts (same logical element in multiple repos)

**Use when:** You have architecture spread across multiple repos and want a system-wide view.

## Prerequisites

**GitHub org layout:** The public `sruja-ai` product and documentation deploy targets are summarized in [RELATED_REPOSITORIES.md](RELATED_REPOSITORIES.md) (main repo, staging site, production site). Federation can still include additional private or external repositories in your own `system.index.json`.

1. **Sruja CLI installed**
   ```bash
   curl -fsSL https://sruja.ai/install.sh | bash
   sruja --version  # Verify: should show version
   ```

2. **Two or more repositories to test with**
   - Use existing repos OR create test repos (instructions below)
   - Can be any language (TypeScript, Python, Go, Rust, Java, etc.)

3. **Git initialized in each repo**
   ```bash
   cd /path/to/each/repo
   git init  # If not already a git repo
   ```

---

## Part 1: Create Test Repositories (Optional)

If you don't have existing repos, create three simple test repos:

```bash
# Create a workspace directory
mkdir ~/sruja-federation-test
cd ~/sruja-federation-test

# Repo 1: API Service
mkdir api-service
cd api-service
npm init -y
npm install express

cat > index.js << 'EOF'
const express = require('express');
const app = express();
const { Pool } = require('pg');

const db = new Pool({ host: 'postgres' });

app.get('/users', async (req, res) => {
  const result = await db.query('SELECT * FROM users');
  res.json(result.rows);
});

app.listen(3000);
EOF

git init
git add .
git commit -m "Initial API service"
cd ..

# Repo 2: Payment Service
mkdir payment-service
cd payment-service
npm init -y
npm install express

cat > index.js << 'EOF'
const express = require('express');
const app = express();
const stripe = require('stripe')('sk_test_...');

app.post('/payment', async (req, res) => {
  const payment = await stripe.charges.create(req.body);
  res.json(payment);
});

app.listen(3001);
EOF

git init
git add .
git commit -m "Initial payment service"
cd ..

# Repo 3: Frontend
mkdir frontend
cd frontend
npm init -y
npm install vite

cat > main.js << 'EOF'
import axios from 'axios';

async function getUsers() {
  const res = await axios.get('http://api:3000/users');
  return res.data;
}

export { getUsers };
EOF

git init
git add .
git commit -m "Initial frontend"
cd ..
```

---

## Part 2: Generate Architecture in Each Repo

For **each repository**, use the AI skill to generate `repo.sruja`:

### Step 2.1: Install the AI skill (once)

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

### Step 2.2: Generate repo.sruja in each repo

In your AI editor (Cursor, Copilot, Claude, etc.), run this prompt in **each repo**:

```
Use sruja-architecture. Generate repo.sruja for this repository.
Run sruja lint and fix any errors.
```

**Expected output:** A `repo.sruja` file in each repo that passes lint.

Example what you should see:

```bash
cd ~/sruja-federation-test/api-service
ls repo.sruja  # Should exist
sruja lint repo.sruja  # Should say "No errors found"
```

### Step 2.3: Commit repo.sruja in each repo

```bash
cd ~/sruja-federation-test/api-service
git add repo.sruja
git commit -m "Add repo.sruja architecture"

cd ../payment-service
git add repo.sruja
git commit -m "Add repo.sruja architecture"

cd ../frontend
git add repo.sruja
git commit -m "Add repo.sruja architecture"
```

---

## Part 3: Publish Each Repo as a Bundle

In **each repository**, run the publish command to create a bundle:

```bash
cd ~/sruja-federation-test/api-service
sruja publish -r . -o repo.bundle.json

cd ../payment-service
sruja publish -r . -o repo.bundle.json

cd ../frontend
sruja publish -r . -o repo.bundle.json
```

**What this does:**
- Scans the repo codebase
- Reads `repo.sruja` baseline
- Creates `repo.bundle.json` with:
  - `repo_id` (inferred from directory or git remote)
  - `baseline_dsl` (content of repo.sruja)
  - `context` (from `.sruja/context.json` if present)
  - `truth_status` ("reviewed", "drifted", or "unknown")
  - `git_commit` (short SHA)

**Expected output:**
```
Wrote repo.bundle.json (repo_id=api-service, truth_status=reviewed)
Wrote repo.bundle.json (repo_id=payment-service, truth_status=reviewed)
Wrote repo.bundle.json (repo_id=frontend, truth_status=reviewed)
```

---

## Part 4: Compose System Index

Now combine all bundles into a single system index:

```bash
cd ~/sruja-federation-test
mkdir bundles

# Copy all bundles to a central location (rename to avoid collisions)
cp api-service/repo.bundle.json bundles/api-service.repo.bundle.json
cp payment-service/repo.bundle.json bundles/payment-service.repo.bundle.json
cp frontend/repo.bundle.json bundles/frontend.repo.bundle.json

# Compose into a single system index
sruja compose -i bundles -o system.index.json
```

**What this does:**
- Reads all bundle files matching `repo.bundle.json` or `*.repo.bundle.json` pattern
- Creates `system.index.json` with:
  - `repos`: List of all composed repos with metadata
  - `nodes`: All nodes with canonical IDs (`repo_id::local_id`)
  - `edges`: All relationships with source/target as canonical IDs
  - `conflicts`: Any duplicate elements across repos

**Expected output:**
```
Wrote system.index.json (3 repos, 15 nodes, 20 edges, 0 conflict(s))
```

---

## Part 5: Verify and Inspect Results

### 5.1: Check the system index structure

```bash
cat system.index.json | jq 'keys'
```

Should show: `["schema_version", "repos", "nodes", "edges", "conflicts"]`

### 5.2: List all repos

```bash
cat system.index.json | jq '.repos[] | {repo_id, repo_path, truth_status, git_commit}'
```

Example output:
```json
{
  "repo_id": "api-service",
  "repo_path": "/Users/.../api-service",
  "truth_status": "reviewed",
  "git_commit": "abc123"
}
{
  "repo_id": "payment-service",
  "repo_path": "/Users/.../payment-service",
  "truth_status": "reviewed",
  "git_commit": "def456"
}
{
  "repo_id": "frontend",
  "repo_path": "/Users/.../frontend",
  "truth_status": "reviewed",
  "git_commit": "ghi789"
}
```

### 5.3: View canonical nodes

```bash
cat system.index.json | jq '.nodes[] | {canonical_id, kind, label, repo_id}'
```

Example output:
```json
{
  "canonical_id": "api-service::API",
  "kind": "container",
  "label": "API",
  "repo_id": "api-service"
}
{
  "canonical_id": "payment-service::Payment",
  "kind": "container",
  "label": "Payment",
  "repo_id": "payment-service"
}
```

### 5.4: Check for conflicts

```bash
cat system.index.json | jq '.conflicts'
```

If you see conflicts, it means the same element (kind + label) exists in multiple repos:
```json
{
  "key": "container::Database",
  "repos": ["api-service", "payment-service"],
  "message": "Same kind+label in multiple repos; resolve canonical ownership or rename."
}
```

---

## Part 6: Use in CI/CD (Optional)

### 6.1: Per-repo CI (validate each repo individually)

Create `.github/workflows/sruja-validate.yml` in **each repo**:

```yaml
name: Validate Sruja Architecture

on:
  push:
    branches: [main, master]
  pull_request:

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Sruja
        run: |
          curl -fsSL https://sruja.ai/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Lint architecture
        run: |
          if [ -f "repo.sruja" ]; then
            sruja lint repo.sruja
          else
            echo "No repo.sruja found; skipping lint"
          fi

      - name: Publish bundle (for federation)
        run: |
          if [ -f "repo.sruja" ]; then
            sruja publish -r . -o repo.bundle.json
          fi

      - name: Upload bundle
        uses: actions/upload-artifact@v4
        with:
          name: repo-bundle
          path: repo.bundle.json
```

### 6.2: Central federation job (compose all repos)

Create a dedicated repo (e.g., `architecture-federation`) with this workflow:

```yaml
name: Compose System Architecture

on:
  workflow_dispatch:
  schedule:
    - cron: '0 * * * *'  # Every hour

jobs:
  compose:
    runs-on: ubuntu-latest
    steps:
      - name: Install Sruja
        run: |
          curl -fsSL https://sruja.ai/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Download all bundles
        uses: actions/download-artifact@v4
        with:
          path: bundles

      - name: Compose system index
        run: sruja compose -i bundles -o system.index.json

      - name: Upload system index
        uses: actions/upload-artifact@v4
        with:
          name: system-index
          path: system.index.json

      - name: Report conflicts
        run: |
          conflicts=$(cat system.index.json | jq '.conflicts | length')
          echo "Conflicts: $conflicts"
          if [ "$conflicts" -gt 0 ]; then
            echo "::warning::Found $conflicts conflicts in system architecture"
            cat system.index.json | jq '.conflicts'
          fi
```

---

## Part 7: Troubleshooting

### Issue: "No bundle files found"

**Cause:** The `sruja compose` input path doesn't contain any `repo.bundle.json` files.

**Fix:**
```bash
# Verify bundles exist
ls bundles/*.repo.bundle.json

# Check exact filenames (must be repo.bundle.json or *.repo.bundle.json)
ls -la bundles/
```

### Issue: Conflicts in system.index.json

**Cause:** Same element (kind + label) exists in multiple repos.

**Example:** Both `api-service` and `payment-service` define a container called "Database".

**Resolution options:**
1. **Rename in one repo** (e.g., `PaymentDatabase` vs `UserDatabase`)
2. **Mark as external system** (one repo references, other defines)
3. **Document ownership** (both repos acknowledge it's shared)

### Issue: "schema_version mismatch"

**Cause:** Bundle files from different Sruja versions.

**Fix:** Update Sruja CLI in all repos to the same version:
```bash
curl -fsSL https://sruja.ai/install.sh | bash
sruja --version
```

### Issue: "git_commit is null"

**Cause:** Repo is not a git repository or no commits exist.

**Fix:**
```bash
cd /path/to/repo
git init
git add .
git commit -m "Initial commit"
```

---

## Part 8: What You've Built

After completing this guide, you have:

1. ✅ **Per-repo architecture:** Each repo has its own `repo.sruja`
2. ✅ **Validation:** Each repo validates its architecture in CI
3. ✅ **Bundles:** Each repo publishes a `repo.bundle.json` artifact
4. ✅ **System index:** Composed `system.index.json` with canonical IDs
5. ✅ **Conflict detection:** Duplicate elements are flagged, not silently merged

**Next steps:**
- Export diagrams from the system index
- Use the system index in AI code generation (skill can load impacted slices)
- Set up scheduled federation jobs in CI/CD
- Document ownership for cross-repo elements

---

## Artifact Reference

### repo.bundle.json Schema

```json
{
  "schema_version": 1,
  "repo_id": "api-service",
  "repo_path": "/path/to/repo",
  "git_commit": "abc123",
  "baseline_path": "repo.sruja",
  "baseline_dsl": "System = system \"API\" { ... }",
  "context": {
    "updated_at": "2024-03-15T10:30:00Z",
    "truth_status": "reviewed",
    "git_commit": "abc123",
    "baseline_path": "repo.sruja"
  },
  "truth_status": "reviewed",
  "intent_refs": [],
  "contracts": null,
  "owners": null
}
```

### system.index.json Schema

```json
{
  "schema_version": 1,
  "repos": [
    {
      "repo_id": "api-service",
      "repo_path": "/path/to/repo",
      "truth_status": "reviewed",
      "git_commit": "abc123"
    }
  ],
  "nodes": [
    {
      "canonical_id": "api-service::API",
      "kind": "container",
      "label": "API",
      "technology": "Express",
      "repo_id": "api-service",
      "local_id": "API"
    }
  ],
  "edges": [
    {
      "source": "api-service::API",
      "target": "api-service::Database",
      "kind": "ReadsFrom",
      "label": "SQL",
      "repo_id": "api-service"
    }
  ],
  "conflicts": []
}
```

---

## Related Documentation

- [Federation Technical Spec](FEDERATION.md) - Full artifact schemas and composition rules
- [Getting Started with Skills](GETTING_STARTED_SKILL.md#multi-repo) - Multi-repo workflow overview
- [AI Skill Documentation](../skills/sruja-architecture/SKILL.md) - AI retrieval order for federation
- [CLI Commands](../book/src/reference/cli.md) - Complete command reference
