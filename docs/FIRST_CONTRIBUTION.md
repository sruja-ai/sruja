# Your First Contribution to Sruja

Welcome! This guide will walk you through making your first contribution to Sruja, even if you're new to the project.

## What is Sruja?

Sruja is an architecture-as-code language that helps you define and visualize software architectures. Think of it as a way to describe your system's structure in code, similar to how you might describe data structures.

## Ways to Contribute (No Code Required!)

You don't need to write code to contribute! Here are easy ways to get started:

### 1. 📝 Documentation
- Fix typos or improve clarity
- Add examples to existing docs
- Translate documentation
- Write tutorials or blog posts

### 2. 🐛 Testing & Bug Reports
- Test the CLI and report issues
- Try examples and suggest improvements
- Test the website and report bugs
- Verify documentation accuracy

### 3. 💡 Examples
- Add new example `.sruja` files to `book/valid-examples/`
- Improve existing canonical examples and the matching book pages under `book/src/examples/`
- Create example architectures for different domains

### 4. 🎨 Content
- Write blog posts about architecture
- Create tutorials
- Improve course content
- Add challenges or quizzes

## Your First Code Contribution

### Step 1: Find Something to Work On

**Option A: Find an Issue (if available)**
1. Go to [GitHub Issues](https://github.com/sruja-ai/sruja/issues)
2. Look for issues labeled `good first issue` or `help wanted`
3. Read the issue carefully
4. Comment "I'd like to work on this" to claim it

**Option B: Pick from Contribution Ideas (no issue needed!)**
Since the project is new, there may not be many issues yet. That's okay! Check out:
- Common tasks that don't need issues:
  - Fix typos in documentation
  - Add examples to `book/valid-examples/`
  - Improve error messages
  - Add test cases
  - Write tutorials or blog posts

**Good First Contributions:**
- Documentation improvements (typos, clarity)
- Adding examples
- Adding test cases
- Fixing small bugs
- Improving error messages
- Writing content (tutorials, blog posts)

### Step 2: Set Up Your Environment

```bash
# 1. Fork the repository on GitHub
# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/sruja.git
cd sruja

# 3. Install Go (if you don't have it)
# Visit: https://go.dev/doc/install

# 4. Install dependencies
go mod download

# 5. Build the CLI
make build

# 6. Test that it works
./bin/sruja --help
```

### Step 3: Create a Branch

```bash
# Create a new branch for your changes
git checkout -b fix/your-issue-name

# Example:
git checkout -b docs/fix-typo-in-readme
```

### Step 4: Make Your Changes

- Make small, focused changes
- Test your changes locally
- Follow the coding style (run `make fmt`)

**Example: Fixing a Typo**

```bash
# 1. Edit the file
vim README.md  # or use your favorite editor

# 2. Check your changes
git diff

# 3. Test that everything still works
make build
make test
```

### Step 5: Commit Your Changes

```bash
# Stage your changes
git add README.md

# Commit with a clear message
git commit -m "docs: fix typo in README"

# Push to your fork
git push origin fix/your-issue-name
```

**Commit Message Format:**
- `docs: fix typo in README`
- `fix: correct error message`
- `feat: add new example`
- `test: add test case for X`

### Step 6: Open a Pull Request

1. Go to your fork on GitHub
2. Click "New Pull Request"
3. Select your branch
4. Fill out the PR template
5. Click "Create Pull Request"

**PR Checklist:**
- [ ] Linked to the issue
- [ ] Changes are small and focused
- [ ] Tests pass (`make test`)
- [ ] Code is formatted (`make fmt`)
- [ ] Documentation updated if needed

### Step 7: Respond to Feedback

- Maintainers may request changes
- Don't worry - this is normal!
- Make the requested changes
- Push updates to your branch
- The PR will update automatically

## Getting Help

**Stuck? We're here to help!**

- 💬 **Discord**: https://discord.gg/VNrvHPV5
- 💬 **GitHub Discussions**: Ask questions
- 📝 **GitHub Issues**: Report problems
- 📧 **Comment on your PR**: Ask for clarification

## Common First Contributions

### 1. Fix a Typo
- Find a typo in documentation
- Fix it
- Submit a PR with `docs: fix typo in [file]`

### 2. Add an Example
- Create a new `.sruja` file in `book/valid-examples/`
- Add a matching page under `book/src/examples/` (and link it from the Examples Gallery)
- Test it: `./bin/sruja compile book/valid-examples/your-file.sruja`
- Submit a PR with `feat: add example for [architecture type]`

### 3. Improve Documentation
- Find unclear documentation
- Rewrite for clarity
- Add examples
- Submit a PR with `docs: improve [topic] documentation`

### 4. Add a Test Case
- Find a function without tests
- Write a test
- Submit a PR with `test: add test for [function]`

### 5. Write a Tutorial
- Create a step-by-step tutorial
- Add to `apps/website/src/content/tutorials/`
- Submit a PR with `docs: add tutorial for [topic]`

**💡 Need more ideas?** Check [Contribution Ideas](CONTRIBUTION_IDEAS.md) for a complete list!

## What Happens Next?

1. **Review**: A maintainer will review your PR
2. **Feedback**: You may get suggestions for improvements
3. **Merge**: Once approved, your changes are merged!
4. **Celebration**: 🎉 You're now a contributor!

## Tips for Success

- ✅ Start small - fix one thing at a time
- ✅ Ask questions if you're unsure
- ✅ Be patient - reviews take time
- ✅ Learn from feedback
- ✅ Have fun!

## Next Steps

After your first contribution:
- Look for more `good first issue` labels
- Help review other PRs
- Answer questions from other contributors
- Consider becoming a maintainer

Welcome to the Sruja community! 🚀

