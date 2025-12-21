# Contribution Ideas

This document lists specific ways you can contribute to Sruja, especially useful when there aren't many GitHub issues yet.

## 🎯 Quick Wins (No Issue Required)

These are things you can start working on right away without waiting for an issue:

### Documentation

1. **Fix Typos**
   - Read through `README.md`, `docs/`, and `apps/website/src/content/`
   - Fix any typos or grammatical errors
   - Submit a PR with `docs: fix typo in [file]`

2. **Improve Clarity**
   - Find unclear sentences in documentation
   - Rewrite for better clarity
   - Add examples where helpful

3. **Add Missing Documentation**
   - Check if all commands are documented
   - Add examples to existing docs
   - Create missing guides

### Examples

4. **Add Example Architectures**
   - Create new `.sruja` files in `examples/` directory
   - Add interesting architecture patterns:
     - Microservices architecture
     - Event-driven architecture
     - Serverless architecture
     - Monolithic architecture
     - Distributed systems
   - Add to `examples/manifest.json`
   - Include comments explaining the architecture

5. **Improve Existing Examples**
   - Add comments to existing examples
   - Add more realistic scenarios
   - Fix any issues in examples

### Testing

6. **Add Test Cases**
   - Find functions without tests in `pkg/`
   - Write unit tests
   - Add edge case tests
   - Improve test coverage

7. **Test the CLI**
   - Try all CLI commands
   - Test with different inputs
   - Report any bugs you find
   - Suggest improvements

8. **Test the Website**
   - Browse the website
   - Try all features
   - Test on different browsers
   - Report bugs or UX issues

### Content

9. **Write Tutorials**
   - Create step-by-step tutorials
   - Cover common use cases
   - Add to `apps/website/src/content/tutorials/`

10. **Write Blog Posts**
    - Share your experience using Sruja
    - Write about architecture patterns
    - Add to `apps/website/src/content/blog/`

11. **Create Courses**
    - Design learning paths
    - Create course content
    - Add to `apps/website/src/content/courses/`

### Code Quality

12. **Improve Error Messages**
    - Find unclear error messages
    - Make them more helpful
    - Add context (filename, line number)

13. **Add CLI Help Text**
    - Improve command descriptions
    - Add examples to help text
    - Make help more user-friendly

14. **Code Comments**
    - Add comments to complex code
    - Explain "why" not just "what"
    - Improve code readability

## 🚀 Feature Ideas (Discuss First)

These are bigger contributions - consider opening a discussion or issue first:

### Language Features

- **New Export Formats**: Add support for PlantUML, Mermaid, etc.
- **New Validation Rules**: Add more architectural checks
- **Language Improvements**: Enhance the DSL syntax
- **Better Error Messages**: Improve parser error reporting

### Tooling

- **IDE Plugins**: Create plugins for other editors
- **CI/CD Integration**: Add GitHub Actions templates
- **Documentation Tools**: Improve documentation generation
- **Testing Tools**: Add more testing utilities

### Website Features

- **New Pages**: Add missing documentation pages
- **Interactive Features**: Enhance the playground
- **Search Improvements**: Improve search functionality
- **UI/UX**: Improve website design and usability

## 📋 Specific Tasks

### For Beginners

1. **Add an Example**
   ```bash
   # 1. Create a new file
   vim examples/my-architecture.sruja
   
   # 2. Add your architecture
   specification {
       element system
   }
   model {
       # ... your code ...
       sys = system "My System"
   }
   
   # 3. Add to manifest.json
   # 4. Test it: ./bin/sruja compile examples/my-architecture.sruja
   # 5. Submit PR
   ```

2. **Fix a Typo**
   - Find a typo in any `.md` file
   - Fix it
   - Submit PR with `docs: fix typo`

3. **Add a Test**
   - Find a function in `pkg/` without tests
   - Write a test
   - Submit PR with `test: add test for [function]`

### For Intermediate Contributors

4. **Improve Error Messages**
   - Find error messages in `pkg/`
   - Make them more descriptive
   - Add context (filename, line number)

5. **Add a New Export Format**
   - Study existing exporters in `pkg/export/`
   - Implement a new format
   - Add tests
   - Update documentation

6. **Add Validation Rules**
   - Study existing rules in `pkg/engine/`
   - Add a new validation rule
   - Add tests
   - Update documentation

## 💡 How to Proceed

1. **Pick something from this list**
2. **Check if it's already being worked on** (search GitHub issues/PRs)
3. **Start working on it**
4. **Open a PR** - even if it's a draft, it's good to show what you're working on
5. **Ask for feedback** - we're happy to help!

## 🎯 Priority Areas

Right now, these areas could use the most help:

1. **Examples** - More diverse architecture examples
2. **Documentation** - More tutorials and guides
3. **Testing** - More test coverage
4. **Error Messages** - Better user experience
5. **Content** - Blog posts, courses, tutorials

## 📝 Contribution Template

When you start working on something, you can:

1. **Open a Draft PR** with "[WIP]" in the title
2. **Describe what you're working on** in the PR description
3. **Ask for feedback** early
4. **Iterate based on feedback**

This way, maintainers can guide you and you won't waste time on something that might not fit.

## 🤝 Need Help?

- 💬 **Discord**: https://discord.gg/VNrvHPV5
- 💬 **GitHub Discussions**: Ask questions
- 📝 **Open an Issue**: Discuss your idea first

Remember: **No contribution is too small!** Even fixing a typo helps the project. 🚀

