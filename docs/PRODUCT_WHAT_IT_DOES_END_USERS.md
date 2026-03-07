# What Sruja Does (Product View for End Users)

This document describes what Sruja **actually does** from the perspective of someone using it—based on the real behavior of the product, not only the marketing or docs. It is written for end users (developers, tech leads, architects) in plain language.

---

## In One Sentence

**Sruja looks at your source code, figures out “who depends on whom,” and then tells you about structural problems (circular dependencies, disconnected or oversized pieces) and—optionally—how that compares to an architecture you describe in a file.**

---

## Who It’s For

- **Developers** who want a quick, zero-setup check on their repo’s dependency structure.
- **Tech leads / architects** who want to see cycles, orphans, and “god” modules, or to keep code aligned with a written architecture.
- **Teams** that want to put an architecture check in CI (e.g. “fail if we introduce new cycles or violations”).

It is **not** a full “understand my whole system’s domain and runtime” tool. It is a **structural dependency and drift** tool.

---

## What You Can Do (and What Actually Happens)

### 1. “Give me a quick picture of this repo” — **Quickstart**

**You run:** `sruja quickstart -r .` (or another path).

**What happens:**

1. Sruja **walks your repo** and only looks at source files it knows how to parse (today: **JavaScript, TypeScript, Python, Go, Rust**). It skips things like `node_modules`, `target`, and common test/example paths.
2. It **parses** those files (using language-specific parsers) and extracts **imports** and **exports**. From that it builds a **graph**: each file or logical module is a “node,” and each “A imports B” is a “link” (edge).
3. It **analyzes that graph** for:
   - **Circular dependencies** (e.g. A → B → C → A).
   - **Orphan modules** (nodes with no connections; possible dead code or missing integration). It tries to ignore test/example/tool paths.
   - **Layer violations**: if it thinks a node is “frontend” (label contains “frontend”, “ui”, “web”) and it links directly to something it thinks is a “database,” it flags that (e.g. “add a service layer”).
   - **God modules**: nodes with a lot of outgoing dependencies (e.g. more than 10), suggesting something that might need splitting.
4. It **scores** that into a single **health score (0–100)** and prints:
   - A short **inventory** (how many modules, services, databases it inferred).
   - **Top findings** (e.g. “Circular dependency: X → Y → Z → X”).
   - **Actionable fixes** (e.g. “Break cycles,” “Review orphans,” “Introduce layering”).
   - **Next steps** (e.g. run drift with a baseline, or ask “why” questions).

**No config, no API keys.** You get a snapshot of structure and issues in seconds.

---

### 2. “Does the code match our intended architecture?” — **Drift with a baseline**

**You run:** `sruja drift -r . -a architecture.sruja`

**What happens:**

1. Same **scan** as above: repo → graph of nodes and edges from code.
2. Sruja **parses your `.sruja` file** (your declared architecture: systems, containers, databases, relationships).
3. It **compares** the “actual” graph (from code) to the “declared” graph (from the file):
   - What’s in the code but not in the design?
   - What’s in the design but not in the code?
   - Are there direct DB accesses from “frontend”-like nodes?
4. It **reports** differences and violations. You can use this in **CI**: the command can exit with code 1 if there are errors (e.g. new cycles or layer violations), so the pipeline fails.

So: **it really does solve “is the code drifting from what we said?”** when you have a declared architecture file.

---

### 3. “Why are we using X?” — **Why**

**You run:** `sruja why "Why are we using Postgres?" -r .`

**What happens:**

1. Again, it **scans** the repo and builds the dependency graph (or uses a graph you already saved).
2. It **merges** that graph into an internal “knowledge graph” (nodes, edges, and—if present—decisions/ADRs).
3. It **interprets your question** by simple rules:
   - “Why” → looks for **technology** mentions (e.g. “Postgres”, “Node”) in the question and finds **nodes** that use that technology; if there are **decisions/ADRs** attached, it can cite those; otherwise it answers with “Component X uses Y technology” and points to file/location.
   - “What” / “Which” → looks for **kinds** (e.g. services, databases) and lists them.
   - “How” → looks at **paths** (how components connect).
   - If nothing matches, it falls back to a **generic** answer (e.g. counts and a short summary).

**Important:** Answers are **deterministic** and based only on the graph (and any ADRs you’ve added). There is **no LLM** in the default “why” path—so you get “we have component X that uses Postgres, here’s the file” rather than a deep narrative. It’s **evidence from code and declared decisions**, not creative explanation.

So: **it really does solve “point me to where and why we use X”** in a narrow, evidence-based way.

---

### 4. “Catch bad structure in CI” — **Drift (no baseline)**

**You run:** `sruja drift -r .` (no `-a` file).

**What happens:**

1. **Scan** → build graph from code.
2. **Same structural checks** as quickstart: cycles, orphans, layer violations, god modules.
3. **Report** + **health score**. If there are **errors** (e.g. circular dependencies, layer violations), the command **exits with 1**, so CI can fail.

So: **it really does solve “block PRs that introduce certain structural problems.”**

---

### 5. “Define architecture in code and export docs” — **DSL + Lint + Export**

**You:** Write a `.sruja` file (systems, containers, databases, relationships in a simple text format).

**You run:**  
`sruja lint example.sruja`  
`sruja export markdown example.sruja`  
`sruja export mermaid example.sruja`

**What happens:**

- **Lint:** Checks that the file is valid (no undefined refs, no cycles at the system level, etc.).
- **Export:** Produces **Markdown** or **Mermaid** diagrams from that file.

So: **it really does solve “version-controlled architecture docs and diagrams from one source file.”**

---

### 6. Optional: Deeper analysis and desktop app

- **`sruja analyze`** runs **structural** (same graph + complexity metrics) plus **semantic** (e.g. vocabulary, coupling) and **intent** (e.g. ADR vs code). Some parts need more setup (e.g. intent directory, optional traces).
- **Desktop app:** A **Slack-style** UI for chat, with optional **AI agents** and **extraction** of decisions from conversation. This path typically **requires an LLM API key** and is separate from the “zero-key” CLI flow above.

---

## What Sruja Is **Not** Doing (By Design or Limitation)

| Area | Reality |
|------|--------|
| **Semantic meaning** | It sees “A imports B,” not “A is the payment service.” It doesn’t reason about domain or responsibility. |
| **Layering** | “Frontend” and “database” are inferred from **labels/paths** (e.g. “frontend”, “ui”, “web”). It doesn’t model your real layer rules (e.g. “domain must not depend on infra”). |
| **Runtime / deployment** | It doesn’t see processes, containers, or network. “Which service talks to which over the wire?” is outside this. |
| **Data flow** | It sees dependency edges, not “this data flows from A to B” or read vs write. |
| **Languages** | Only **JS/TS, Python, Go, Rust** are parsed. Other languages get no or partial structure. |
| **Truth** | Node kinds (service, database) and layer rules are **heuristics**. They can be wrong (e.g. orphans that are entry points, “god” modules that are facades). |
| **“Why” depth** | “Why” is **evidence-based**: “component X uses Y, here’s the file.” It is not a full narrative explanation unless you add ADRs/decisions. |

So: for **“understand the whole architecture, domain, runtime, and data flow”** it is **not** sufficient by itself. For **“see structure, find obvious structural problems, compare to what we said, and ask evidence-based ‘why’ questions”** it **is** sufficient.

---

## Is It Really Solving the Problem?

**Yes, for the problem it sets out to solve:**

1. **“I want a fast, zero-setup structural view of my repo”**  
   → **Yes.** Quickstart gives you a graph, a health score, and a short list of findings (cycles, orphans, layer violations, god modules) with no config.

2. **“I want to enforce that code doesn’t drift from our written architecture”**  
   → **Yes.** With a `.sruja` baseline, drift compares code to design and can fail CI.

3. **“I want to block obvious structural regressions (e.g. new cycles) in CI”**  
   → **Yes.** `sruja drift -r .` (no baseline) does that.

4. **“I want to ask ‘why do we use X?’ and get evidence from code (and optionally ADRs)”**  
   → **Yes.** You get deterministic, citeable answers (component + technology + file), not hand-wavy narrative.

5. **“I want architecture-as-code and generated docs/diagrams”**  
   → **Yes.** Lint + export from `.sruja` works.

**No, if you expect:**

- Deep **semantic** or **domain** understanding.
- **Runtime** or **deployment** view.
- **Full** “why” storytelling without ADRs.
- Support for **all** languages.

So: **it really is solving “structural dependency analysis, drift vs declared architecture, and evidence-based ‘why’ answers.”** It is **not** solving “full architecture understanding and quality judgment” by itself—and the codebase and internal docs are clear about that.

---

## Summary for End Users

- **Use Sruja for:** Quick structural health, finding cycles/orphans/layer issues, comparing code to a declared architecture, CI gates on structure, and evidence-based “why do we use X?” answers.
- **Don’t rely on it alone for:** Domain modeling, runtime/deployment view, data flow, or a single “is this architecture good?” verdict. Combine it with docs, ADRs, and human judgment.
- **Best fit:** Teams that want **low-friction**, **repeatable** structure and drift checks, and optional architecture-as-code with generated docs.
