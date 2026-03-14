# Known Limitations

This document outlines the known limitations of Sruja's architecture analysis capabilities. Understanding these limitations helps you interpret results accurately and set appropriate expectations.

---

## Scanner Limitations

### Dynamic Imports and Reflection

**Issue:** Sruja may miss dependencies that are loaded dynamically at runtime.

**Examples:**
- `require()` calls with dynamic paths (e.g., `require('./' + module + '.js')`)
- `import()` expressions with variable paths
- Dependency injection frameworks that configure dependencies at runtime
- Plugin systems that load modules dynamically

**Impact:** Some dependencies may not appear in the architecture graph, leading to incomplete analysis.

**Mitigation:** Review your application's runtime loading patterns manually. Consider adding explicit documentation for dynamic dependencies in your `.sruja` file.

---

### Orphan False Positives

**Issue:** Components may be flagged as "orphans" (no incoming dependencies) when they are valid entry points or utility modules.

**Common Causes:**
- Main application entry points (e.g., `main.rs`, `index.js`, `app.py`)
- Test files and test utilities
- Configuration files and initialization code
- Event handlers and callback functions
- Library exports that are consumed by external applications

**Impact:** High number of orphan findings that are not architectural problems.

**Mitigation:**
- Review orphan findings manually before taking action
- Mark expected entry points in your `.sruja` file if appropriate
- Use `sruja lint` and `sruja drift` in combination to identify actionable issues

---

### Language-Specific Limitations

#### Kotlin

**Issue:** Kotlin uses minimal line-based extraction instead of full Tree-sitter AST parsing.

**Root Cause:** Tree-sitter Kotlin 0.3 depends on Tree-sitter 0.20, but Sruja uses Tree-sitter 0.24.

**Impact:**
- Only imports and class definitions are extracted
- No precise dependency analysis for nested structures
- May miss relationships in complex Kotlin codebases

**Mitigation:** For Kotlin projects, supplement Sruja analysis with manual review of key architectural patterns.

---

### Framework-Specific Limitations

#### Web Frameworks

**Issue:** Some web frameworks use routing conventions that may not be fully captured.

**Examples:**
- File-based routing (Next.js, Remix)
- Annotation-based routing (Spring Boot, NestJS)
- Dynamic route generation

**Impact:** Some API endpoints or service boundaries may be missed.

**Mitigation:** Consider defining explicit service boundaries in your `.sruja` file for complex routing scenarios.

---

## Analysis Limitations

### Semantic Layer Weakness Without ADRs

**Issue:** The "why" and semantic analysis features provide limited value without rich Architecture Decision Records (ADRs) or intent documentation.

**Impact:**
- Answers to "why are we using X?" may be generic or low-confidence
- Semantic analysis may not detect the intended purpose of components
- Context exports may lack business rationale

**Mitigation:**
- Maintain ADRs in your repository (e.g., in `docs/adr/` or `docs/architecture/`)
- Use `sruja intent check` to validate alignment between documented intent and code
- For code-only repos, rely on structural analysis (`quickstart`, `drift`) rather than semantic features

---

### False Positives in Large Codebases

**Issue:** Large repositories may generate numerous findings, including some that are noise rather than actionable insights.

**Common Noise Sources:**
- Legacy code that has technical debt but is not currently problematic
- Third-party library dependencies
- Generated code or boilerplate
- Test utilities and mocks

**Impact:** Reviewing findings can be time-consuming; signal-to-noise ratio may decrease.

**Mitigation:**
- Focus on high-severity findings first (Error level violations)
- Use `sruja drift --fail-on cycles,layer-violations` to focus on critical issues
- Create baselines and use `sruja drift-pr` to detect only new violations

---

## CI/CD Limitations

### First-Run False Positives

**Issue:** When first adding Sruja to CI/CD, existing architectural violations will fail the pipeline.

**Impact:** CI may block merges until all violations are addressed, which may not be practical for large existing codebases.

**Mitigation:**
- Use `sruja drift-pr` in CI to detect only NEW violations introduced in a PR
- Gradually address existing violations outside of CI
- Configure `sruja drift --fail-on` to fail only on specific violation types

---

## Scope and Coverage

### Not a Replacement for Design Review

**Issue:** Sruja is a tool for structural analysis and drift detection, not a complete replacement for architectural design review.

**Limitations:**
- Does not evaluate non-functional requirements (performance, security, scalability)
- Does not assess business logic correctness
- Does not replace human judgment in architectural decisions

**Best Use:** Sruja should be used as a complement to design review, providing data-driven insights to support human decision-making.

---

### Intent-Dependent Features

**Issue:** Some features require well-documented architectural intent to be useful.

**Features Affected:**
- `sruja intent check`
- `sruja compliance`
- Semantic "why" questions

**Mitigation:** For repos without intent documentation, focus on:
- `sruja quickstart` for structural overview
- `sruja drift` for detecting structural problems
- `sruja lint` for validating architecture-as-code files

---

## Reporting and Feedback

If you encounter limitations not documented here, please:

1. **Check the documentation:** Review [docs/](docs/) for more detailed information
2. **Search GitHub Issues:** Check if the limitation has been reported at [github.com/sruja-ai/sruja/issues](https://github.com/sruja-ai/sruja/issues)
3. **Report the issue:** If it's a new limitation, please file an issue with details about your use case

We continuously improve Sruja based on real-world feedback. Your reports help us prioritize enhancements.
