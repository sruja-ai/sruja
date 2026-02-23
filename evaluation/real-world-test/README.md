# Sruja Real-World Usefulness Test

## Goal

Test whether Sruja AI skills can effectively generate useful architecture documentation from real-world codebases.

## Approach

1. **Pick 5 popular repositories** with different architectures
2. **Clone and analyze** each codebase using Sruja AI skills
3. **Generate architecture DSL** for each project
4. **Evaluate usefulness** - Does the generated architecture help understand the codebase?

## Test Repositories

We picked 5 diverse, popular open-source projects:

### 1. **Express.js** (Node.js web framework)
- **Why**: Simple, well-documented, middleware architecture
- **GitHub**: https://github.com/expressjs/express
- **Expected**: Clear layered architecture with middleware pipeline

### 2. **Redis** (In-memory database)
- **Why**: Complex C codebase, multiple subsystems
- **GitHub**: https://github.com/redis/redis
- **Expected**: Client-server architecture with data structures

### 3. **React** (UI library)
- **Why**: Component-based, complex state management
- **GitHub**: https://github.com/facebook/react
- **Expected**: Component tree, reconciliation, fiber architecture

### 4. **Kubernetes** (Container orchestration)
- **Why**: Microservices, complex distributed system
- **GitHub**: https://github.com/kubernetes/kubernetes
- **Expected**: Control plane, worker nodes, API-driven

### 5. **VS Code** (Code editor)
- **Why**: Extension-based architecture, Electron app
- **GitHub**: https://github.com/microsoft/vscode
- **Expected**: Extension host, main process, renderer process

## Test Process

### Step 1: Clone Repositories

```bash
# Create test directory
mkdir -p test-repos
cd test-repos

# Clone repositories (shallow clone to save time)
git clone --depth 1 https://github.com/expressjs/express.git
git clone --depth 1 https://github.com/redis/redis.git
git clone --depth 1 https://github.com/facebook/react.git
git clone --depth 1 https://github.com/kubernetes/kubernetes.git
git clone --depth 1 https://github.com/microsoft/vscode.git
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
├── redis/
│   └── architecture.sruja
├── react/
│   └── architecture.sruja
├── kubernetes/
│   └── architecture.sruja
└── vscode/
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

### Option B: Simple LLM Check

Use an LLM to quickly evaluate:

```bash
# For each repository
cat test-repos/express/architecture.sruja | llm-prompt "
You are a software architect. Evaluate this generated architecture DSL for the Express.js codebase.

Rate 1-10 on:
1. Completeness (are main components captured?)
2. Accuracy (does it match Express.js architecture?)
3. Usefulness (would it help a new developer?)

Provide brief justification for each score.
"
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
✅ **At least 1 complex repository** (Kubernetes/VS Code) scores **≥6/10**
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

## Quick Start Script

```bash
#!/bin/bash
# setup-test.sh

echo "Setting up Sruja real-world test..."

# Create directories
mkdir -p test-repos results

# Clone repositories
cd test-repos

echo "Cloning repositories..."
git clone --depth 1 https://github.com/expressjs/express.git &
git clone --depth 1 https://github.com/redis/redis.git &
git clone --depth 1 https://github.com/facebook/react.git &
git clone --depth 1 https://github.com/kubernetes/kubernetes.git &
git clone --depth 1 https://github.com/microsoft/vscode.git &

wait
echo "All repositories cloned!"

echo ""
echo "Next steps:"
echo "1. Open each repository in your editor"
echo "2. Use Sruja AI skills to generate architecture"
echo "3. Save as architecture.sruja"
echo "4. Run evaluation"
echo ""
echo "Example for Express:"
echo "  cd test-repos/express"
echo "  code .  # Open in VS Code"
echo "  # In Copilot Chat: @sruja Generate architecture DSL for this codebase"
```

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
A: Diverse architectures, popular/well-known, different languages, varying complexity.

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