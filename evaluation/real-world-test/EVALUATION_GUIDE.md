# Sruja Real-World Usefulness Test

> **Preferred:** Use `./setup_repos.sh` and `./evaluate_architecture.sh` (shell + sruja CLI, no Python).

## Goal

Test whether Sruja's AI skills can effectively generate useful architecture documentation from real-world codebases.

## Test Repositories

### Default set (this directory)

The scripts in this directory use a **default list** defined in `setup_repos.sh` and documented in `test-repos/MANIFEST.md`. To use it:

```bash
# From evaluation/real-world-test
./setup_repos.sh
```

That clones: **express**, **fastapi**, **next.js**, **prometheus**, **django** into `test-repos/`. To evaluate: `./evaluate_architecture.sh express` (or any repo name under `test-repos/`). See [QUICKSTART.md](QUICKSTART.md) for the short path and [README.md](README.md) for the full process.

### Alternative / custom repositories

You can use other repos by cloning them (or adding entries to `setup_repos.sh`). Below is an **alternative** list you can use for manual setup or as inspiration:

1. **React** (Frontend Library) — https://github.com/facebook/react  
   - Component architecture, reconciliation, fiber

2. **Express.js** (Backend Framework) — https://github.com/expressjs/express  
   - Request/response flow, middleware chain

3. **Next.js** (Full-Stack Framework) — https://github.com/vercel/next.js  
   - Pages, API routes, build pipeline

4. **Kubernetes** (Container Orchestration) — https://github.com/kubernetes/kubernetes  
   - Control plane, scheduler, etcd integration

5. **Stripe Node SDK** (API Client Library) — https://github.com/stripe/stripe-node  
   - HTTP client, resource management, auth flow

**Other options:** Vue.js, FastAPI, Django, MongoDB, PostgreSQL

---

## Testing Process

### Step 1: Setup (default)

```bash
# From evaluation/real-world-test: clone default test repos into test-repos/
./setup_repos.sh
```

To use a different layout or repos, create your own directory and clone; then run `./evaluate_architecture.sh /path/to/repo` (repo must contain `architecture.sruja`).

### Step 2: Generate Architecture with Sruja AI Skills

For each repository:

#### Option A: Using Cursor/Copilot with Sruja Skill

```bash
# Open repo in Cursor/VS Code with Sruja skill installed
cd repos/react

# In AI chat, use this prompt:
"Analyze this codebase and generate a Sruja architecture DSL file (architecture.sruja) 
that captures:
- Main components/modules
- Key relationships and data flows
- External dependencies
- Technology choices

Follow Sruja DSL conventions from .cursorrules"
```

#### Option B: Using Sruja CLI (if available)

```bash
cd repos/react
sruja generate architecture --output architecture.sruja
```

#### Option C: Manual with AI Assistance

```bash
# Ask AI to analyze key files
# In Cursor chat:
"Look at package.json, src/, and docs/ to understand the architecture.
Generate a Sruja DSL file that represents this codebase's architecture."

# Save output to architecture.sruja
```

### Step 3: Validate Generated Architecture

```bash
# Validate syntax
sruja lint architecture.sruja

# Export to visual format
sruja export mermaid architecture.sruja > architecture.md
```

---

## Evaluation Checklist

For each generated `architecture.sruja`, evaluate:

### 1. Accuracy (Does it match the code?)

- [ ] Main components/modules are correctly identified
- [ ] Component purposes are accurately described
- [ ] Technology choices are correct (languages, frameworks)
- [ ] External dependencies are captured
- [ ] No hallucinated components

**Score:** ___/5

### 2. Completeness (Is anything important missing?)

- [ ] All major subsystems are documented
- [ ] Key data flows are captured
- [ ] External integrations are included
- [ ] Important architectural patterns visible
- [ ] Critical files/modules represented

**Score:** ___/5

### 3. Usefulness (Would this help a new developer?)

- [ ] I can understand the high-level architecture from this DSL
- [ ] Relationships between components are clear
- [ ] Would help onboard new team members
- [ ] Could guide architectural decisions
- [ ] Reveals system complexity appropriately

**Score:** ___/5

### 4. Quality (Is it well-structured?)

- [ ] Follows Sruja DSL conventions
- [ ] Descriptions are clear and concise
- [ ] Hierarchy makes sense
- [ ] No validation errors
- [ ] Consistent naming and style

**Score:** ___/5

---

## Evaluation Template

### Repository: [Name]

**Generated File:** `repos/[name]/architecture.sruja`

**Time to Generate:** [X minutes/hours]

#### Accuracy Assessment

- Components identified correctly? [Yes/Partially/No]
- Missing major components: [List them]
- Incorrect components: [List them]
- Hallucinated components: [List them]

**Notes:**
```
[Your observations]
```

#### Completeness Assessment

- Coverage: [All major parts / Most parts / Some parts / Minimal]
- Missing subsystems: [List them]
- Missing important flows: [List them]

**Notes:**
```
[Your observations]
```

#### Usefulness Assessment

- Would this help understanding? [Yes/Maybe/No]
- Clarity of relationships: [Clear/Somewhat/Unclear]
- Onboarding value: [High/Medium/Low]

**Notes:**
```
[Your observations]
```

#### Comparison with Existing Docs

Does repo have existing architecture docs? [Yes/No]

If yes:
- How does Sruja DSL compare? [Better/Same/Worse]
- What's missing in Sruja version?
- What's better in Sruja version?

**Notes:**
```
[Your observations]
```

#### Overall Scores

| Criteria | Score (1-5) | Notes |
|----------|-------------|-------|
| Accuracy | ___/5 | |
| Completeness | ___/5 | |
| Usefulness | ___/5 | |
| Quality | ___/5 | |
| **Total** | **___/20** | |

**Verdict:** [Exceptional/Good/Acceptable/Poor]

---

## Summary Template

After evaluating all 5 repos, fill this summary:

### Overall Sruja Usefulness Score

| Repository | Accuracy | Completeness | Usefulness | Quality | Total | Verdict |
|------------|----------|--------------|------------|---------|-------|---------|
| React | __/5 | __/5 | __/5 | __/5 | __/20 | |
| Express | __/5 | __/5 | __/5 | __/5 | __/20 | |
| Next.js | __/5 | __/5 | __/5 | __/5 | __/20 | |
| Kubernetes | __/5 | __/5 | __/5 | __/5 | __/20 | |
| Stripe | __/5 | __/5 | __/5 | __/5 | __/20 | |
| **Average** | **__/5** | **__/5** | **__/5** | **__/5** | **__/20** | |

### Key Findings

**What Worked Well:**
1.
2.
3.

**What Needs Improvement:**
1.
2.
3.

**Surprising Discoveries:**
1.
2.
3.

### Recommendations for Sruja

Based on this evaluation:

1. **Feature improvements:**
   -

2. **Documentation needs:**
   -

3. **AI skill enhancements:**
   -

### Conclusion

**Is Sruja useful for real-world architecture documentation?**

[ ] Yes, highly effective
[ ] Yes, with some improvements
[ ] Maybe, needs more work
[ ] No, not yet ready

**Primary reason:**
```
[Your assessment]
```

**Would you use Sruja for your own projects?**

[ ] Definitely
[ ] Probably
[ ] Maybe
[ ] Unlikely

**Why or why not?**
```
[Your reasoning]
```

---

## Automation (Optional)

### Simple manual evaluation

If you want a consistent checklist, use this prompt (with your editor's AI or manually):

```
I have a Sruja architecture DSL file for [repository name].

Repository: [link or description]
Generated DSL:
[architecture.sruja content]

Existing docs (if any):
[existing docs or "None"]

Evaluate:
1. Accuracy (1-5): Does it match the codebase?
2. Completeness (1-5): Are major components included?
3. Usefulness (1-5): Would this help developers?
4. Quality (1-5): Is it well-structured?

Provide specific feedback on what's good and what's missing.
```

### Script for Quick Comparison

```bash
#!/bin/bash
# compare_docs.sh

REPO=$1

echo "=== $REPO Evaluation ==="
echo ""

echo "Generated architecture:"
wc -l repos/$REPO/architecture.sruja
echo ""

echo "Existing docs:"
find repos/$REPO -name "README*" -o -name "ARCHITECTURE*" -o -name "CONTRIBUTING*" | head -5
echo ""

echo "Generated Mermaid diagram:"
sruja export mermaid repos/$REPO/architecture.sruja | wc -l
echo ""

echo "Validation:"
sruja lint repos/$REPO/architecture.sruja
```

---

## Next Steps

After evaluation:

1. **Document findings** in this file
2. **Share results** with Sruja team/community
3. **Provide feedback** on GitHub issues
4. **Contribute improvements** if possible

---

## Notes Section

Use this space for general observations:

```
[Your notes here]
```

---

**Remember:** The goal is to answer "Is Sruja actually useful for understanding real-world codebases?" Be honest and specific in your evaluation.