---
title: "CI/CD Integration"
weight: 50
summary: "Integrate Sruja validation and documentation into your CI/CD pipelines for automated architecture governance."
tags: ["devops", "cicd", "automation", "governance"]
difficulty: "intermediate"
estimatedTime: "20 min"
---

# CI/CD Integration

Integrate Sruja into your CI/CD pipeline to automatically validate architecture, enforce standards, and generate documentation on every commit.

## Why CI/CD Integration?

**For DevOps teams:**
- Catch architecture violations before they reach production
- Automate documentation generation
- Enforce architectural standards across teams
- Reduce manual review overhead

**For software architects:**
- Ensure architectural decisions are documented
- Prevent architectural drift
- Scale governance across multiple teams

**For product teams:**
- Keep architecture docs up-to-date automatically
- Track architecture changes over time
- Ensure compliance with requirements

## Real-World Scenario

**Challenge**: A team of 50 engineers across 10 microservices. Architecture documentation is outdated, and violations happen frequently.

**Solution**: Integrate Sruja validation into CI/CD to:
- Validate architecture on every PR
- Generate updated documentation automatically
- Block merges if constraints are violated
- Track architecture changes over time

## GitHub Actions Integration

### Basic Setup

```yaml
name: Architecture Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  validate-architecture:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/go/bin" >> $GITHUB_PATH

      - name: Validate Architecture
        run: |
          sruja lint architecture.sruja

      - name: Export Documentation
        run: |
          sruja export markdown architecture.sruja > architecture.md
          sruja export json architecture.sruja > architecture.json

      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: architecture-docs
          path: |
            architecture.md
            architecture.json
```

### Advanced: Enforce Constraints

```yaml
name: Architecture Governance

on: [pull_request]

jobs:
  enforce-architecture:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for diff

      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/go/bin" >> $GITHUB_PATH

      - name: Validate Architecture
        id: validate
        run: |
          sruja lint architecture.sruja > lint-output.txt 2>&1
          exit_code=$?
          echo "exit_code=$exit_code" >> $GITHUB_OUTPUT
          cat lint-output.txt

      - name: Check for Constraint Violations
        if: steps.validate.outputs.exit_code != 0
        run: |
          echo "❌ Architecture validation failed!"
          echo "Please fix the errors before merging."
          exit 1

      - name: Comment PR with Validation Results
        if: always()
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const lintOutput = fs.readFileSync('lint-output.txt', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Architecture Validation Results\n\n\`\`\`\n${lintOutput}\n\`\`\``
            });
```

### Multi-Architecture Validation

For monorepos with multiple architecture files:

```yaml
name: Validate All Architectures

on: [push, pull_request]

jobs:
  validate-all:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        architecture:
          - architecture.sruja
          - services/payment-service.sruja
          - services/order-service.sruja
          - services/user-service.sruja
    steps:
      - uses: actions/checkout@v4

      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/go/bin" >> $GITHUB_PATH

      - name: Validate ${{ matrix.architecture }}
        run: |
          sruja lint ${{ matrix.architecture }}
```

## GitLab CI Integration

```yaml
stages:
  - validate

validate-architecture:
  stage: validate
  image: golang:1.21
  before_script:
    - curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
    - export PATH="$HOME/go/bin:$PATH"
  script:
    - sruja lint architecture.sruja
    - sruja export markdown architecture.sruja > architecture.md
    - sruja export json architecture.sruja > architecture.json
  artifacts:
    paths:
      - architecture.md
      - architecture.json
    expire_in: 30 days
  only:
    - merge_requests
    - main
```

## Jenkins Integration

```groovy
pipeline {
    agent any
    
    stages {
        stage('Validate Architecture') {
            steps {
                sh '''
                    curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
                    export PATH="$HOME/go/bin:$PATH"
                    sruja lint architecture.sruja
                '''
            }
        }
        
        stage('Generate Documentation') {
            steps {
                sh '''
                    export PATH="$HOME/go/bin:$PATH"
                    sruja export markdown architecture.sruja > architecture.md
                    sruja export json architecture.sruja > architecture.json
                '''
            }
        }
        
        stage('Archive Documentation') {
            steps {
                archiveArtifacts artifacts: 'architecture.*', fingerprint: true
            }
        }
    }
    
    post {
        failure {
            emailext (
                subject: "Architecture Validation Failed: ${env.JOB_NAME} - ${env.BUILD_NUMBER}",
                body: "Architecture validation failed. Please check the build logs.",
                to: "${env.CHANGE_AUTHOR_EMAIL}"
            )
        }
    }
}
```

## CircleCI Integration

```yaml
version: 2.1

jobs:
  validate-architecture:
    docker:
      - image: golang:1.21
    steps:
      - checkout
      - run:
          name: Install Sruja
          command: |
            curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
            export PATH="$HOME/go/bin:$PATH"
      - run:
          name: Validate
          command: |
            export PATH="$HOME/go/bin:$PATH"
            sruja lint architecture.sruja
      - run:
          name: Generate Docs
          command: |
            export PATH="$HOME/go/bin:$PATH"
            sruja export markdown architecture.sruja > architecture.md
      - store_artifacts:
          path: architecture.md

workflows:
  version: 2
  validate:
    jobs:
      - validate-architecture
```

## Pre-commit Hooks

Validate before every commit locally:

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Install Sruja if not present
if ! command -v sruja &> /dev/null; then
    echo "Installing Sruja..."
    curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
    export PATH="$HOME/go/bin:$PATH"
fi

# Validate architecture
sruja lint architecture.sruja
if [ $? -ne 0 ]; then
    echo "❌ Architecture validation failed. Fix errors before committing."
    exit 1
fi

# Format architecture file
sruja fmt architecture.sruja > architecture.formatted.sruja
mv architecture.formatted.sruja architecture.sruja
git add architecture.sruja

exit 0
```

Or use pre-commit framework:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: sruja-lint
        name: Sruja Lint
        entry: bash -c 'curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash && export PATH="$HOME/go/bin:$PATH" && sruja lint'
        language: system
        files: \.sruja$
        pass_filenames: true
```

## Automated Documentation Updates

Generate and commit documentation automatically:

```yaml
name: Update Architecture Docs

on:
  push:
    branches: [main]
    paths:
      - 'architecture.sruja'

jobs:
  update-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/go/bin" >> $GITHUB_PATH

      - name: Generate Documentation
        run: |
          sruja export markdown architecture.sruja > docs/architecture.md
          sruja export json architecture.sruja > docs/architecture.json

      - name: Commit Changes
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add docs/architecture.*
          git diff --staged --quiet || git commit -m "docs: update architecture documentation"
          git push
```

## Architecture Change Tracking

Track architecture changes over time:

```yaml
name: Track Architecture Changes

on:
  pull_request:
    paths:
      - 'architecture.sruja'

jobs:
  track-changes:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/go/bin" >> $GITHUB_PATH

      - name: Compare Architectures
        run: |
          # Get base version
          git show origin/${{ github.base_ref }}:architecture.sruja > base.sruja
          
          # Export both versions
          sruja export json base.sruja > base.json
          sruja export json architecture.sruja > current.json
          
          # Compare (using jq or custom script)
          echo "## Architecture Changes" >> $GITHUB_STEP_SUMMARY
          echo "Comparing base and current architecture..." >> $GITHUB_STEP_SUMMARY
          
      - name: Comment Changes
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '## Architecture Changes Detected\n\nReview the architecture changes in this PR.'
            });
```

## Real-World Example: Microservices Platform

Complete CI/CD setup for a microservices platform:

```yaml
name: Architecture Governance

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  validate-architecture:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service:
          - payment-service
          - order-service
          - user-service
          - inventory-service
    steps:
      - uses: actions/checkout@v4

      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/go/bin" >> $GITHUB_PATH

      - name: Validate ${{ matrix.service }}
        run: |
          sruja lint services/${{ matrix.service }}/architecture.sruja

      - name: Check Constraints
        run: |
          sruja lint --strict services/${{ matrix.service }}/architecture.sruja
          if [ $? -ne 0 ]; then
            echo "❌ Constraint violations found. PR cannot be merged."
            exit 1
          fi

      - name: Generate Service Docs
        run: |
          sruja export markdown services/${{ matrix.service }}/architecture.sruja > docs/services/${{ matrix.service }}.md

  validate-platform:
    runs-on: ubuntu-latest
    needs: validate-architecture
    steps:
      - uses: actions/checkout@v4

      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/go/bin" >> $GITHUB_PATH

      - name: Validate Platform Architecture
        run: |
          sruja lint platform-architecture.sruja

      - name: Generate Platform Docs
        run: |
          sruja export markdown platform-architecture.sruja > docs/platform.md
          sruja export json platform-architecture.sruja > docs/platform.json

      - name: Upload Documentation
        uses: actions/upload-artifact@v3
        with:
          name: architecture-docs
          path: docs/
```

## Key Takeaways

1. **Automate everything**: Don't rely on manual validation
2. **Fail fast**: Block merges if constraints are violated
3. **Generate docs automatically**: Keep documentation up-to-date
4. **Track changes**: Monitor architecture evolution over time
5. **Scale governance**: Use CI/CD to enforce standards across teams

## Exercise: Set Up CI/CD Integration

**Tasks:**
1. Choose a CI/CD platform (GitHub Actions, GitLab CI, etc.)
2. Create a workflow that validates architecture on every PR
3. Add documentation generation
4. Test the workflow with a sample architecture file

**Time**: 20 minutes

## Further Reading

- Tutorial: [Validation & Linting](/tutorials/basic/validation-linting)
- Docs: [Adoption Playbook](/docs/adoption-playbook)
- Course: [Advanced Architects - Policy as Code](/courses/advanced-architects/module-1-policy-as-code)
