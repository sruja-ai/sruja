# Quick Start: Test Sruja on Real Projects

**Goal**: Test if Sruja AI skills can generate useful architecture documentation from real codebases.

**Time needed**: ~2-3 hours (including evaluation)

## Step 1: Setup (10 minutes)

```bash
# Navigate to evaluation directory
cd sruja/evaluation/real-world-test

# Install dependencies (optional, for automated evaluation)
pip install openai  # Only if you want LLM-based evaluation

# Set API key (optional, for LLM evaluation)
export OPENAI_API_KEY="sk-proj-..."
```

## Step 2: Clone Test Repositories (15 minutes)

```bash
# Run the setup script
python setup_repos.py

# This will clone 5 popular repos:
# - Express.js (Node.js web framework)
# - FastAPI (Python API framework)
# - Next.js (React framework)
# - Prometheus (Go monitoring system)
# - Django (Python web framework)
```

**Alternative: Manual clone**
```bash
mkdir -p test-repos
cd test-repos

git clone --depth 1 https://github.com/expressjs/express.git
git clone --depth 1 https://github.com/tiangolo/fastapi.git
git clone --depth 1 https://github.com/vercel/next.js.git
git clone --depth 1 https://github.com/prometheus/prometheus.git
git clone --depth 1 https://github.com/django/django.git
```

## Step 3: Generate Architectures (1-2 hours)

For each repository, use Sruja AI skills to generate architecture:

### Using Cursor/VS Code with Copilot Chat

```bash
# Open repository in editor
cd test-repos/express
code .  # or cursor .
```

**In the AI chat, use this prompt:**
```
Analyze this codebase and generate a Sruja architecture DSL file.

The file should capture:
- Main components and modules
- Key data flows and relationships
- External dependencies
- Technology choices

Follow Sruja DSL conventions with:
- Clear descriptions for each component
- Specific relationship labels
- Proper nesting and hierarchy

Save the output as architecture.sruja in the repository root.
```

### Save the Generated File

1. Copy the AI-generated DSL
2. Save it as `test-repos/express/architecture.sruja`
3. Repeat for all 5 repositories

## Step 4: Evaluate (30-60 minutes)

### Option A: Quick Manual Check

For each generated architecture:

```bash
# Run evaluation script
python evaluate_architecture.py express

# This will:
# 1. Show file statistics
# 2. Run validation (if sruja CLI installed)
# 3. Display evaluation checklist
# 4. Generate a report
```

**Answer the checklist questions:**
- Completeness: Are main components captured? (1-10)
- Accuracy: Does it match the codebase? (1-10)
- Clarity: Is it understandable? (1-10)
- Usefulness: Would it help a new developer? (1-10)

### Option B: LLM-Assisted Evaluation

```bash
# Use GPT-4 to evaluate (requires OPENAI_API_KEY)
python evaluate_architecture.py express --llm

# This will automatically score the architecture
# and identify strengths/weaknesses
```

## Step 5: Review Results

### Check Generated Reports

```bash
# View reports
ls results/

# Read a specific report
cat results/evaluation_express_*.md
```

### Compare All Repositories

Create a summary table:

| Repository | Completeness | Accuracy | Clarity | Usefulness | Average | Verdict |
|------------|--------------|----------|---------|------------|---------|---------|
| Express.js | __/10 | __/10 | __/10 | __/10 | __/10 | |
| FastAPI | __/10 | __/10 | __/10 | __/10 | __/10 | |
| Next.js | __/10 | __/10 | __/10 | __/10 | __/10 | |
| Prometheus | __/10 | __/10 | __/10 | __/10 | __/10 | |
| Django | __/10 | __/10 | __/10 | __/10 | __/10 | |

## Success Criteria

Sruja is **useful** if:
- ✅ Average score ≥ 7/10 across all repos
- ✅ At least 3/5 repos score ≥ 7/10
- ✅ No repo scores < 5/10
- ✅ Generated architectures are accurate (no hallucinations)

## What to Look For

### Good Signs ✅
- Correct component names
- Real relationships (not made up)
- Captures key architectural patterns
- Would actually help understanding
- Clear and readable

### Red Flags ❌
- Fabricated components that don't exist
- Missing major subsystems
- Wrong technology choices
- Confusing or overly complex
- Doesn't match the code at all

## Example Output

### Good Architecture (Express.js)

```sruja
express = system "Express.js" {
  description "Fast, unopinionated, minimalist web framework for Node.js"
  
  router = container "Router" {
    technology "JavaScript"
    description "Core routing system handling HTTP requests"
  }
  
  middleware = container "Middleware" {
    technology "JavaScript"
    description "Middleware pipeline for request processing"
  }
  
  app = container "Application" {
    technology "JavaScript"
    description "Main application factory and configuration"
  }
}

// Real relationships from the codebase
client -> express.app "HTTP request"
express.app -> express.middleware "processes through"
express.middleware -> express.router "routes to"
```

**Score**: 8/10 (Accurate, complete, useful)

### Poor Architecture (Made Up)

```sruja
express = system "Express.js" {
  database = container "Database" {  // ❌ Express doesn't have a database!
    technology "PostgreSQL"
  }
  
  frontend = container "Frontend" {  // ❌ Express is backend-only!
    technology "React"
  }
}
```

**Score**: 2/10 (Inaccurate, hallucinated components)

## Next Steps

1. **If results are good (≥7/10)**:
   - ✅ Sruja is useful for real projects
   - Share success stories
   - Use for your own projects

2. **If results are mixed (5-6/10)**:
   - ⚠️ Sruja shows promise
   - Identify what's missing
   - Provide feedback to improve

3. **If results are poor (<5/10)**:
   - ❌ Sruja needs improvement
   - Document specific failures
   - Consider alternative approaches

## Troubleshooting

**"No architecture.sruja found"**
- Make sure you ran the AI generation step
- Check the file is saved in the repository root

**"sruja command not found"**
- Install Sruja CLI: `curl -fsSL https://sruja.ai/install.sh | bash`
- Or skip validation (it's optional)

**"OPENAI_API_KEY not set"**
- Set the environment variable
- Or use manual evaluation instead

**"Generated architecture is wrong"**
- This is valuable feedback! Document what went wrong
- Try a different prompt for generation
- Note which repos worked better than others

## Tips

1. **Start simple**: Test Express.js first (it's the smallest)
2. **Compare with docs**: Check if repo has existing architecture docs
3. **Be critical**: Honest evaluation helps improve Sruja
4. **Take notes**: Document what worked and what didn't
5. **Share results**: Your findings help the community

## Need Help?

- **Detailed guide**: See `EVALUATION_GUIDE.md`
- **Examples**: Check `examples/` directory
- **Issues**: Open a GitHub issue
- **Community**: Join Discord

---

**Ready to test?**

```bash
python setup_repos.py
# Then generate architectures for each repo
# Then evaluate with: python evaluate_architecture.py <repo-name>
```

**Time to answer**: Is Sruja actually useful for understanding real-world codebases? 🚀