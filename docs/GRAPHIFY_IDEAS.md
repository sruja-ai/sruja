# Ideas from Graphify Relevant to Sruja

Source: https://github.com/safishamsi/graphify

Graphify is a Claude Code skill that turns any folder of code, docs, papers, and images into a queryable knowledge graph with community detection, audit trails, and multiple export formats.

Below are ideas from graphify that could strengthen Sruja's architecture-as-code and context engineering capabilities.

---

## 1. Incremental/Watch Mode with SHA256 Caching

**Graphify approach:** Uses SHA256 content hashing to detect changed files. Only re-extracts modified files on `--update`, merging results into the existing graph. Also offers `--watch` for live auto-rebuild on file changes.

**Relevance to Sruja:** Sruja already has `sruja watch -r .` but could benefit from:
- Content-hash-based caching for `discover` scans (skip unchanged files)
- Faster incremental drift detection by only re-scanning modules whose source changed
- A manifest file (like graphify's) that records the last-known hash of each scanned file

**Implementation notes:**
- Store `.sruja/scan-manifest.json` with `{path: sha256_hash}` entries
- On `sruja discover` or `sruja drift`, compare hashes before re-parsing
- Report "X files unchanged, Y files re-scanned" for transparency

---

## 2. Confidence Labels on Edges (EXTRACTED / INFERRED / AMBIGUOUS)

**Graphify approach:** Every relationship edge is tagged with a confidence level:
- `EXTRACTED` — explicitly stated in source (import, call, citation)
- `INFERRED` — reasonable deduction (shared data structure, implicit dependency)
- `AMBIGUOUS` — uncertain, flagged for human review

**Relevance to Sruja:** Sruja's `discover` command infers architecture from code. Currently the output doesn't distinguish between high-confidence structural relationships (explicit imports) and lower-confidence inferences (implied dependencies). Adding confidence tags would:
- Let users filter the architecture graph by reliability
- Enable `sruja lint` to warn about AMBIGUOUS relationships
- Help AI agents know which edges to trust vs question

**Implementation notes:**
- Add an optional `confidence` field to relationship syntax: `A -> B "uses" { confidence "inferred" }`
- `sruja discover` can tag each generated relationship with its confidence level
- `sruja lint` could warn on unresolved AMBIGUOUS edges

---

## 3. God Nodes / Surprising Connections Analysis

**Graphify approach:** After building the graph, graphify identifies:
- **God nodes** — highest-degree concepts that everything connects through
- **Surprising connections** — edges that cross community boundaries, ranked by a composite score
- **Suggested questions** — questions the graph is uniquely positioned to answer

**Relevance to Sruja:** Sruja's `impact` command already does blast-radius analysis. But it could add:
- A "god components" report identifying highly-coupled components (architectural smell)
- Cross-boundary edge detection (when a component in one layer unexpectedly connects to another)
- Auto-generated architectural questions for review ("Why does the auth module depend on the billing service?")

**Implementation notes:**
- Compute degree centrality and betweenness centrality on the architecture graph
- Flag components with degree > 2σ above mean as "god nodes" / coupling hotspots
- Cross-layer edges already violate boundary rules — surfacing them as "surprising connections" adds human-readable context

---

## 4. Multiple Export Formats (Obsidian, GraphML, Neo4j, SVG)

**Graphify approach:** Exports to many formats:
- Interactive HTML (vis.js)
- Obsidian vault (one note per node with wikilinks)
- GraphML (for Gephi/yEd)
- Cypher (for Neo4j)
- SVG (for embedding in docs)
- MCP server (for live agent queries)

**Relevance to Sruja:** Sruja currently exports to Mermaid, Markdown, and JSON. Additional formats to consider:
- **Obsidian vault export** — one `.md` file per component with wikilinks to related components. Architects can explore the architecture in Obsidian's graph view.
- **GraphML export** — for teams using Gephi or yEd for advanced graph analysis
- **D2 export** — D2 is a popular diagram language; export would broaden adoption
- **Cypher/Neo4j export** — for enterprise teams using graph databases for architecture governance

**Implementation notes:**
- Most of these are straightforward serialization from the existing internal graph representation
- Obsidian export is particularly compelling since architects already use Obsidian for knowledge management

---

## 5. Token Reduction Benchmark

**Graphify approach:** Automatically benchmarks token reduction — comparing "read all raw files" vs "query the structured graph". Reports the compression ratio (e.g., 71.5x for large corpora).

**Relevance to Sruja:** Sruja's value proposition includes providing AI editors with high-quality context. Quantifying this would be powerful:
- "Sruja context is X tokens vs Y tokens reading raw source" 
- Ties directly into `sruja context-score` which already rates AI-readiness

**Implementation notes:**
- After `sruja context`, compute: tokens(raw source files) vs tokens(structured context output)
- Report this as part of `sruja context-score` or `sruja doctor`
- Useful marketing metric and a concrete way for users to measure ROI

---

## 6. Community Detection / Clustering

**Graphify approach:** Uses Leiden algorithm (via graspologic) to detect communities in the graph, then labels each community with human-readable names.

**Relevance to Sruja:** Sruja already has explicit architectural boundaries (layers, systems, containers). But automatic community detection could:
- Validate that declared architecture matches actual code coupling
- Discover undeclared sub-modules within large containers
- Power a "suggested architecture" feature that infers boundaries from code before the user declares them

**Implementation notes:**
- Run community detection on the discovered code graph
- Compare detected communities to declared architecture boundaries
- Report misalignments as "potential architectural drift" or "suggested boundary"
- This complements the existing drift detection with a bottom-up perspective

---

## 7. Wiki Generation for Agent Navigation

**Graphify approach:** Generates a Wikipedia-style markdown wiki with an `index.md` entry point. Agents navigate by reading files instead of parsing JSON — each community gets an article.

**Relevance to Sruja:** The `sruja onboard` command already generates onboarding content. A wiki-style output could:
- Generate a browsable architecture wiki (one page per system/container/component)
- Provide an `index.md` that any AI agent can follow to understand the codebase
- Work as a static site or Obsidian vault without requiring the CLI at runtime

**Implementation notes:**
- Generate `architecture-wiki/index.md` with links to per-component pages
- Each page includes: description, technology, relationships, layer, and relevant code paths
- This becomes a zero-runtime-dependency way to share architecture context

---

## 8. Cumulative Cost Tracking

**Graphify approach:** Tracks token usage across runs in `graphify-out/cost.json`, accumulating input/output tokens per run with timestamps.

**Relevance to Sruja:** For teams using `--enrich` (LLM enrichment), tracking cumulative LLM costs would be valuable:
- Shows cost of maintaining architecture context over time
- Enables budgeting and cost-aware CI decisions
- Useful for enterprise procurement justification

**Implementation notes:**
- Store in `.sruja/usage.json`: `{runs: [{date, input_tokens, output_tokens, command}], totals: {...}}`
- Only populated when LLM features are used (enrich, ai command)

---

## Priority Ranking

| # | Idea | Impact | Effort | Priority |
|---|------|--------|--------|----------|
| 1 | Incremental scan caching | High (perf) | Medium | **P1** |
| 2 | Confidence labels on edges | High (trust) | Medium | **P1** |
| 3 | God nodes / coupling analysis | High (insight) | Low | **P1** |
| 5 | Token reduction benchmark | Medium (metrics) | Low | **P2** |
| 6 | Community detection validation | Medium (drift) | Medium | **P2** |
| 4 | Additional export formats | Medium (adoption) | Medium | **P2** |
| 7 | Wiki generation | Medium (context) | Medium | **P3** |
| 8 | Cost tracking | Low (ops) | Low | **P3** |

---

## Next Steps

1. Discuss with the team which ideas align with the current roadmap
2. For P1 items, draft ADRs before implementation
3. Consider whether confidence labels should be a DSL-level feature or metadata-only
4. Evaluate whether community detection complements or conflicts with the existing layer/boundary model
