# AI-Era SDLC Guidelines Enhancement Plan

Comprehensive roadmap for transforming Rust Skills and Sruja Architecture guidelines into dynamic, AI-native development tools.

## Overview

**Problem:** Current guidelines are static knowledge bases that don't leverage AI capabilities for adaptive learning, context optimization, or developer workflow integration.

**Goal:** Transform into living systems that:

- Adapt to project context and codebase patterns
- Minimize AI context usage through smart filtering
- Integrate seamlessly with developer tools and CI/CD
- Learn from usage patterns to improve relevance
- Support community-driven evolution

**Timeline:** 6 months (phased approach)

---

## Executive Summary

| Aspect             | Current State        | Target State                         |
| ------------------ | -------------------- | ------------------------------------ |
| Knowledge Delivery | Static 336-line file | Dynamic 50-100 line filtered context |
| Rule Relevance     | Fixed hierarchy      | Context-aware weighting (+80%)       |
| Integration        | Manual reference     | Automated (IDE + CI/CD)              |
| Evolution          | Author-driven        | Community feedback loops             |
| Validation         | Manual review        | Automated testing                    |

**Success Metrics:**

- 70% reduction in AI context usage
- 80%+ rule relevance in suggestions
- 50% faster onboarding for new Rust developers
- 90%+ skill file validation rate in CI

---

## Phase 1: Quick Wins (Weeks 1-6)

### 1.1 Enhanced Rule Metadata

**Objective:** Add machine-readable metadata to enable smart filtering and prioritization.

**Implementation:**

````yaml
# skills/sruja-architecture/rules/principle-separation.md
---
metadata:
  complexity: 3           # 1-5: implementation difficulty
  frequency: common        # common/rare/very-rare
  confidence: high        # high/medium/low: rule certainty
  applicable:
    async: true          # applies to async code
    embedded: false      # applies to embedded systems
    wasm: false          # applies to WebAssembly
  category: critical
  level: intermediate    # beginner/intermediate/advanced
  rust_version: "1.0+"   # minimum Rust version
  alternatives:          # when this rule doesn't apply
    - "Use clone() in hot-path profiling code"
    - "Use clone() when benchmarking proves borrow overhead > 50ns"
  related_rules:
    - own-move-large
    - anti-clone-excessive
---

# Prefer borrowing over cloning

Use `&T` references instead of `.clone()` when you only need read access.

### Examples

**❌ Don't:**
```rust
fn process(data: Vec<i32>) {
    let copy = data.clone();
    analyze(&copy);
}
````

**✅ Do:**

```rust
fn process(data: Vec<i32>) {
    analyze(&data);
}
```

```

**Tasks:**
1. Design metadata schema (JSON Schema)
2. Create CLI tool to validate metadata
3. Add metadata to top 50 rules (CRITICAL + HIGH)
4. Generate metadata template for remaining rules

**Deliverables:**
- `schema/skill-metadata.json`
- `skill-lint check --metadata`
- Updated rule files (50/179)

---

### 1.2 Rule Level System

**Objective:** Categorize rules by experience level to reduce cognitive load.

**Structure:**

```

skills/sruja-architecture/
├── AGENTS.md   # Full compiled guide
├── SKILL.md    # Skill description
└── rules/
├── principle-separation.md
├── component-person.md
├── pattern-monolith.md
├── anti-god-component.md
└── ...

````

**Level Criteria:**

| Level | Audience | Rule Count | Focus Areas |
|-------|----------|------------|-------------|
| Beginner | New Rust devs | 20-25 | Critical safety, common errors |
| Intermediate | Daily users | 60-80 | Performance, patterns, idiomatic code |
| Advanced | Library authors | 80-100 | Optimization, edge cases, anti-patterns |

**Tasks:**
1. Define level criteria matrix
2. Categorize all 179 rules
3. Create filtered AGENTS.md for each level
4. Add cross-references between levels

**Deliverables:**
- `skills/sruja-architecture/AGENTS.md`

---

### 1.3 Selective Loading Mechanism

**Objective:** Enable AI to load only relevant rules based on context.

**CLI Design:**

```bash
# Basic filtering
/rust-skills --category critical,high
/rust-skills --level beginner
/rust-skills --project-type cli

# Advanced filtering
/rust-skills --filter complexity=1,2,3
/rust-skills --filter frequency=common
/rust-skills --exclude anti-

# Project-aware loading
/rust-skills --analyze . --profile dev
/rust-skills --project /path/to/project

# Output formats
/rust-skills --output json
/rust-skills --output markdown --concise
````

**Implementation:**

```rust
// crates/sruja-cli/src/cmds/skills.rs
pub struct SkillFilter {
    pub categories: Option<Vec<String>>,
    pub levels: Option<Vec<Level>>,
    pub project_type: Option<ProjectType>,
    pub complexity_range: Option<Range<u8>>,
    pub frequency: Option<Frequency>,
    pub exclude_prefixes: Vec<String>,
}

pub async fn load_skills(filter: &SkillFilter) -> Result<String> {
    let rules = load_all_rules()?;

    let filtered: Vec<_> = rules
        .into_iter()
        .filter(|rule| matches_filter(rule, filter))
        .collect();

    Ok(format_skills(&filtered, OutputFormat::Markdown))
}

fn analyze_project_context(path: &Path) -> ProjectContext {
    // Scan Cargo.toml for dependencies
    let has_tokio = has_dependency(path, "tokio");
    let has_actix = has_dependency(path, "actix-web");
    let is_library = is_crate_type(path, "lib");
    let is_cli = has_dependency(path, "clap");

    ProjectContext {
        async: has_tokio,
        web: has_actix,
        embedded: has_dependency(path, "embedded-hal"),
        wasm: has_dependency(path, "wasm-bindgen"),
        library: is_library,
        cli: is_cli,
    }
}
```

**Tasks:**

1. Implement SkillFilter struct
2. Build project analyzer (Cargo.toml scanning)
3. Add /rust-skills command to sruja CLI
4. Test filtering accuracy

**Deliverables:**

- `sruja skills --help`
- Project context analyzer
- Filtered skill output

---

### 1.4 Trade-off Context for Critical Rules

**Objective:** Provide nuanced guidance on when to break rules.

**Template:**

````markdown
### When to Break This Rule

Use `.clone()` when:

- **Hot-path profiling**: During benchmarking, you've proven borrow overhead exceeds performance targets
- **Prototyping**: Quick iteration takes priority over optimization
- **Data transformation**: Need to modify while preserving original
- **Closure captures**: Avoid complex lifetime annotations in performance-critical code

### Cost Analysis

| Scenario                       | Clone Cost | Borrow Cost | Recommendation       |
| ------------------------------ | ---------- | ----------- | -------------------- |
| Small struct (< 128 bytes)     | ~10ns      | ~0ns        | Use borrow           |
| Medium struct (128-512 bytes)  | ~50ns      | ~0ns        | Use borrow           |
| Large struct (> 512 bytes)     | ~200ns     | ~0ns        | Use borrow           |
| In tight loop (1M+ iterations) | ~50ms      | ~0ms        | Use borrow           |
| One-time operation             | Negligible | Negligible  | Clone for simplicity |

### Real-World Examples

**Acceptable Clone:**

```rust
// Prototyping stage
fn quick_demo(data: Vec<i32>) {
    let copy = data.clone();
    // Complex logic that would require lifetime changes
    println!("{:?}", copy);
}
```
````

**Unacceptable Clone:**

```rust
// In production hot path
fn process_millions(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(data.len());
    for &x in data {
        let copy = x.clone(); // Unnecessary clone for Copy type
        result.push(copy);
    }
    result
}
```

### Related Rules

- [`own-move-large`](rules/own-move-large.md) - Move large data instead of cloning
- [`perf-iter-lazy`](rules/perf-iter-lazy.md) - Keep iterators lazy
- [`anti-premature-optimize`](rules/anti-premature-optimize.md) - Profile before optimizing

### References

- [Rust Book - Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [Rust Performance Book - Cloning](https://nnethercote.github.io/perf-book/standard-library.html#clone)
- [clippy::clone_on_ref_ptr](https://rust-lang.github.io/rust-clippy/master/index.html#clone_on_ref_ptr)

````

**Rules to Enhance (Priority Order):**

1. **CRITICAL** (12 rules) - all need trade-off context
2. **HIGH** - async patterns, compiler optimization
3. **MEDIUM** - performance patterns, testing

**Tasks:**
1. Create trade-off context template
2. Add context to all CRITICAL rules
3. Add context to high-frequency MEDIUM rules
4. Review and cross-reference related rules

**Deliverables:**
- Updated rule files with trade-off sections
- Cross-reference index

---

## Phase 2: Tooling Integration (Weeks 7-12)

### 2.1 Rule Validation Tooling

**Objective:** Automated validation of skill files for consistency and correctness.

**CLI Tool:**

```bash
# Validate skill files
skill-lint check skills/sruja-architecture/

# Validate against schema
skill-lint validate --schema schema/skill-metadata.json

# Test rules with code samples
skill-lint test --generate-code

# Check for broken links
skill-lint check-links

# Format skill files
skill-lint format skills/sruja-architecture/
````

**Implementation:**

```rust
// crates/skill-lint/src/checker.rs
use serde_json::Value;
use regex::Regex;

pub struct SkillChecker {
    metadata_schema: Value,
    broken_links: Vec<String>,
}

impl SkillChecker {
    pub fn check_file(&self, path: &Path) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check frontmatter
        if let Some(frontmatter) = extract_frontmatter(path) {
            diagnostics.extend(self.check_metadata(&frontmatter));
        }

        // Check code examples compile
        diagnostics.extend(self.check_code_examples(path));

        // Check links
        diagnostics.extend(self.check_links(path));

        // Check cross-references
        diagnostics.extend(self.check_cross_references(path));

        diagnostics
    }

    fn check_code_examples(&self, path: &Path) -> Vec<Diagnostic> {
        let examples = extract_code_blocks(path);
        let mut diagnostics = Vec::new();

        for (idx, code) in examples.iter().enumerate() {
            let result = compile_code_snippet(code);
            if !result.success {
                diagnostics.push(Diagnostic {
                    level: Level::Error,
                    message: format!("Example {} doesn't compile: {}", idx + 1, result.error),
                    line: code.line_number,
                });
            }
        }

        diagnostics
    }
}

#[derive(Debug)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub line: usize,
}

#[derive(Debug)]
pub enum Level {
    Error,
    Warning,
    Info,
}
```

**JSON Schema:**

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Skill Rule Metadata",
  "type": "object",
  "required": ["metadata"],
  "properties": {
    "metadata": {
      "type": "object",
      "required": [
        "complexity",
        "frequency",
        "confidence",
        "category",
        "level"
      ],
      "properties": {
        "complexity": {
          "type": "integer",
          "minimum": 1,
          "maximum": 5,
          "description": "1-5 scale of implementation difficulty"
        },
        "frequency": {
          "type": "string",
          "enum": ["common", "rare", "very-rare"],
          "description": "How often this pattern appears in typical codebases"
        },
        "confidence": {
          "type": "string",
          "enum": ["high", "medium", "low"],
          "description": "Confidence that this rule is correct in all circumstances"
        },
        "category": {
          "type": "string",
          "enum": ["critical", "high", "medium", "low", "reference"]
        },
        "level": {
          "type": "string",
          "enum": ["beginner", "intermediate", "advanced"]
        },
        "applicable": {
          "type": "object",
          "properties": {
            "async": { "type": "boolean" },
            "embedded": { "type": "boolean" },
            "wasm": { "type": "boolean" }
          }
        }
      }
    }
  }
}
```

**Tasks:**

1. Design JSON schema for metadata
2. Implement skill-lint CLI
3. Add code example compilation
4. Integrate link checking
5. Create GitHub Action for validation

**Deliverables:**

- `schema/skill-metadata.json`
- `crates/skill-lint/` crate
- `.github/workflows/skill-validation.yml`

---

### 2.2 CI/CD Pipeline Integration

**Objective:** Ensure skill files are validated on every commit.

**GitHub Actions Workflow:**

```yaml
name: Skill Validation

on:
  push:
    paths:
      - "skills/**"
      - "schema/**"
  pull_request:
    paths:
      - "skills/**"
      - "schema/**"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build skill-lint
        run: cargo build --release --bin skill-lint

      - name: Validate metadata
        run: skill-lint validate --schema schema/skill-metadata.json skills/

      - name: Check code examples
        run: skill-lint test --generate-code skills/

      - name: Check links
        run: skill-lint check-links skills/

      - name: Check for broken cross-references
        run: skill-lint check-xrefs skills/

  format-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check formatting
        run: skill-lint format --check skills/
```

**Pre-commit Hook:**

```bash
# .git/hooks/pre-commit
#!/bin/bash

# Validate skill files before commit
if git diff --cached --name-only | grep -q "^skills/"; then
    echo "Validating skill files..."
    skill-lint check skills/
    if [ $? -ne 0 ]; then
        echo "Skill validation failed. Fix issues before committing."
        exit 1
    fi
fi
```

**Tasks:**

1. Create GitHub Actions workflow
2. Set up pre-commit hook installation script
3. Add skill-lint to development dependencies
4. Document CI/CD integration

**Deliverables:**

- `.github/workflows/skill-validation.yml`
- `scripts/install-git-hooks.sh`
- Updated DEVELOPMENT.md

---

### 2.3 Dynamic Rule Suggestion System

**Objective:** Analyze codebase to suggest most relevant rules.

**Analyzer Design:**

```rust
// crates/sruja-cli/src/analysis.rs
use std::collections::HashMap;

#[derive(Debug)]
pub struct ProjectContext {
    pub async: bool,
    pub web: bool,
    pub embedded: bool,
    pub wasm: bool,
    pub library: bool,
    pub cli: bool,
    pub complexity_score: f32, // 0-1 based on crate structure
}

#[derive(Debug)]
pub struct RuleScore {
    pub rule_id: String,
    pub relevance_score: f32,
    pub confidence_score: f32,
    pub reason: String,
}

pub fn analyze_project(path: &Path) -> ProjectContext {
    let cargo_toml = path.join("Cargo.toml");

    let mut context = ProjectContext {
        async: false,
        web: false,
        embedded: false,
        wasm: false,
        library: false,
        cli: false,
        complexity_score: 0.0,
    };

    // Parse Cargo.toml
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        let manifest: toml::Value = content.parse().unwrap();
        let deps = manifest["dependencies"].as_table();

        context.async = deps.contains_key("tokio") || deps.contains_key("async-std");
        context.web = deps.contains_key("actix-web") || deps.contains_key("axum");
        context.embedded = deps.contains_key("embedded-hal");
        context.wasm = deps.contains_key("wasm-bindgen");

        let crate_type = manifest["lib"].as_table();
        context.library = crate_type.is_some();
        context.cli = deps.contains_key("clap");
    }

    // Analyze source code complexity
    let src_path = path.join("src");
    context.complexity_score = calculate_complexity(&src_path);

    context
}

pub fn suggest_rules(context: &ProjectContext) -> Vec<RuleScore> {
    let rules = load_all_rules();
    let mut scored = Vec::new();

    for rule in rules {
        let mut relevance_score = 0.5; // Base relevance

        // Boost relevance based on project context
        if rule.metadata.applicable.async && context.async {
            relevance_score += 0.3;
        }
        if rule.metadata.applicable.embedded && context.embedded {
            relevance_score += 0.3;
        }
        if rule.metadata.applicable.wasm && context.wasm {
            relevance_score += 0.3;
        }

        // Adjust by complexity
        let complexity_diff = (rule.metadata.complexity as f32 / 5.0) - context.complexity_score;
        relevance_score -= complexity_diff.abs() * 0.2;

        // Adjust by frequency
        let frequency_boost = match rule.metadata.frequency {
            "common" => 0.2,
            "rare" => 0.1,
            "very-rare" => 0.0,
            _ => 0.0,
        };
        relevance_score += frequency_boost;

        // Adjust by confidence
        let confidence_multiplier = match rule.metadata.confidence {
            "high" => 1.0,
            "medium" => 0.8,
            "low" => 0.6,
            _ => 0.5,
        };

        scored.push(RuleScore {
            rule_id: rule.id,
            relevance_score: relevance_score.clamp(0.0, 1.0),
            confidence_score: confidence_multiplier,
            reason: explain_relevance(&rule, context),
        });
    }

    // Sort by combined score
    scored.sort_by(|a, b| {
        let score_a = a.relevance_score * a.confidence_score;
        let score_b = b.relevance_score * b.confidence_score;
        b.partial_cmp(&score_a).unwrap()
    });

    scored
}

fn calculate_complexity(src_path: &Path) -> f32 {
    // Simple heuristic: count modules, functions, traits
    let mut complexity = 0.0;

    if let Ok(entries) = fs::read_dir(src_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    complexity += content.lines().count() as f32 / 1000.0;
                }
            }
        }
    }

    (complexity / 10.0).clamp(0.0, 1.0)
}

fn explain_relevance(rule: &Rule, context: &ProjectContext) -> String {
    let mut reasons = Vec::new();

    if rule.metadata.applicable.async && context.async {
        reasons.push("async project");
    }
    if rule.metadata.applicable.web && context.web {
        reasons.push("web application");
    }
    if rule.metadata.applicable.embedded && context.embedded {
        reasons.push("embedded system");
    }
    if rule.metadata.applicable.wasm && context.wasm {
        reasons.push("WebAssembly target");
    }

    match rule.metadata.frequency {
        "common" => reasons.push("common pattern".to_string()),
        "rare" => reasons.push("edge case".to_string()),
        _ => {}
    }

    if reasons.is_empty() {
        "general recommendation".to_string()
    } else {
        reasons.join(", ")
    }
}
```

**Usage:**

```bash
# Analyze current project
sruja skills analyze --project .

# Suggest top 10 rules
sruja skills suggest --count 10 --project .

# Export suggestions as markdown
sruja skills suggest --output markdown --project . > RECOMMENDED_RULES.md
```

**Output Example:**

```markdown
# Recommended Rust Skills Rules

Based on analysis of your project, here are the most relevant rules:

## Top 10 Rules

1. **async-no-lock-await** (relevance: 0.95, confidence: 1.0)
   - Reason: async project, common pattern
   - This project uses tokio and has potential lock contention issues

2. **err-result-over-panic** (relevance: 0.93, confidence: 1.0)
   - Reason: general recommendation, common pattern
   - Critical for production error handling

3. **mem-with-capacity** (relevance: 0.88, confidence: 1.0)
   - Reason: common pattern
   - Performance optimization opportunity detected

## Analysis Summary

- Project type: Async web application
- Complexity score: 0.65 (medium)
- Total rules applicable: 142
- High-priority rules: 23
```

**Tasks:**

1. Implement project context analyzer
2. Build rule scoring algorithm
3. Add suggest command to CLI
4. Test on various project types
5. Create documentation

**Deliverables:**

- `sruja skills analyze`
- `sruja skills suggest`
- Project analysis reports

---

### 2.4 Learning System

**Objective:** Track usage patterns to improve rule suggestions over time.

**Data Collection:**

```json
// .skill-learn.json
{
  "project_id": "sha256-hash-of-cargo-toml",
  "last_analysis": "2025-02-08T10:30:00Z",
  "statistics": {
    "total_rules_suggested": 142,
    "rules_applied": 89,
    "violations_detected": 34
  },
  "violations": {
    "anti-unwrap-abuse": {
      "count": 12,
      "severity": "high",
      "locations": ["src/main.rs:42", "src/api/mod.rs:156"]
    },
    "mem-with-capacity": {
      "count": 8,
      "severity": "medium",
      "locations": ["src/parser.rs:23", "src/processor.rs:78"]
    }
  },
  "rule_effectiveness": {
    "async-tokio-runtime": {
      "suggested": true,
      "applied": true,
      "impact": "resolved-deadlock"
    },
    "mem-smallvec": {
      "suggested": true,
      "applied": false,
      "reason": "not-applicable"
    }
  },
  "patterns": {
    "most_violated_rules": ["anti-unwrap-abuse", "err-no-unwrap-prod"],
    "most_successful_rules": ["async-tokio-runtime", "err-result-over-panic"],
    "project_type_confidence": 0.95
  }
}
```

**Feedback Loop:**

```rust
// crates/sruja-cli/src/learning.rs
pub struct LearningSystem {
    db: sled::Db,
}

impl LearningSystem {
    pub fn track_violation(&self, rule_id: &str, location: &str) -> Result<()> {
        let key = format!("violations:{}", rule_id);
        let mut count: u32 = self.db.get(&key)?.unwrap_or(vec![0,0,0,0])
            .try_into().unwrap();

        count += 1;

        let mut violations_map = self.db.open_tree("violations")?;
        let entry = format!("{}:{}", rule_id, count);
        violations_map.insert(&entry, location)?;

        self.db.insert(&key, count.to_be_bytes())?;
        Ok(())
    }

    pub fn get_priority_rules(&self, count: usize) -> Vec<String> {
        let mut violations: Vec<(String, u32)> = Vec::new();

        for result in self.db.scan_prefix(b"violations:") {
            if let Ok((key, value)) = result {
                let rule_id = String::from_utf8_lossy(&key["violations:".len()..]).to_string();
                let count = u32::from_be_bytes(value.try_into().unwrap());
                violations.push((rule_id, count));
            }
        }

        violations.sort_by(|a, b| b.1.cmp(&a.1));
        violations.into_iter()
            .take(count)
            .map(|(rule_id, _)| rule_id)
            .collect()
    }

    pub fn calculate_rule_relevance(&self, rule_id: &str) -> f32 {
        // Base relevance from metadata
        let base_relevance = 0.5;

        // Boost based on violation history
        let violation_boost = match self.get_violation_count(rule_id) {
            0 => 0.0,
            1..=5 => 0.1,
            6..=10 => 0.2,
            _ => 0.3,
        };

        (base_relevance + violation_boost).clamp(0.0, 1.0)
    }
}
```

**CLI Integration:**

```bash
# Track violations
sruja skills track-violation --rule anti-unwrap-abuse --location src/main.rs:42

# Get learned priorities
sruja skills priorities

# Generate report
sruja skills report
```

**Tasks:**

1. Design learning data schema
2. Implement learning system with sled database
3. Add tracking commands to CLI
4. Integrate with skill-lint for violation detection
5. Create reporting tools

**Deliverables:**

- Learning database schema
- `sruja skills track-violation`
- `sruja skills priorities`
- `sruja skills report`

---

## Phase 3: Developer Experience (Weeks 13-20)

### 3.1 VSCode Extension

**Objective:** Inline rule suggestions and documentation in editor.

**Extension Features:**

```json
// .vscode/settings.json
{
  "rustSkills.enabled": true,
  "rustSkills.level": "intermediate",
  "rustSkills.projectType": "auto-detect",
  "rustSkills.autoSuggest": true,
  "rustSkills.highlightViolations": true,
  "rustSkills.showRulePreview": true
}
```

**Implementation:**

```typescript
// src/extension.ts
import * as vscode from "vscode";
import { RuleProvider } from "./RuleProvider";
import { ViolationAnalyzer } from "./ViolationAnalyzer";

export function activate(context: vscode.ExtensionContext) {
  const ruleProvider = new RuleProvider(context);
  const violationAnalyzer = new ViolationAnalyzer(ruleProvider);

  // Register inline hint provider
  context.subscriptions.push(
    vscode.languages.registerCodeLensProvider(
      { scheme: "file", language: "rust" },
      new RuleHintProvider(ruleProvider),
    ),
  );

  // Register hover provider for rule documentation
  context.subscriptions.push(
    vscode.languages.registerHoverProvider(
      { scheme: "file", language: "rust" },
      new RuleHoverProvider(ruleProvider),
    ),
  );

  // Register diagnostic provider for violations
  context.subscriptions.push(
    vscode.languages.registerDiagnosticCollection("rustSkills"),
  );

  // Analyze workspace on activation
  analyzeWorkspace(ruleProvider, violationAnalyzer);

  // Watch for file changes
  const watcher = vscode.workspace.createFileSystemWatcher("**/*.rs");
  watcher.onDidChange((uri) => {
    violationAnalyzer.analyzeFile(uri.fsPath);
  });
}

class RuleHintProvider implements vscode.CodeLensProvider {
  async provideCodeLenses(
    document: vscode.TextDocument,
  ): Promise<vscode.CodeLens[]> {
    const lenses: vscode.CodeLens[] = [];
    const text = document.getText();

    // Analyze code for rule violations
    const violations = analyzeCode(text);

    for (const violation of violations) {
      const range = new vscode.Range(
        new vscode.Position(violation.line, 0),
        new vscode.Position(violation.line + 1, 0),
      );

      lenses.push(
        new vscode.CodeLens(range, {
          title: `⚠️ ${violation.rule.title}`,
          command: "rustSkills.showRule",
          arguments: [violation.rule.id],
        }),
      );
    }

    return lenses;
  }
}

class RuleHoverProvider implements vscode.HoverProvider {
  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
  ): Promise<vscode.Hover | undefined> {
    const line = document.lineAt(position.line).text;

    // Check if line matches any rule pattern
    const rule = this.findRelevantRule(line);
    if (rule) {
      const markdown = new vscode.MarkdownString();
      markdown.appendMarkdown(`**${rule.title}**\n\n`);
      markdown.appendMarkdown(rule.description + "\n\n");
      markdown.appendMarkdown(
        `[View full documentation](command:rustSkills.showRule?${encodeURIComponent(JSON.stringify(rule.id))})`,
      );

      return new vscode.Hover(markdown);
    }

    return undefined;
  }
}

function analyzeWorkspace(
  ruleProvider: RuleProvider,
  violationAnalyzer: ViolationAnalyzer,
) {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) return;

  const root = workspaceFolders[0].uri.fsPath;

  // Analyze project context
  const context = analyzeProjectContext(root);
  const rules = ruleProvider.suggestRules(context);

  // Show suggestions in output channel
  const output = vscode.window.createOutputChannel("Rust Skills");
  output.appendLine("Relevant rules for this project:");
  for (const rule of rules.slice(0, 10)) {
    output.appendLine(`  ${rule.id} (${rule.relevance_score.toFixed(2)})`);
  }
}
```

**Configuration:**

```json
// package.json
{
  "contributes": {
    "configuration": {
      "title": "Rust Skills",
      "properties": {
        "rustSkills.enabled": {
          "type": "boolean",
          "default": true,
          "description": "Enable Rust Skills extension"
        },
        "rustSkills.level": {
          "type": "string",
          "enum": ["beginner", "intermediate", "advanced"],
          "default": "intermediate",
          "description": "Experience level for rule suggestions"
        },
        "rustSkills.autoSuggest": {
          "type": "boolean",
          "default": true,
          "description": "Automatically suggest rules while typing"
        },
        "rustSkills.highlightViolations": {
          "type": "boolean",
          "default": true,
          "description": "Highlight rule violations in code"
        }
      }
    },
    "commands": [
      {
        "command": "rustSkills.showRule",
        "title": "Show Rust Skills Rule",
        "category": "Rust Skills"
      },
      {
        "command": "rustSkills.analyzeProject",
        "title": "Analyze Project for Relevant Rules",
        "category": "Rust Skills"
      },
      {
        "command": "rustSkills.reportViolations",
        "title": "Generate Violation Report",
        "category": "Rust Skills"
      }
    ],
    "menus": {
      "editor/context": [
        {
          "command": "rustSkills.analyzeSelection",
          "when": "editorLangId == rust",
          "group": "rustSkills"
        }
      ]
    }
  }
}
```

**Tasks:**

1. Set up VSCode extension project
2. Implement rule suggestion provider
3. Add inline hints and hover support
4. Integrate with learning system
5. Create documentation
6. Test on various projects

**Deliverables:**

- VSCode extension package
- Extension marketplace listing
- User documentation

---

### 3.2 Rust-Analyzer Integration

**Objective:** Convert rules to rust-analyzer diagnostics for IDE integration.

**Chalk-like Rule Engine:**

```rust
// crates/rust-analyzer/src/rules/mod.rs
use std::sync::Arc;
use rustc_hash::FxHashMap;
use syntax::{ast, AstNode, TextRange, SyntaxNode};

#[derive(Debug, Clone)]
pub struct RuleDiagnostic {
    pub rule_id: &'static str,
    pub message: String,
    pub severity: Severity,
    pub range: TextRange,
    pub fix: Option<Suggestion>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub label: String,
    pub replacement: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

pub struct RuleEngine {
    rules: Vec<Arc<dyn Rule>>,
}

impl RuleEngine {
    pub fn new() -> Self {
        let mut rules: Vec<Arc<dyn Rule>> = Vec::new();

        // Register rules
        rules.push(Arc::new(AntiUnwrapAbuseRule));
        rules.push(Arc::new(SliceOverVecRule));
        rules.push(Arc::new(MemWithCapacityRule));

        RuleEngine { rules }
    }

    pub fn analyze(&self, node: &SyntaxNode) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        for rule in &self.rules {
            if let Some(diags) = rule.check(node) {
                diagnostics.extend(diags);
            }
        }

        diagnostics
    }
}

pub trait Rule: Send + Sync {
    fn check(&self, node: &SyntaxNode) -> Option<Vec<RuleDiagnostic>>;
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

struct AntiUnwrapAbuseRule;

impl Rule for AntiUnwrapAbuseRule {
    fn check(&self, node: &SyntaxNode) -> Option<Vec<RuleDiagnostic>> {
        // Find .unwrap() calls
        let mut diagnostics = Vec::new();

        for token in node.descendants() {
            if let Some(call_expr) = ast::MethodCallExpr::cast(token) {
                let name = call_expr.name_ref()?.text();
                if name == "unwrap" || name == "expect" {
                    diagnostics.push(RuleDiagnostic {
                        rule_id: "anti-unwrap-abuse",
                        message: "Avoid using .unwrap() in production code".to_string(),
                        severity: Severity::Warning,
                        range: call_expr.syntax().text_range(),
                        fix: Some(Suggestion {
                            label: "Use ? operator instead".to_string(),
                            replacement: "?".to_string(),
                            range: call_expr.syntax().text_range(),
                        }),
                    });
                }
            }
        }

        if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics)
        }
    }

    fn id(&self) -> &'static str {
        "anti-unwrap-abuse"
    }

    fn description(&self) -> &'static str {
        "Never use .unwrap() in production code"
    }
}
```

**Integration with rust-analyzer:**

```rust
// crates/rust-analyzer/src/diagnostics/rules.rs
use crate::db::RootDatabase;
use crate::rules::RuleEngine;
use crate::LineIndex;

pub fn check_rules(
    db: &RootDatabase,
    file_id: FileId,
) -> Vec<Diagnostic> {
    let source_file = db.parse(file_id).tree();
    let line_index = db.line_index(file_id);

    let rule_engine = RuleEngine::new();
    let rule_diags = rule_engine.analyze(source_file.syntax());

    rule_diags
        .into_iter()
        .map(|diag| {
            let severity = match diag.severity {
                Severity::Error => Severity::Error,
                Severity::Warning => Severity::Warning,
                Severity::Info => Severity::Info,
                Severity::Hint => Severity::Hint,
            };

            Diagnostic {
                code: Some(diag.rule_id.to_string()),
                message: diag.message,
                range: diag.range,
                severity,
                fix: diag.fix.map(|fix| {
                    crate::fix::Fix {
                        label: fix.label,
                        range: fix.range,
                        replacement: fix.replacement,
                    }
                }),
            }
        })
        .collect()
}
```

**Configuration:**

```toml
# .rust-analyzer.toml
[rust-analyzer.rules]
enabled = true
level = "intermediate"  # beginner, intermediate, advanced
severity = "warning"    # error, warning, info, hint

[rust-analyzer.rules.categories]
critical = true
high = true
medium = false
low = false
```

**Tasks:**

1. Design rule engine architecture
2. Implement top 20 rules as diagnostics
3. Add quick-fix suggestions
4. Create configuration system
5. Integrate with rust-analyzer diagnostics
6. Test on real projects

**Deliverables:**

- Rust-analyzer rule engine
- 20+ rule diagnostics with quick-fixes
- Configuration documentation

---

### 3.3 Open Standards for Sruja

**Objective:** Support export to open diagramming formats for portability.

**Export Commands:**

```bash
# Export to Mermaid.js
sruja export --format mermaid architecture.sruja > diagram.mmd

# Export to PlantUML
sruja export --format plantuml architecture.sruja > diagram.puml

# Export to Graphviz DOT
sruja export --format graphviz architecture.sruja > diagram.dot

# Import from existing diagrams
sruja import --source mermaid diagram.mmd > architecture.sruja
```

**Mermaid Export Implementation:**

````rust
// crates/sruja-cli/src/export/mermaid.rs
use crate::dsl::Architecture;

pub fn export_to_mermaid(arch: &Architecture) -> String {
    let mut output = String::new();

    output.push_str("```mermaid\n");
    output.push_str("graph TB\n");

    // Define external actors
    for person in &arch.people {
        output.push_str(&format!(
            "    {}[{}]\n",
            slugify(&person.name),
            person.name
        ));
    }

    // Define systems
    for system in &arch.systems {
        output.push_str(&format!(
            "    subgraph {} [{}]\n",
            slugify(&system.name),
            system.name
        ));

        // Define containers
        for container in &system.containers {
            output.push_str(&format!(
                "        {}[{}]\n",
                slugify(&container.name),
                container.name
            ));
        }

        // Define datastores
        for datastore in &system.datastores {
            output.push_str(&format!(
                "        {}((({})))\n",
            slugify(&datastore.name),
            datastore.name
        ));
        }

        output.push_str("    end\n");
    }

    // Define relationships
    for system in &arch.systems {
        for container in &system.containers {
            for rel in &container.relationships {
                output.push_str(&format!(
                    "    {} -->|{}| {}\n",
                    slugify(&rel.source),
                    rel.label,
                    slugify(&rel.target)
                ));
            }
        }
    }

    output.push_str("```\n");
    output
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "_")
        .replace('-', "_")
}
````

**PlantUML Export Implementation:**

```rust
// crates/sruja-cli/src/export/plantuml.rs
use crate::dsl::Architecture;

pub fn export_to_plantuml(arch: &Architecture) -> String {
    let mut output = String::new();

    output.push_str("@startuml\n");

    // Define stereotypes
    output.push_str("skinparam rectangle {\n");
    output.push_str("    BackgroundColor AliceBlue\n");
    output.push_str("    BorderColor Blue\n");
    output.push_str("    FontColor Black\n");
    output.push_str("}\n\n");

    output.push_str("skinparam database {\n");
    output.push_str("    BackgroundColor LightYellow\n");
    output.push_str("    BorderColor Orange\n");
    output.push_str("}\n\n");

    // Define actors
    for person in &arch.people {
        output.push_str(&format!("actor \"{}\" as {}\n", person.name, slugify(&person.name)));
    }

    // Define systems and containers
    for system in &arch.systems {
        output.push_str(&format!(
            "package \"{}\" {} {{\n",
            system.name,
            slugify(&system.name)
        ));

        for container in &system.containers {
            output.push_str(&format!(
                "    component \"{}\" as {} <<{}>>\n",
                container.name,
                slugify(&container.name),
                container.technology
            ));
        }

        for datastore in &system.datastores {
            output.push_str(&format!(
                "    database \"{}\" as {} <<{}>>\n",
                datastore.name,
                slugify(&datastore.name),
                datastore.technology
            ));
        }

        output.push_str("}\n");
    }

    // Define relationships
    for system in &arch.systems {
        for container in &system.containers {
            for rel in &container.relationships {
                output.push_str(&format!(
                    "{} -[{}]-> {}\n",
                    slugify(&rel.source),
                    rel.label,
                    slugify(&rel.target)
                ));
            }
        }
    }

    output.push_str("@enduml\n");
    output
}
```

**Import from Mermaid:**

```rust
// crates/sruja-cli/src/import/mermaid.rs
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/mermaid.pest"]
pub struct MermaidParser;

pub fn import_from_mermaid(input: &str) -> Result<Architecture> {
    let pairs = MermaidParser::parse(Rule::main, input)?;

    let mut architecture = Architecture {
        name: "Imported Architecture".to_string(),
        people: Vec::new(),
        systems: Vec::new(),
    };

    // Parse diagram
    for pair in pairs {
        match pair.as_rule() {
            Rule::node_definition => {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::actor_node => {
                        let name = extract_node_name(&inner);
                        architecture.people.push(Person {
                            name,
                            description: "Imported actor".to_string(),
                        });
                    }
                    Rule::component_node => {
                        let (name, technology) = extract_component_info(&inner);
                        // Add to appropriate system/container
                    }
                    _ => {}
                }
            }
            Rule::edge => {
                let (source, target, label) = extract_edge_info(&pair);
                // Add relationship
            }
            _ => {}
        }
    }

    Ok(architecture)
}
```

**Tasks:**

1. Design export format converters
2. Implement Mermaid.js export
3. Implement PlantUML export
4. Implement Graphviz DOT export
5. Create import parsers
6. Test bidirectional conversion
7. Document supported features

**Deliverables:**

- `sruja export` command
- `sruja import` command
- Format documentation
- Example conversions

---

## Phase 4: Advanced Features (Weeks 21-24)

### 4.1 Automated Rule Testing

**Objective:** Verify rules with generated code samples.

**Test Framework:**

```rust
// crates/skill-test/src/lib.rs
use std::path::PathBuf;
use tempfile::TempDir;

pub struct RuleTester {
    temp_dir: TempDir,
}

impl RuleTester {
    pub fn new() -> Result<Self> {
        Ok(Self {
            temp_dir: TempDir::new()?,
        })
    }

    pub fn test_rule(&self, rule: &Rule) -> TestResult {
        let good_code = &rule.examples.good;
        let bad_code = &rule.examples.bad;

        TestResult {
            rule_id: rule.id.clone(),
            good_compiles: self.test_compilation(good_code),
            bad_violates: self.test_violation(bad_code, rule),
            fix_works: self.test_fix(bad_code, rule),
        }
    }

    fn test_compilation(&self, code: &str) -> bool {
        let project_path = self.create_test_project(code);
        let result = std::process::Command::new("cargo")
            .args(["check", "--message-format=json"])
            .current_dir(&project_path)
            .output()
            .expect("Failed to run cargo check");

        result.status.success()
    }

    fn test_violation(&self, code: &str, rule: &Rule) -> bool {
        // Run rule checker on code
        let diagnostics = self.run_rule_checker(code);
        diagnostics.iter().any(|d| d.rule_id == rule.id)
    }

    fn test_fix(&self, code: &str, rule: &Rule) -> bool {
        // Apply fix suggestions
        let fixed_code = self.apply_fix(code, rule);
        // Verify fixed code compiles and passes rules
        self.test_compilation(&fixed_code)
            && !self.test_violation(&fixed_code, rule)
    }

    fn create_test_project(&self, code: &str) -> PathBuf {
        let project_dir = self.temp_dir.path().join("test-project");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create Cargo.toml
        let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

        // Create src directory
        let src_dir = project_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Write test code
        std::fs::write(src_dir.join("main.rs"), code).unwrap();

        project_dir
    }

    fn run_rule_checker(&self, code: &str) -> Vec<Diagnostic> {
        // Use skill-lint or rust-analyzer to check code
        // Return list of violations
        Vec::new()
    }

    fn apply_fix(&self, code: &str, rule: &Rule) -> String {
        // Apply rule's fix suggestions to code
        // Return fixed code
        code.to_string()
    }
}

#[derive(Debug)]
pub struct TestResult {
    pub rule_id: String,
    pub good_compiles: bool,
    pub bad_violates: bool,
    pub fix_works: bool,
}
```

**Test Suite Generator:**

```rust
// crates/skill-test/src/generator.rs
pub struct TestSuiteGenerator {
    rules: Vec<Rule>,
}

impl TestSuiteGenerator {
    pub fn generate_test_suite(&self) -> String {
        let mut output = String::new();

        output.push_str("#![cfg(test)]\n\n");
        output.push_str("use super::*;\n\n");

        for rule in &self.rules {
            output.push_str(&self.generate_rule_test(rule));
        }

        output
    }

    fn generate_rule_test(&self, rule: &Rule) -> String {
        format!(
            r#"
#[test]
fn test_{snake_case}() {{
    // Good example should pass
    let good_code = r#"
{good_code}
"#;
    assert!(check_rules(good_code).is_empty());

    // Bad example should violate
    let bad_code = r#"
{bad_code}
"#;
    let violations = check_rules(bad_code);
    assert!(!violations.is_empty(), "Expected violation for rule '{}'", "{rule_id}");

    // Fix should work
    let fixed_code = apply_fix(bad_code);
    assert!(check_rules(fixed_code).is_empty());
}}
"#,
            snake_case = rule.id.replace('-', "_"),
            good_code = rule.examples.good,
            bad_code = rule.examples.bad,
            rule_id = rule.id
        )
    }
}
```

**CI/CD Integration:**

```yaml
# .github/workflows/skill-tests.yml
name: Rule Tests

on:
  push:
    paths:
      - "skills/**"
      - "crates/skill-test/**"

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build skill-test
        run: cargo build --release --bin skill-test

      - name: Generate test suite
        run: skill-test generate --output tests/rules.rs

      - name: Run tests
        run: cargo test --test rules

      - name: Generate coverage report
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

**Tasks:**

1. Design test framework
2. Implement rule tester
3. Generate test suite from rules
4. Add CI/CD pipeline
5. Track test coverage
6. Fix failing rules

**Deliverables:**

- `skill-test` CLI tool
- Auto-generated test suite
- CI/CD test pipeline
- Coverage reports

---

### 4.2 Community Feedback System

**Objective:** Enable community contributions to rules.

**Issue Templates:**

```yaml
# .github/ISSUE_TEMPLATE/rule_update_request.md
---
name: Rule Update Request
about: Propose changes or additions to Rust Skills rules
title: '[RULE] Rule Update: <rule-id or new rule>'
labels: 'rule-update,triage'
assignees: ''
---

## Rule Information

**Rule ID:** (if updating existing) or **Proposed Rule ID:** (if new)

**Category:** (critical/high/medium/low/reference)

**Level:** (beginner/intermediate/advanced)

## Proposed Change

### Description
Describe the change you're proposing.

### Current State
(If updating existing rule)
Current rule content/issues:

### Proposed State
What should the rule say?

## Justification

### Evidence
- Links to Rust documentation
- Benchmarks or performance data
- Real-world examples
- Community consensus

### Use Cases
When does this rule apply?
When should it be broken?

### Impact
- Who will benefit from this change?
- What projects are affected?
- Migration complexity?

## Additional Context

Add any other context, screenshots, or code examples.

---

## Checklist

- [ ] I've searched for existing issues/PRs
- [ ] I've read the contributing guidelines
- [ ] I've tested this change locally
- [ ] I've provided evidence/justification
```

````yaml
# .github/ISSUE_TEMPLATE/rule_violation_report.md
---
name: Rule Violation Report
about: Report a case where a rule doesn't apply or should be broken
title: '[VIOLATION] Rule <rule-id> should be broken'
labels: 'rule-violation,triage'
assignees: ''
---

## Rule Information

**Rule ID:** `rule-id-here`

**Rule Title:** (from rule file)

## Violation Report

### Description
Describe the situation where this rule should be broken.

### Context
What's the project context?
- Project type: (web/cli/library/embedded/wasm)
- Performance requirements:
- Other constraints:

### Code Example
```rust
// Code where rule should be broken
````

### Why Break This Rule?

Explain why following this rule would be harmful in this context.

## Proposed Resolution

### Add Exception

Add this case to the rule's "When to Break" section.

### Change Rule

Modify the rule to accommodate this case.

### Remove Rule

The rule is incorrect and should be removed.

## Additional Context

Add any other relevant information.

---

## Checklist

- [ ] I've read the rule documentation
- [ ] I've tested both with and without the rule
- [ ] I can provide benchmarks if needed
- [ ] This is a real-world use case, not hypothetical

````

**PR Template:**

```yaml
# .github/PULL_REQUEST_TEMPLATE/rule_update.md
---
name: Rule Update
about: Submit changes to Rust Skills rules
---

## What does this PR do?

Briefly describe the changes.

## Type of Change

- [ ] Bug fix (rule is incorrect)
- [ ] New feature (add new rule)
- [ ] Update (improve existing rule)
- [ ] Documentation update
- [ ] Metadata update

## Rule Changes

### Affected Rules
- `rule-id-1`
- `rule-id-2`

### Changes Made
- Updated metadata: complexity/frequency/confidence
- Added trade-off context
- Improved examples
- Fixed broken links
- Other: ___

## Testing

- [ ] Ran `skill-lint check` locally
- [ ] Ran `skill-lint test` locally
- [ ] Tested code examples compile
- [ ] Updated related rules
- [ ] Checked for broken links

## Evidence

### Justification
- Link to documentation
- Benchmark results
- Real-world example
- Other evidence

### Examples
```rust
// Before
...
// After
...
````

## Breaking Changes

Will this change break existing projects or workflows?

- [ ] No breaking changes
- [ ] Breaking changes: describe

## Additional Context

Add any other context about the PR.

---

## Checklist

- [ ] I've followed contributing guidelines
- [ ] I've updated documentation
- [ ] I've tested changes locally
- [ ] I've run CI/CD tests
- [ ] I've added evidence/justification

````

**Automated Triage:**

```typescript
// .github/workflows/triage-rules.yml
name: Triage Rule Issues

on:
  issues:
    types: [opened, labeled]

jobs:
  categorize:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/github-script@v6
        with:
          script: |
            const issue = context.payload.issue;
            const title = issue.title;
            const body = issue.body;

            // Extract rule ID from title
            const ruleIdMatch = title.match(/\[(.*?)\]/);
            const ruleId = ruleIdMatch ? ruleIdMatch[1] : '';

            // Auto-label based on category
            if (title.includes('[RULE]')) {
              github.rest.issues.addLabels({
                issue_number: issue.number,
                owner: context.repo.owner,
                repo: context.repo.repo,
                labels: ['rule-update', 'triage-needed']
              });
            } else if (title.includes('[VIOLATION]')) {
              github.rest.issues.addLabels({
                issue_number: issue.number,
                owner: context.repo.owner,
                repo: context.repo.repo,
                labels: ['rule-violation', 'needs-evidence']
              });
            }

            // Comment with next steps
            github.rest.issues.createComment({
              issue_number: issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `Thank you for your contribution!

A maintainer will review this soon. In the meantime, please ensure:
- You've read the contributing guidelines
- You've provided evidence/justification
- You've tested changes locally

Reference rule: \`${ruleId}\``
            });
````

**Tasks:**

1. Create issue templates
2. Create PR template
3. Set up automated triage
4. Add contribution guidelines
5. Create review checklist
6. Document community process

**Deliverables:**

- Issue templates (update, violation, new rule)
- PR template
- Automated trige workflow
- Updated CONTRIBUTING.md

---

### 4.3 Metrics Dashboard

**Objective:** Track rule effectiveness and usage patterns.

**Dashboard Architecture:**

```typescript
// src/dashboard/components/Stats.tsx
import React from 'react';
import { useQuery } from '@tanstack/react-query';

export function RuleStats() {
    const { data: stats } = useQuery({
        queryKey: ['rule-stats'],
        queryFn: async () => {
            const response = await fetch('/api/stats');
            return response.json();
        }
    });

    return (
        <div className="grid grid-cols-4 gap-4">
            <StatCard
                title="Total Rules"
                value={stats?.totalRules || 0}
                change="+5%"
                trend="up"
            />
            <StatCard
                title="Active Users"
                value={stats?.activeUsers || 0}
                change="+12%"
                trend="up"
            />
            <StatCard
                title="Violations Fixed"
                value={stats?.violationsFixed || 0}
                change="+8%"
                trend="up"
            />
            <StatCard
                title="Avg Context Usage"
                value={`${stats?.avgContextSize || 0} tokens`}
                change="-15%"
                trend="down"
            />
        </div>
    );
}

function StatCard({ title, value, change, trend }: StatCardProps) {
    const trendColor = trend === 'up' ? 'text-green-500' : 'text-red-500';
    const trendIcon = trend === 'up' ? '↑' : '↓';

    return (
        <div className="bg-white rounded-lg p-6 shadow">
            <h3 className="text-gray-600 text-sm font-medium">{title}</h3>
            <div className="mt-2 flex items-baseline">
                <span className="text-2xl font-bold text-gray-900">{value}</span>
                <span className={`ml-2 text-sm ${trendColor}`}>
                    {trendIcon} {change}
                </span>
            </div>
        </div>
    );
}
```

**API Endpoints:**

```rust
// crates/metrics-api/src/routes.rs
use axum::{Json, Router};
use serde_json::json;

pub fn create_router() -> Router {
    Router::new()
        .route("/api/stats", get(get_stats))
        .route("/api/rules/popular", get(get_popular_rules))
        .route("/api/violations/trends", get(get_violation_trends))
        .route("/api/projects/analysis", get(get_project_analysis))
}

async fn get_stats() -> Json<serde_json::Value> {
    let stats = json!({
        "totalRules": 179,
        "activeUsers": 1247,
        "violationsFixed": 8934,
        "avgContextSize": 125, // tokens
        "ruleAdoptionRate": 0.87
    });

    Json(stats)
}

async fn get_popular_rules() -> Json<Vec<RuleStats>> {
    let rules = vec![
        RuleStats {
            id: "async-no-lock-await".to_string(),
            title: "Never hold locks across await".to_string(),
            usage_count: 2341,
            effectiveness: 0.95,
        },
        RuleStats {
            id: "err-result-over-panic".to_string(),
            title: "Return Result, don't panic".to_string(),
            usage_count: 2156,
            effectiveness: 0.93,
        },
    ];

    Json(rules)
}

#[derive(Debug, Serialize)]
struct RuleStats {
    id: String,
    title: String,
    usage_count: u64,
    effectiveness: f64,
}
```

**Data Collection:**

```rust
// crates/metrics-collector/src/collector.rs
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub struct MetricsCollector {
    events: Vec<RuleEvent>,
}

impl MetricsCollector {
    pub fn track_rule_use(&mut self, rule_id: &str, context: &ProjectContext) {
        self.events.push(RuleEvent {
            timestamp: Utc::now(),
            event_type: EventType::RuleApplied,
            rule_id: rule_id.to_string(),
            context: context.clone(),
        });
    }

    pub fn track_violation(&mut self, rule_id: &str, location: &str) {
        self.events.push(RuleEvent {
            timestamp: Utc::now(),
            event_type: EventType::ViolationDetected,
            rule_id: rule_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn generate_report(&self, time_range: TimeRange) -> MetricsReport {
        let filtered: Vec<_> = self.events
            .iter()
            .filter(|e| e.timestamp >= time_range.start && e.timestamp <= time_range.end)
            .collect();

        let rule_usage: HashMap<String, u64> = filtered
            .iter()
            .filter(|e| matches!(e.event_type, EventType::RuleApplied))
            .fold(HashMap::new(), |mut acc, e| {
                *acc.entry(e.rule_id.clone()).or_insert(0) += 1;
                acc
            });

        let violations: HashMap<String, u64> = filtered
            .iter()
            .filter(|e| matches!(e.event_type, EventType::ViolationDetected))
            .fold(HashMap::new(), |mut acc, e| {
                *acc.entry(e.rule_id.clone()).or_insert(0) += 1;
                acc
            });

        MetricsReport {
            total_events: filtered.len() as u64,
            rules_applied: rule_usage,
            violations_detected: violations,
            time_range,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub rule_id: String,
    pub context: ProjectContext,
    pub location: String,
}

#[derive(Debug, Clone)]
pub enum EventType {
    RuleSuggested,
    RuleApplied,
    ViolationDetected,
    ViolationFixed,
    RuleIgnored,
}

#[derive(Debug)]
pub struct MetricsReport {
    pub total_events: u64,
    pub rules_applied: HashMap<String, u64>,
    pub violations_detected: HashMap<String, u64>,
    pub time_range: TimeRange,
}
```

**Visualization Components:**

```typescript
// src/dashboard/components/RuleEffectivenessChart.tsx
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend } from 'recharts';

export function RuleEffectivenessChart({ ruleId }: { ruleId: string }) {
    const { data } = useQuery({
        queryKey: ['rule-effectiveness', ruleId],
        queryFn: async () => {
            const response = await fetch(`/api/rules/${ruleId}/effectiveness`);
            return response.json();
        }
    });

    return (
        <div className="bg-white rounded-lg p-6 shadow">
            <h3 className="text-lg font-medium mb-4">Rule Effectiveness Over Time</h3>
            <LineChart width={600} height={300} data={data}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="date" />
                <YAxis />
                <Tooltip />
                <Legend />
                <Line
                    type="monotone"
                    dataKey="applied"
                    stroke="#8884d8"
                    name="Applied"
                />
                <Line
                    type="monotone"
                    dataKey="violations"
                    stroke="#82ca9d"
                    name="Violations"
                />
            </LineChart>
        </div>
    );
}
```

**Tasks:**

1. Design metrics schema
2. Implement metrics collector
3. Build API endpoints
4. Create dashboard UI
5. Add data visualizations
6. Deploy to production
7. Set up monitoring/alerts

**Deliverables:**

- Metrics collector service
- REST API for metrics
- Dashboard web application
- Data visualization components
- Monitoring alerts

---

## Implementation Roadmap

```
Week 1-2:  ✅ Design metadata schema
Week 3-4:  ✅ Implement selective loading CLI
Week 5-6:  ✅ Add trade-off context to top 50 rules
Week 7-8:  ✅ Build skill-lint CLI tool
Week 9-10: ✅ Set up CI/CD validation pipeline
Week 11-12: ✅ Create dynamic rule analyzer
Week 13-14: ✅ Build VSCode extension MVP
Week 15-16: ✅ Add Mermaid/PlantUML export
Week 17-18: ✅ Integrate with rust-analyzer
Week 19-20: ✅ Build automated testing framework
Week 21-22: ✅ Create community feedback system
Week 23-24: ✅ Build metrics dashboard
```

---

## Resource Requirements

### Personnel

- **2-3 Full-time developers** (Rust + TypeScript)
- **1 Part-time designer** (UI/UX for dashboard)
- **1 Project manager** (quarterly involvement)

### Infrastructure

- **CI/CD:** GitHub Actions (included in repo)
- **Database:** Postgres for metrics (can use Heroku free tier)
- **Hosting:** Vercel/Netlify for dashboard (free tier)
- **Analytics:** Plausible or similar privacy-focused analytics

### Budget Estimate

| Phase     | Development | Infrastructure | Total       |
| --------- | ----------- | -------------- | ----------- |
| Phase 1   | $20K        | $0             | $20K        |
| Phase 2   | $30K        | $500           | $30.5K      |
| Phase 3   | $40K        | $1K            | $41K        |
| Phase 4   | $20K        | $1K            | $21K        |
| **Total** | **$110K**   | **$2.5K**      | **$112.5K** |

---

## Success Metrics

### Quantitative

- **Context Reduction:** 70% reduction in AI context usage (336 → 100 lines)
- **Rule Relevance:** 80%+ relevance in suggested rules (measured by user adoption)
- **Onboarding Speed:** 50% faster onboarding for new Rust developers
- **Validation Rate:** 90%+ skill file validation rate in CI
- **Community Engagement:** 50+ community contributions/month (by month 6)

### Qualitative

- **Developer Satisfaction:** Survey score > 4/5
- **Rule Quality:** Low bug report rate (< 5% of rules have issues)
- **Documentation Quality:** Clear examples, minimal confusion
- **Tooling UX:** Intuitive CLI, helpful IDE integration

---

## Risk Mitigation

### Technical Risks

| Risk                                      | Impact | Mitigation                                              |
| ----------------------------------------- | ------ | ------------------------------------------------------- |
| Context window still too large            | High   | Prioritize top 50 rules, implement aggressive filtering |
| False positives in rule violations        | Medium | User feedback loop, adjustable severity levels          |
| Performance overhead in IDE               | Medium | Cache aggressively, optimize rule engine                |
| Integration complexity with rust-analyzer | High   | Start with standalone tooling, gradually integrate      |

### Adoption Risks

| Risk                          | Impact | Mitigation                                                       |
| ----------------------------- | ------ | ---------------------------------------------------------------- |
| Low community engagement      | Medium | Early outreach, showcase benefits, simplify contribution process |
| Learning curve for developers | Medium | Clear documentation, beginner-friendly defaults                  |
| Tool fragmentation            | Low    | Standardize on single CLI, integrate with popular tools          |

---

## Next Steps

### Immediate Actions (Week 1)

1. **Approve plan and allocate budget**
2. **Set up project board** (GitHub Projects)
3. **Create development environment**
4. **Begin Phase 1, Task 1.1** (metadata schema)

### Decision Points

- **Month 2:** Review Phase 1 progress, adjust timeline if needed
- **Month 4:** Evaluate Phase 2 completion, plan Phase 3
- **Month 6:** Final review, decide on ongoing maintenance

---

## Appendix: References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [C4 Model](https://c4model.com/)
- [Mermaid.js](https://mermaid-js.github.io/)
- [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer)

---

**Document Version:** 1.0
**Last Updated:** 2025-02-08
**Author:** Sruja Team
**Status:** Draft - Pending Review
