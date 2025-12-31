# Developer Tools

This directory contains tools for Sruja developers to improve and maintain the codebase.

## Diagram Quality Tools

Located in `diagram-quality/`, these tools help developers improve the layout quality of generated architecture diagrams.

### Available Tools

#### `improve-diagram-quality.sh`

Bash script for iterative diagram quality improvement.

**Purpose**: Automate the process of testing and improving diagram layouts across all example files.

**Usage**:

```bash
cd /Users/dilipkola/Workspace/sruja
./dev-tools/diagram-quality/improve-diagram-quality.sh
```

**What it does**:

1. Runs E2E tests to measure diagram quality metrics
2. Analyzes results and identifies quality issues
3. Suggests improvements to layout algorithms
4. Tracks progress over iterations

**When to use**:

- After modifying DOT generation logic in `pkg/export/dot/`
- When adding new Graphviz constraints
- To verify layout quality hasn't regressed
- During development of layout improvements

#### `improve-iteratively.ts`

TypeScript automation for diagram quality improvement.

**Purpose**: Provides programmatic access to quality measurement and improvement workflows.

**Usage**:

```bash
cd /Users/dilipkola/Workspace/sruja/apps/designer
npx tsx ../../dev-tools/diagram-quality/improve-iteratively.ts
```

**What it does**:

1. Loads all example diagrams in browser
2. Extracts quality metrics from rendering
3. Tracks quality history over time
4. Generates improvement reports

**When to use**:

- For automated quality regression testing
- To generate quality reports for all examples
- During CI/CD quality checks (future)
- For data-driven layout optimization

### Quality Metrics Explained

The quality measurement system evaluates diagrams on several dimensions:

**Critical Issues** (must be zero):

- **Node overlaps**: Nodes rendering on top of each other
- **Label overlaps**: Edge labels obscuring nodes or other labels

**Layout Quality** (higher is better):

- **Edge crossings**: Fewer crossings = clearer diagram
- **Rank alignment**: How well nodes align horizontally
- **Cluster balance**: Distribution of nodes within containers
- **Spacing consistency**: Uniform spacing between elements

**Overall Score**:

- A-grade: > 90% quality (excellent)
- B-grade: 80-90% quality (good)
- C-grade: 70-80% quality (acceptable)
- D-grade: 60-70% quality (needs improvement)
- F-grade: < 60% quality (poor)

### Development Workflow

1. **Make layout changes**:

   ```bash
   # Edit layout logic
   vim pkg/export/dot/builder.go
   ```

2. **Rebuild WASM**:

   ```bash
   make build-wasm
   ```

3. **Run quality tests**:

   ```bash
   ./dev-tools/diagram-quality/improve-diagram-quality.sh
   ```

4. **Review results**:
   - Check console output for quality scores
   - Identify diagrams with quality regressions
   - Adjust constraints and re-test

5. **Iterate**:
   - Repeat until quality targets are met across all examples

### Quality Metrics in Development

Quality metrics are **dev-only** features:

- Visible in development environment (`import.meta.env.DEV`)
- Hidden in production builds (tree-shaken out)
- Exposed to `window.__DIAGRAM_QUALITY__` for testing

**In the UI**:

- Quality score card shows in bottom-left (dev only)
- Click to expand/collapse detailed metrics
- Color-coded indicators: green (good), yellow (warning), red (poor)

### Tips for Improving Layout Quality

**Reduce edge crossings**:

- Adjust node ranking constraints
- Use `weight` attribute to prioritize important edges
- Consider hub node optimization for high-degree nodes

**Fix node overlaps**:

- Increase `nodesep` and `ranksep` values
- Review cluster sizing constraints
- Check for over-constrained layouts

**Improve alignment**:

- Use same rank constraints for related nodes
- Align cluster boundaries
- Consider using invisible edges for alignment

**Balance clusters**:

- Adjust cluster padding
- Review subgraph constraints
- Consider splitting large clusters

### E2E Test Integration

The quality tests are integrated with Playwright:

```typescript
// apps/designer/tests/diagram-quality-iterative.spec.ts
test("measure diagram quality", async ({ page }) => {
  // Load diagram
  await page.goto("/?example=demo.sruja");

  // Extract quality metrics
  const quality = await page.evaluate(() => window.__DIAGRAM_QUALITY__);

  // Assert quality targets
  expect(quality.score).toBeGreaterThan(0.8);
});
```

### Future Improvements

- [ ] CI/CD integration for automated quality checks
- [ ] Quality regression detection in PRs
- [ ] Historical quality tracking database
- [ ] Automated constraint optimization
- [ ] Machine learning for layout parameter tuning

---

## Contributing

When adding new developer tools:

1. Place them in appropriate subdirectory under `dev-tools/`
2. Document usage in this README
3. Add examples and workflows
4. Mark clearly as dev-only tools
5. Update `CONTRIBUTING.md` if needed
