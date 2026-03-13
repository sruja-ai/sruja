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
# From evaluation/real-world-test

# Frameworks/libraries (quick, for demos)
./setup_repos.sh

# Realistic applications (product-like: gitea, saleor, documenso, cal.com)
./setup_repos.sh --apps

# Or clone manually (see setup_repos.sh for the current list)
mkdir -p test-repos
cd test-repos
git clone --depth 1 https://github.com/expressjs/express.git
# ...
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

**Interactive/selective capture:** To extract or focus on a specific area (e.g. one subpath, concise summary only), use the **sruja-architecture-agent** skill as described in `skills/sruja-architecture-agent/SKILL.md` (suggested areas, pick scope, concise output). No separate script is used for this flow.

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

### Option B: Automated validation

Run `./evaluate_architecture.sh express` for validation, stats, and checklist. To compare a generated file to the golden reference: `./compare_architecture.sh test-repos/express/architecture.sruja run_results/generated_express.sruja`. For a batch report across repos: `./run_architecture_comparison_report.sh`. To build a **diff-and-refine** prompt (context + drift) for the AI: `./run_diff_refine_prompt.sh . architecture.sruja`. See [EVALUATION_METHODOLOGY.md](EVALUATION_METHODOLOGY.md) for metrics and how to improve the skill from results. Sruja CLI does not use LLM; for AI-assisted review, use the Sruja skill in your editor.

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

# Option A: Run demo with baseline (copies example architecture into express), then evaluate
./run_demo.sh --baseline
./evaluate_architecture.sh express

# Option B: One script to test and observe (demo + evaluate + quickstart fastapi), writes observations to run_results/
./run_test_and_observe.sh          # with clone + prepare_skill
./run_test_and_observe.sh --no-clone   # use existing test-repos only

# Option C: Generate architecture via AI (Cursor/agent or editor), then evaluate
./evaluate_architecture.sh express
```

**Test the Sruja skill on real projects:** Run `./prepare_skill_in_real_projects.sh`, then open a repo (e.g. `test-repos/express`) in Cursor or VS Code and use `/sruja-architecture` in chat. See [TEST_ON_REAL_PROJECTS.md](TEST_ON_REAL_PROJECTS.md#testing-the-sruja-skill-and-slash-command-on-real-projects) for the full checklist.

**Test with Cursor CLI (`agent`) locally:** Cursor CLI runs only on your machine (no CI). To test skills on cloned repos using the terminal: clone with `./setup_repos.sh`, then run the agent inside a repo. See [LOCAL_CURSOR_CLI_TESTING.md](LOCAL_CURSOR_CLI_TESTING.md).

**Test with OpenCode CLI (`opencode`) locally:** OpenCode CLI runs on your machine. To test Sruja skills on cloned repos using OpenCode: clone with `./setup_repos.sh`, then run `opencode` inside a repo to generate `architecture.sruja`, validate with `sruja lint`, and optionally run drift/evaluation. See [LOCAL_OPENCODE_CLI_TESTING.md](LOCAL_OPENCODE_CLI_TESTING.md).

**Realistic applications:** Use product-like repos (not frameworks):  
`./setup_repos.sh --apps` — clones gitea, saleor, documenso, cal.com. Then run quickstart/drift on them. See [run_results/REALISTIC_APPS_RUN_SUMMARY.md](run_results/REALISTIC_APPS_RUN_SUMMARY.md) for a run summary.

**Test customer-facing applications (one script):** Run Sruja (quickstart + drift + discover) on a curated set of product-like apps (gitea, saleor, documenso, cal.com, react-admin). Clone then test:  
`./run_customer_facing_apps_test.sh --setup`  
Or, if repos already exist:  
`./run_customer_facing_apps_test.sh`  
Report: `run_results/CUSTOMER_FACING_APPS_TEST_<timestamp>.md`. See [CUSTOMER_FACING_APPS_TEST.md](CUSTOMER_FACING_APPS_TEST.md).

**Multiple repos:** Run quickstart + drift on all test-repos and get a summary table:  
`./run_test_and_observe.sh --no-clone --multi-repo`  
See [SKILLS_VS_CLI_AND_DOES_IT_HELP.md](SKILLS_VS_CLI_AND_DOES_IT_HELP.md) for what uses skills vs CLI and how to tell if the skill helps.

Integration is skills + CLI; no API keys required for evaluation.

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