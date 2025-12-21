# Real-World Examples

This directory contains realistic, production-ready examples that demonstrate Sruja’s capabilities on modern architectures. Use them as templates and learning aids.

## Microservices with Implied Relationships

**File:** `pattern_microservices.sruja` and `demo_implied_relationships.sruja`

**Scenario:** E-commerce microservices platform

**Highlights:**
- Many services, queues, and data stores
- External integrations
- Implied parent relationships reduce boilerplate

**Benefits:**
- ~30–40% fewer explicit relations
- Clear service-to-service and system-level views

## Views Customization for Stakeholders

**File:** `demo_views_customization.sruja`

**Scenario:** SaaS analytics platform

**Highlights:**
- Multiple stakeholder views (Developer, Product, Data Flow, Executive)
- Tag-based styling and filtered diagrams
- One architecture, many tailored views

## Agentic AI Systems

**File:** `pattern_agentic_ai.sruja`

**Scenario:** Multi-agent orchestration (planner, executor, tools)

**Highlights:**
- Agent graph orchestration
- Tooling and memory components
- Governance and constraints for safe execution

## RAG Pipeline

**File:** `pattern_rag_pipeline.sruja`

**Scenario:** Retrieval-Augmented Generation for production Q&A

**Highlights:**
- Indexing and retrieval services
- Embeddings, vector store, and generation
- Observability and evaluation hooks

## SaaS and Platform Architectures

**Files:** `project_saas_platform.sruja`, `project_ecommerce.sruja`, `project_iot_platform.sruja`

**Scenario:** Full-platform architectures with production features

**Highlights:**
- Clear boundaries and contracts
- Event-driven flows where appropriate
- Deployment considerations

## Usage

```bash
# Export as JSON (supported format)
sruja export json examples/pattern_agentic_ai.sruja
sruja export json examples/pattern_rag_pipeline.sruja > rag.json

# Extended JSON includes views metadata if present
sruja export json --extended examples/demo_views_customization.sruja
```

Note: Markdown, SVG, and HTML exports are produced via the website viewers. Use the website’s preview to render diagrams and styled views.

## Key Takeaways

**Implied Relationships**
- Best for complex architectures with nested elements
- Reduce repetitive parent relations

**Views Block**
- Provide audience-specific filtered/styled views
- Keep DSL minimal; add views only when needed

**AI Patterns**
- Agentic orchestration and RAG are first-class modeling targets
- Model tools, memory, policies, and evaluation for production readiness

## Next Steps

- Try the examples and adapt them to your system
- Combine implied relationships with stakeholder views
- Add policies, requirements, and SLOs to capture governance
