# Sruja Real-World Usefulness Test

## Goal

Test whether Sruja AI skills can effectively generate useful architecture documentation from real-world codebases.

## Approach

1. **Pick 5 popular repositories** with different architectures
2. **Clone and analyze** each codebase using Sruja AI skills
3. **Generate architecture DSL** for each project
4. **Evaluate usefulness** - Does the generated architecture help understand the codebase?

## Test Repositories

The default set is defined in `setup_repos.sh` and documented in `test-repos/MANIFEST.md`. Run `./setup_repos.sh` to clone. Current default set:

### 1. **Express.js** (Node.js web framework)
- **Why**: Simple, well-documented, middleware architecture
- **GitHub**: https://github.com/expressjs/express
- **Expected**: Clear layered architecture with middleware pipeline

### 2. **FastAPI** (Python API framework)
- **Why**: Modern async API framework, clear structure
- **GitHub**: https://github.com/tiangolo/fastapi
- **Expected**: Starlette-based layers, OpenAPI integration

### 3. **Next.js** (React framework)
- **Why**: Full-stack React, SSR, and tooling
- **GitHub**: https://github.com/vercel/next.js
- **Expected**: App/router, build pipeline, server/client boundaries

### 4. **Prometheus** (Go monitoring system)
- **Why**: Distributed system, time-series storage, scraping
- **GitHub**: https://github.com/prometheus/prometheus
- **Expected**: Scraper, storage, query layer, federation

### 5. **Django** (Python web framework)
- **Why**: Large, layered framework with ORM and admin
- **GitHub**: https://github.com/django/django
- **Expected**: Apps, middleware, ORM, request/response cycle

## Test Process

### Step 1: Clone Repositories

```bash
# From evaluation/real-world-test: clone all default test repos
./setup_repos.sh

# Or clone manually (see setup_repos.sh for the current list)
mkdir -p test-repos
cd test-repos
git clone --depth 1 https://github.com/expressjs/express.git
git clone --depth 1 https://github.com/tiangolo/fastapi.git
git clone --depth 1 https://github.com/vercel/next.js.git
git clone --depth 1 https://github.com/prometheus/prometheus.git
git clone --depth 1 https://github.com/django/django.git
```

### Step 2: Generate Architecture with Sruja AI Skills

For each repository:

```bash
# Example for Express.js
cd express

# Use Sruja AI skill to analyze codebase and generate architecture
# (This assumes you have Sruja AI skills installed in your editor)
# In VS Code with Copilot Chat:
# "@sruja Analyze this codebase and generate architecture DSL"

# Or use the CLI if available:
# sruja generate architecture . > architecture.sruja
```

**What to ask the AI:**
1. "Analyze this codebase's structure and main components"
2. "Generate a Sruja architecture DSL showing:"
   - Main systems and containers
   - Key data flows
   - External dependencies
   - Technology choices

### Step 3: Save Generated Architecture

Save the generated DSL as `architecture.sruja` in each repo's directory:

```
test-repos/
├── express/
│   └── architecture.sruja
├── fastapi/
│   └── architecture.sruja
├── next.js/
│   └── architecture.sruja
├── prometheus/
│   └── architecture.sruja
└── django/
    └── architecture.sruja
```

### Step 4: Evaluate Usefulness

For each generated architecture, answer these questions:

#### **Completeness** (Does it capture the essentials?)
- [ ] Main components identified?
- [ ] Key relationships shown?
- [ ] Technologies mentioned?
- [ ] External dependencies included?

#### **Accuracy** (Is it correct?)
- [ ] Component names match codebase?
- [ ] Relationships reflect actual dependencies?
- [ ] No fabricated components?
- [ ] Technology choices accurate?

#### **Usefulness** (Does it help understanding?)
- [ ] Would a new developer understand the architecture faster?
- [ ] Are the main abstractions clear?
- [ ] Is the complexity manageable?
- [ ] Does it highlight key design decisions?

#### **Comparison to Existing Docs** (if available)
- [ ] Does it match existing architecture docs?
- [ ] Does it add new insights?
- [ ] Is it more/less clear than existing docs?

## Evaluation Method

### Option A: Manual Review (Recommended)

1. **Open the repository** in your editor
2. **Read the generated architecture.sruja**
3. **Explore the codebase** to verify accuracy
4. **Check existing documentation** (README, docs/, etc.)
5. **Fill out the evaluation checklist** above

Time: ~30 minutes per repository

### Option B: LLM Check

Use `sruja eval` for automated LLM evaluation (any provider):

```bash
export OPENAI_API_KEY="sk-..."   # or OPENROUTER, ANTHROPIC, GEMINI
./evaluate_architecture.sh express --llm
# Or: sruja eval test-repos/express
```

### Option C: Team Review

Share generated architectures with your team:
1. Show architecture DSL to team members unfamiliar with the codebase
2. Ask them to explain what they understand
3. Compare with team members who know the codebase
4. Collect feedback on gaps or inaccuracies

## Success Criteria

Sruja is **useful** if:

✅ **3/5 repositories** score **≥7/10** on average across all criteria
✅ **At least 1 high-complexity repository** (e.g. next.js, prometheus, django) scores **≥6/10**
✅ **No generated architectures** are completely wrong (≤3/10)
✅ **Generated architectures** provide value over README alone

## Results Template

For each repository, record:

```markdown
## Repository: [Name]

### Generated Architecture
- File: `test-repos/[name]/architecture.sruja`
- Lines of DSL: [count]
- Time to generate: [minutes]

### Evaluation Scores (1-10)

| Criterion      | Score | Notes |
|----------------|-------|-------|
| Completeness   |       |       |
| Accuracy       |       |       |
| Usefulness     |       |       |
| **Average**    |       |       |

### What Worked Well
- [List specific strengths]

### What Was Missing
- [List gaps or inaccuracies]

### Comparison to Existing Docs
- [How does it compare to README/docs?]

### Verdict
- [ ] Useful (≥7/10)
- [ ] Partially useful (5-6/10)
- [ ] Not useful (<5/10)

### Key Insights
[What did you learn about Sruja's capabilities?]
```

## Quick Start

**Fast path (zero config, ~2 min):**
```bash
./run_demo.sh
```

**Full evaluation:**
```bash
# Clone test repositories (shell - no Python required)
./setup_repos.sh

# Generate architecture for each repo using Sruja AI skills, then evaluate:
./evaluate_architecture.sh express
```

**Optional LLM eval:** Copy `.env.example` to `.env`, add any LLM API key, then `./run_demo.sh --llm` or `./evaluate_architecture.sh express --llm`.

See [QUICKSTART.md](QUICKSTART.md) for the full guide.

## What This Tests

This evaluates **Sruja's core value proposition**:

> "Can AI understand a codebase and create useful architecture documentation?"

**What we're testing:**
- ✅ Can Sruja identify main components?
- ✅ Can it understand relationships?
- ✅ Can it capture architecture patterns?
- ✅ Is the output actually helpful?

**What we're NOT testing:**
- ❌ Syntax validation (that's unit tested)
- ❌ Export features (that's integration tested)
- ❌ Performance (not relevant for usefulness)

## Interpreting Results

### If scores are high (≥7/10):
✅ **Sruja is useful** - It effectively documents architectures
- Share success stories
- Use in onboarding docs
- Promote as key feature

### If scores are mixed (5-6/10):
⚠️ **Sruja shows promise but needs improvement**
- Identify common failure patterns
- Improve AI prompts
- Focus on specific architecture types

### If scores are low (<5/10):
❌ **Sruja needs significant work**
- Analyze what went wrong
- Consider different approach
- May not be ready for real-world use

## Next Steps After Evaluation

1. **Aggregate results** across all 5 repositories
2. **Identify patterns** in what works/doesn't work
3. **Create improvements** based on findings
4. **Re-test** with improved Sruja skills
5. **Document learnings** for future development

## FAQ

**Q: Why these 5 repositories?**
A: The default set (express, fastapi, next.js, prometheus, django) gives diverse architectures, languages, and complexity. You can change the list in `setup_repos.sh` and regenerate the manifest.

**Q: Can I use different repositories?**
A: Yes! Pick repositories relevant to your use case. Just ensure they have:
- Active development
- Reasonable size (not too small/large)
- Some existing documentation to compare

**Q: What if I don't have Sruja AI skills installed?**
A: Install with: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`

**Q: How long does this take?**
A: 
- Setup: 10 minutes
- Generation: 30-60 minutes (varies by repo size)
- Evaluation: 2-3 hours (manual review)
- Total: ~4 hours

**Q: Can I automate this?**
A: The generation can be automated if you have API access to an LLM. Evaluation is best done manually for accurate assessment.

## Contributing Results

Found something interesting? Share your results:

1. **Create a gist** with your evaluations
2. **Open a discussion** on GitHub
3. **Share insights** with the community

Your findings help improve Sruja for everyone!

---

**Questions?** Check the main README or open an issue.