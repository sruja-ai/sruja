# LLM Cost Optimization for Sruja — Research Report (June 2026)

Practical, implementable techniques for reducing LLM costs in a deterministic architecture scanner that optionally uses LLMs for narrative enrichment, semantic search, and entity resolution.

---

## Table of Contents

1. [Prompt Caching](#1-prompt-caching)
2. [Structured Output / Guided Generation](#2-structured-output--guided-generation)
3. [Small Model Routing](#3-small-model-routing)
4. [RAG vs Full Context](#4-rag-vs-full-context)
5. [Embedding-First Approaches](#5-embedding-first-approaches)
6. [Local/Small Models for Specific Tasks](#6-localsmall-models-for-specific-tasks)
7. [Token Budgeting Techniques](#7-token-budgeting-techniques)
8. [Batching and Async](#8-batching-and-async)
9. [Cost Per Task Estimates](#9-cost-per-task-estimates)

---

## 1. Prompt Caching

### How Each Provider Handles It

| Feature | OpenAI | Anthropic | Google Gemini |
|---|---|---|---|
| **Activation** | Automatic (no code changes) | Manual via `cache_control` param | Manual via `CachedContent.create` |
| **Cost savings** | 50% off on cached input tokens | 90% off on cache reads | 75% off on cache reads |
| **Cache write cost** | No surcharge | 25% surcharge on first write | Standard input cost + storage |
| **TTL** | 5–10 min (up to 1 hr off-peak) | 5 min, refreshed on each use | Default 1 hr, customizable |
| **Min prompt length** | 1,024 tokens | 1,024–2,048 tokens | 32,768 tokens |
| **Max breakpoints** | Automatic prefix matching | 4 manual breakpoints | 1 per cache object |
| **Monitoring** | `prompt_tokens_details.cached_tokens` | `cache_read_input_tokens` | `usage_metadata` |

### Cache Hit Rates in Practice

- Production systems report **91–95% cache hit rates** across Claude models (Reddit/r/Anthropic)
- One team went from **$720/month → $72/month** (90% reduction) using Anthropic prompt caching
- Another reported **70% cost reduction** with consistent prompt structure

### Structure for Maximum Cache Hits

```
[STATIC — cached prefix]
├── System instructions (architecture schema, output format)
├── Tool definitions / function schemas
├── Few-shot examples
└── Large reference documents (repo.sruja, classification.json)
###
[DYNAMIC — not cached]
├── User query / current file context
└── Conversation history (recent turns only)
```

**Critical rules:**
- Prefix must match **exactly** — whitespace, line breaks, casing all matter
- Static content at the top, dynamic at the bottom
- Use consistent delimiters between sections
- For Anthropic: place `cache_control` on the largest static blocks (tools, system prompt)

### Sruja-Specific Recommendation

Sruja's MCP tools and focus briefings have a large static prefix (architecture schema, tool definitions, classification data). Cache the:
- System prompt with architecture rules (~2–4k tokens)
- Tool/function definitions (~1–2k tokens)
- Classification JSON or repo.sruja content (~5–20k tokens)

With Anthropic's 90% discount on reads, this turns a $0.05/call input cost into $0.005/call for the cached portion.

**For local models (llama.cpp, vLLM):** KV-cache reuse works similarly but is session-scoped. vLLM supports automatic prefix caching with `--enable-prefix-caching`.

---

## 2. Structured Output / Guided Generation

### How It Reduces Token Usage

Structured output constrains the model's output at the **token generation level**, not after the fact. This means:

1. **No wasted tokens** on markdown formatting, explanations, or hedging language
2. **No retry loops** when the model produces invalid JSON (typical failure rate: 5–15% without constraints)
3. **Shorter outputs by design** — schemas enforce minimal field presence

### Practical Token Savings

| Approach | Avg output tokens | Failure rate | Effective cost |
|---|---|---|---|
| Free-form text + post-parse | 200–500 | 10–15% need retry | 1.1–1.15x baseline |
| JSON mode (basic) | 150–300 | 2–5% | 1.02–1.05x baseline |
| JSON Schema (strict) | 80–150 | ~0% | 0.4–0.6x baseline |
| Grammar-constrained | 50–120 | ~0% | 0.25–0.5x baseline |

Grammar-constrained decoding can **cut output tokens by 50–75%** compared to free-form generation, because the model only generates valid tokens within the grammar.

### Libraries and Tools

| Library | How It Works | Best For |
|---|---|---|
| **OpenAI Structured Outputs** | JSON Schema enforcement at API level | Simplest integration; GPT models |
| **Anthropic tool_use** | Forces JSON via tool calling schema | Claude models |
| **outlines** (dottxt-ai/outlines) | FSM-based constrained generation; supports regex, JSON Schema, CFGs | Local models; maximum control |
| **lm-format-enforcer** | Token-level filtering with JSON/regex/grammar support | vLLM, Text Generation Inference |
| **guidance** (guidance-ai/guidance) | Template-based generation with interleaved control | Complex multi-step structured output |
| **vLLM structured output** | Built-in support for JSON Schema, regex, grammar via backend | Production local inference |

### Sruja-Specific Recommendation

For entity resolution and classification outputs, use **JSON Schema with strict mode**:

```json
{
  "type": "object",
  "properties": {
    "matches": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "source_id": { "type": "string" },
          "target_id": { "type": "string" },
          "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
          "match_type": { "type": "string", "enum": ["exact", "fuzzy", "semantic"] }
        },
        "required": ["source_id", "target_id", "confidence", "match_type"]
      },
      "maxItems": 10
    }
  }
}
```

This eliminates retries and keeps outputs to ~50–100 tokens vs 300+ for free-form.

---

## 3. Small Model Routing

### When Small Models Are "Good Enough"

Based on production benchmarks and the inference.net pricing analysis:

| Task | Small Model Accuracy | Large Model Accuracy | Cost Ratio | Recommendation |
|---|---|---|---|---|
| **Text classification** (bug/feature/question) | 92–95% (Haiku/Flash) | 96–98% (Sonnet/GPT-5) | 8–12x cheaper | Use small |
| **Entity extraction** (structured fields) | 88–93% | 94–97% | 8–12x cheaper | Use small with validation |
| **Sentiment/scoring** | 90–94% | 93–96% | 8–12x cheaper | Use small |
| **Summarization** (short, factual) | 85–90% | 92–96% | 8–12x cheaper | Use small for drafts |
| **Code analysis** (simple patterns) | 80–85% | 92–95% | 8–12x cheaper | Use small for triage |
| **Complex reasoning** | 60–75% | 90–95% | — | Use large |
| **Narrative generation** (prose quality) | 70–80% | 90–95% | — | Use large for final, small for drafts |
| **Multi-hop reasoning** | 55–70% | 85–93% | — | Use large |

### Routing Strategy

```
                    ┌─────────────────┐
                    │ Incoming request │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Task classifier │  ← rule-based or embedding similarity
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼───────┐     │     ┌────────▼───────┐
     │  Simple/routine │     │     │  Complex/reasoning │
     │  (classification│     │     │  (narrative, multi- │
     │   extraction)   │     │     │   hop, creative)    │
     └────────┬───────┘     │     └────────┬───────┘
              │              │              │
     ┌────────▼───────┐     │     ┌────────▼───────┐
     │ Haiku / Flash / │     │     │ Sonnet / GPT-5 │
     │ Llama-4-Scout   │     │     │ Claude Opus    │
     │ $0.04–0.80/1M   │     │     │ $1.75–5.00/1M  │
     └────────────────┘     │     └────────────────┘
                            │
                    ┌───────▼───────┐
                    │  Quality gate  │  ← confidence threshold check
                    │  If low conf,  │
                    │  escalate up   │
                    └───────────────┘
```

### The "Cascade" Pattern

1. **Try small model first** (Haiku, Gemini Flash, Llama 4 Scout)
2. **Check confidence** — if above threshold (e.g., 0.85), accept
3. **Escalate to large model** only for low-confidence results
4. **Typical escalation rate:** 15–25% of requests
5. **Effective cost:** ~30–40% of using the large model for everything

### Sruja-Specific Recommendation

| Sruja Task | Model | Why |
|---|---|---|
| Entity resolution (code→DSL matching) | Haiku / Flash / local embedding | Classification-like; high accuracy at small scale |
| Boundary violation detection | Deterministic rules first; LLM only for ambiguous cases | Most violations are structural — no LLM needed |
| Narrative enrichment (descriptions) | Sonnet / GPT-4.1 for final; Haiku for drafts | Quality matters for user-facing prose |
| Architecture classification | Haiku / Flash | Structured output, limited taxonomy |
| Semantic search queries | Haiku / Flash | Short input, short output |

---

## 4. RAG vs Full Context

### Cost Comparison (June 2026 Pricing)

| Approach | Tokens per call | Input cost (GPT-4.1) | Latency | Accuracy |
|---|---|---|---|---|
| Full context (100k tokens) | 100,000 in | $0.20/call | 15–30 sec | 60–70% recall (multi-fact) |
| RAG (top-5 chunks, ~2k tokens) | 2,000 in | $0.004/call | 1–2 sec | 85–92% recall |
| RAG + cache (retrieved + system) | 500 uncached | $0.001/call | 0.5–1 sec | 85–92% recall |

**Cost ratio: RAG is ~50–200x cheaper** per query than full-context for retrieval tasks.

### When RAG Loses Quality

RAG degrades when:
1. **Multi-hop reasoning** — answer requires connecting facts from different parts of a document that may end up in separate chunks
2. **Implicit queries** — user doesn't know what to search for ("what's concerning about this architecture?")
3. **Global understanding** — task requires understanding relationships spanning an entire codebase/document
4. **Chunk boundaries split relevant context** — poor chunking strategy destroys coherence

### When Full Context Wins

1. **Cross-document synthesis** — comparing patterns across files (e.g., architecture drift detection)
2. **Static small corpora** — under ~64K tokens that fit reliably within attention
3. **One-off analytical tasks** — legal/architectural review where cost and latency are secondary
4. **Document QA where position matters** — understanding clause interactions in contracts or boundary rules in .sruja files

### The "Lost in the Middle" Problem

- Models show U-shaped attention: best at the beginning and end of context
- Llama-3.1 degrades measurably after **32K tokens**
- GPT-4 holds until ~**64K tokens**
- Content in the middle of a 100K+ context suffers **20+ percentage point degradation**
- RAG naturally places retrieved content at the beginning, avoiding this

### Sruja-Specific Recommendation

For Sruja's use case (architecture scanner with repo.sruja files typically 5–50K tokens):

- **repo.sruja files < 32K tokens:** Full context is fine — send the whole file
- **repo.sruja files > 32K tokens:** Use RAG with chunking by architectural element
- **Multi-file analysis:** RAG is mandatory — no model handles 200K+ reliably
- **Narrative enrichment:** RAG with focused retrieval (the specific element + its neighbors)
- **Drift detection:** Full context of the changed file + RAG for the baseline comparison

The "intelligent routing" pattern fits Sruja well: small files → full context; large repos → RAG with progressive disclosure.

---

## 5. Embedding-First Approaches

### What Embeddings Can Replace

| Task | LLM Approach | Embedding Approach | Quality | Cost Ratio |
|---|---|---|---|---|
| **Entity resolution** | "Do A and B refer to the same thing?" → LLM judges | cosine similarity on embeddings | 85–92% vs 90–96% | 100–500x cheaper |
| **Classification** | "Classify into: {categories}" → LLM | KNN on labeled embeddings | 90–94% vs 93–97% | 50–200x cheaper |
| **Similarity search** | "Find things like X" → LLM scan | Vector similarity search | N/A (native task) | 1000x cheaper |
| **Clustering** | "Group these items" → LLM | Embedding → k-means/DBSCAN | Comparable | 100x cheaper |
| **Deduplication** | "Are these duplicates?" → LLM | Embedding similarity > threshold | 95%+ | 500x cheaper |

### Key Research Finding

From "Embeddings vs. Prompting for Multiclass Classification Tasks" (arXiv 2504.04277, 2025):

> *"The embeddings approach outperforms the best LLM prompts in terms of accuracy, calibration, latency, and financial cost."*

For classification tasks with a known label set, nearest-neighbor on embeddings **beats prompting GPT-4** — not just on cost, but on accuracy too.

### Embedding Model Pricing (June 2026)

| Provider | Model | Dimensions | Cost per 1M tokens |
|---|---|---|---|
| OpenAI | text-embedding-3-small | 1,536 | $0.02 |
| OpenAI | text-embedding-3-large | 3,072 | $0.13 |
| Google | text-embedding-004 | 768 | $0.025 (free tier: 2M tokens/min) |
| Cohere | embed-v4 | 1,024 | $0.10 |
| Local (fastembed) | all-MiniLM-L6-v2 | 384 | Free (CPU) |
| Local (fastembed) | bge-small-en-v1.5 | 384 | Free (CPU) |
| Local (fastembed) | bge-base-en-v1.5 | 768 | Free (CPU) |

### FilingDrift Case Study (Relevant to Sruja)

FilingDrift processes ~5,000 companies' SEC filings using embeddings instead of LLMs:
- **Deterministic:** Same input always produces the same score
- **No context window limits:** Compare any filing against thousands of peers
- **No hallucination:** Scores are mathematical operations on vectors
- **Cost:** "Hundreds of dollars per run" with LLMs → "fractions of a cent per document" with embeddings
- **Speed:** Full corpus re-embeds in minutes on CPU

### Sruja-Specific Recommendation

For entity resolution specifically (matching code elements to .sruja definitions):

1. **Phase 1 — Embedding filter:** Compute embeddings for all code elements and all .sruja elements. Use cosine similarity to find candidate matches (threshold > 0.85). **Cost: ~$0.00002 per comparison.**

2. **Phase 2 — LLM verification (only for borderline cases):** For pairs with similarity 0.75–0.85, use a small LLM (Haiku) to judge. **Eliminates 80–90% of LLM calls.**

3. **Phase 3 — Deterministic rules:** For exact string matches and structural patterns, skip both embeddings and LLM entirely.

This cascade handles 90%+ of cases with zero LLM cost, and reserves LLM calls for the genuinely ambiguous 10%.

---

## 6. Local/Small Models for Specific Tasks

### What Can Run On-Device (No API Call)

| Task | Tool | Model Size | Latency | Quality |
|---|---|---|---|---|
| **Embedding generation** | fastembed (ONNX) | ~80MB | <50ms per text | Excellent for similarity |
| **Text classification** | ONNX Runtime + small BERT | ~400MB | <20ms per text | 90–95% accuracy |
| **Named entity recognition** | spacy / ONNX NER model | ~200MB | <10ms per text | 85–92% accuracy |
| **Code parsing** | tree-sitter (no ML) | ~10MB | <5ms per file | Deterministic (100%) |
| **Similarity scoring** | cosine similarity on cached embeddings | N/A (math) | <1ms per comparison | Exact |
| **Pattern matching** | regex / structural rules | N/A | <1ms per file | Exact |
| **Lightweight generation** | candle (Rust) + Phi-3-mini | ~3.8GB | 2–5 sec | 75–85% for short outputs |
| **Reranking** | fastembed-rerank (ONNX) | ~400MB | <100ms | 90%+ relevance improvement |

### Fastembed (Rust + Python)

Qdrant's fastembed is particularly relevant since Sruja is Rust-based:

- **Rust crate:** `fastembed` on crates.io — uses ONNX Runtime for local inference
- **Supported models:** all-MiniLM-L6-v2, bge-small/base/large, multilingual models
- **No GPU required:** Optimized for CPU with ONNX Runtime
- **No API key:** Fully local, no network calls
- **Speed:** ~10ms per embedding on modern CPU (short texts); ~100ms for 512-token chunks

### Quality Tradeoff

| Task | API LLM (GPT-4.1) | Local embedding + rules | Delta |
|---|---|---|---|
| Entity resolution | 95% F1 | 88% F1 | -7% |
| Classification | 97% accuracy | 93% accuracy | -4% |
| Similarity search | 98% recall@10 | 94% recall@10 | -4% |
| Narrative generation | 9/10 quality | N/A (can't do locally) | — |

### Sruja-Specific Recommendation

**Tier 1 — Always local (zero API cost):**
- Tree-sitter parsing for structural analysis
- fastembed for similarity scoring and entity matching candidates
- Regex/pattern matching for known structural violations
- Cosine similarity for deduplication

**Tier 2 — Local with optional LLM verification:**
- Classification of architectural elements (local embedding → KNN, escalate if low confidence)
- Entity resolution (local embeddings filter → small LLM for borderline)

**Tier 3 — API LLM only:**
- Narrative generation (descriptions, explanations)
- Complex reasoning about architectural intent
- Multi-hop queries across the codebase

---

## 7. Token Budgeting Techniques

### Progressive Disclosure (Sruja's Current Pattern)

Sruja already uses this via MCP tools: `list_architecture_index` → `get_topology` → `get_elements`. Each step returns `estimated_tokens` and `next_suggested_tool`.

**Principle:** Don't send the entire context upfront. Start minimal, expand only what's needed.

```
Level 0: Element list + types (100 tokens)
    ↓ if more detail needed
Level 1: Topology / dependencies (500 tokens)
    ↓ if still unclear
Level 2: Element details + metadata (2,000 tokens)
    ↓ if deep analysis needed
Level 3: Full source / repo.sruja section (5,000–20,000 tokens)
```

**Token savings:** 80–95% of queries resolve at Level 0–2, avoiding the 20K+ token full context.

### Sliding Window for Code Context

Instead of sending an entire file:

```
┌─────────────────────────────┐
│ Function signature + docs   │ ← 100 tokens
├─────────────────────────────┤
│ Function body               │ ← 200 tokens
├─────────────────────────────┤
│ Relevant types/imports      │ ← 100 tokens
└─────────────────────────────┘
Total: 400 tokens vs 2,000+ for full file
```

### Summarization Hierarchy

For large repositories:

1. **Pre-compute summaries** at multiple granularity levels (function → module → crate → workspace)
2. **Store summaries alongside the architecture graph** in .sruja/
3. **Send only the relevant granularity level** to the LLM
4. **Expand on demand** — if the LLM needs more detail, fetch the next level down

### Token Budget Envelope

Set hard limits per task type:

| Task | Max input tokens | Max output tokens | Model |
|---|---|---|---|
| Classification | 2,000 | 100 | Haiku / Flash |
| Entity resolution | 3,000 | 200 | Haiku / Flash |
| Boundary check | 5,000 | 300 | Haiku / Flash |
| Narrative generation | 8,000 | 1,000 | Sonnet / GPT-4.1 |
| Architecture review | 15,000 | 2,000 | Sonnet / GPT-4.1 |

Enforce budgets client-side by truncating context that exceeds the limit, prioritizing most-relevant chunks.

### Semantic Caching

Cache LLM responses keyed by semantic similarity of the input:

- If a new query is >0.95 similar to a cached query, return the cached response
- Studies show **up to 73% cost reduction** for query patterns with high repetition
- Tools: GPTCache, Redis with vector similarity, custom embedding-based cache

For Sruja: cache the results of entity resolution and classification calls. Same code structure → same classification → no need to re-call.

---

## 8. Batching and Async

### Batch API Pricing (All 3 Major Providers)

All three offer **50% off** standard per-token pricing for async batch workloads:

| Dimension | OpenAI Batch | Anthropic Batches | Gemini Batch (Vertex) |
|---|---|---|---|
| **Discount** | 50% off | 50% off | 50% off |
| **Requests per batch** | 50,000 | 100,000 | Unlimited (GCS) |
| **File size limit** | 200MB | 256MB | GCS (no limit) |
| **Typical completion** | 2–6 hours | Under 1 hour | 1–6 hours |
| **SLA window** | 24 hours | 24 hours | 24 hours |
| **Setup complexity** | Low | Low | High (GCP/Vertex) |
| **Rate limit pool** | Separate from realtime | Separate from realtime | Separate |

### Latency Tradeoffs

- **Realtime API:** 1–5 seconds per call, full price
- **Batch API:** 1–6 hours turnaround, 50% off
- **Not suitable for:** Interactive user-facing features (chatbots, copilots)
- **Perfect for:** Overnight classification, bulk entity resolution, eval runs, corpus enrichment

### Real-World Savings

One developer audit: **$2,400/month → $890/month** by enabling batch APIs for non-realtime workloads (63% reduction, no quality degradation).

### Sruja-Specific Recommendation

**Batch-friendly tasks:**
- Initial entity resolution when scanning a new repository
- Bulk classification of architectural elements
- Narrative generation for entire repo.sruja files (generate all descriptions at once)
- Eval runs against the test suite

**Realtime-required tasks:**
- MCP tool queries (user is waiting)
- Interactive architecture exploration
- Live drift detection feedback

**Implementation pattern:**
```
sruja scan --batch    → queue all LLM calls as batch; results in 1-6 hours
sruja scan --live     → realtime API for interactive use
sruja scan --hybrid   → deterministic rules realtime; LLM enrichment batched
```

---

## 9. Cost Per Task Estimates (June 2026)

### Current LLM Pricing Reference

| Model | Input $/1M | Output $/1M | Context | Batch $/1M (in/out) |
|---|---|---|---|---|
| Claude Opus 4.6 | $5.00 | $25.00 | 200K | $2.50 / $12.50 |
| GPT-5.2 | $1.75 | $14.00 | 128K | $0.88 / $7.00 |
| Claude Sonnet 4.x | $3.00 | $15.00 | 200K | $1.50 / $7.50 |
| Gemini 3.1 Pro | $2.00 | $12.00 | 1M | $1.00 / $6.00 |
| GPT-4.1 | $2.00 | $8.00 | 128K | $1.00 / $4.00 |
| Claude Haiku 4.x | $0.80 | $4.00 | 200K | $0.40 / $2.00 |
| Gemini Flash 3.1 | $0.35 | $1.05 | 1M | $0.18 / $0.53 |
| GPT-4.1 mini | $0.40 | $1.60 | 128K | $0.20 / $0.80 |
| DeepSeek V3.2 | $0.14 | $0.28 | 128K | N/A |
| Llama 4 Scout (Groq) | $0.11 | $0.34 | 128K | N/A |
| Schematron-8B (inference.net) | $0.04 | $0.10 | 32K | N/A |

### Sruja Task Cost Estimates

Assumptions: Using Claude Haiku for routing/classification, Sonnet for narrative, prompt caching at 90% hit rate.

| Task | Avg input tokens | Avg output tokens | Model | Cost/call (no cache) | Cost/call (with cache) | Batch cost/call |
|---|---|---|---|---|---|---|
| **Entity resolution** (single element) | 1,500 | 150 | Haiku | $0.0018 | $0.0003 | $0.0009 |
| **Entity resolution** (full repo, 500 elements) | — | — | Haiku | $0.90 | $0.15 | $0.45 |
| **Classification** (single element) | 800 | 50 | Haiku | $0.00084 | $0.0002 | $0.00042 |
| **Classification** (full repo, 500 elements) | — | — | Haiku | $0.42 | $0.10 | $0.21 |
| **Narrative enrichment** (single description) | 2,000 | 300 | Sonnet | $0.0105 | $0.002 | $0.0053 |
| **Narrative enrichment** (full repo, 100 descriptions) | — | — | Sonnet | $1.05 | $0.20 | $0.53 |
| **Boundary violation check** | 3,000 | 200 | Haiku | $0.0032 | $0.0005 | $0.0016 |
| **Architecture review** (comprehensive) | 15,000 | 2,000 | Sonnet | $0.075 | $0.015 | $0.038 |
| **Semantic search query** | 2,000 | 100 | Haiku | $0.002 | $0.0003 | $0.001 |
| **Embedding generation** (per 1M tokens) | — | — | text-embedding-3-small | $0.02 total | — | $0.01 total |
| **Local embedding** (fastembed) | — | — | on-device | $0.00 | — | — |

### Full Repo Scan Cost Scenarios

For a medium-sized repo (500 architectural elements, 100 descriptions):

| Strategy | Cost per scan | Quality |
|---|---|---|
| **All Sonnet, no caching** | ~$8.50 | Highest |
| **Routed (Haiku + Sonnet), cached** | ~$1.20 | Nearly identical |
| **Routed + batch** | ~$0.60 | Nearly identical (6 hr delay) |
| **Embeddings + deterministic + selective LLM** | ~$0.15 | 90–95% of full LLM |
| **All local (fastembed + rules, no API)** | $0.00 | 85–90% of full LLM |

### Monthly Cost Projections

For a team scanning 50 repos/month:

| Strategy | Monthly cost | Annual cost |
|---|---|---|
| All Sonnet, no caching | $425 | $5,100 |
| Routed + cached | $60 | $720 |
| Routed + batch | $30 | $360 |
| Hybrid (local + selective API) | $7.50 | $90 |

---

## Implementation Priority for Sruja

Ordered by impact/effort ratio:

1. **Enable prompt caching** (Anthropic/OpenAI) — immediate 80–90% savings on repeated context, ~10 lines of code
2. **Use structured output** (JSON Schema) — eliminates retries, cuts output tokens 50–75%
3. **Route to small models** for classification/extraction — 8–12x cheaper, 90%+ quality
4. **Embedding-first entity resolution** — eliminate 80–90% of LLM calls for matching
5. **Batch API for non-interactive tasks** — 50% off with no quality loss
6. **Local embeddings via fastembed-rs** — zero API cost for similarity/classification
7. **Token budgets per task** — hard limits prevent runaway costs
8. **Semantic caching** — 73% savings on repeated/ similar queries
9. **Progressive disclosure** (already partially implemented via MCP) — avoid sending full context when summaries suffice

---

## Sources

- Prompt caching comparison: prompthub.us/blog/prompt-caching-with-openai-anthropic-and-google-models
- Batch API comparison: apiscout.dev/guides/openai-vs-anthropic-vs-gemini-batch-api-2026
- RAG vs long context: tianpan.co/blog/2026-04-09-long-context-vs-rag-production-decision-framework
- Embedding-first case study: filingdrift.com/blog/embeddings-vs-llms
- Embeddings vs prompting for classification: arxiv.org/html/2504.04277v1
- LLM pricing comparison: inference.net/content/llm-api-pricing-comparison
- Prompt caching savings: labeveryday.medium.com ($720→$72/month case study)
- Fastembed: github.com/qdrant/fastembed, crates.io/crates/fastembed
- Structured output: vllm.ai/en/latest/features/structured_outputs, dev.to/pockit_tools
- Batch API cost reduction: reddit.com/r/learnmachinelearning ($2,400→$890/month audit)
