use serde_json::{json, Value};

pub(crate) fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "sruja_get_repomap",
            "title": "Sruja RepoMap",
            "description": "Generate a token-optimized repository map with tree-sitter signatures.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_list_architecture_index",
            "title": "Sruja Architecture Index",
            "description": "Progressive disclosure: list architecture element IDs with minimal metadata (Index layer). Prefers declared architecture (repo.sruja) and falls back to scanned graph nodes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "max_tokens": { "type": "integer", "description": "Maximum estimated tokens for the response (default: 2000)", "minimum": 200 },
                    "kinds": { "type": "array", "items": { "type": "string" }, "description": "Optional element kind filter (e.g. system, container, component, database, queue, person)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_topology",
            "title": "Sruja Topology",
            "description": "Progressive disclosure: get upstream/downstream topology for an element (Topology layer). Prefers declared relationships (repo.sruja) and falls back to scanned graph blast radius.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "id": { "type": "string", "description": "Element ID (prefer fully-qualified IDs)" },
                    "depth": { "type": "integer", "description": "Traversal depth (default: 1)", "minimum": 1, "maximum": 4 },
                    "max_tokens": { "type": "integer", "description": "Maximum estimated tokens for the response (default: 5000)", "minimum": 500 }
                },
                "required": ["id"]
            }
        }),
        json!({
            "name": "sruja_get_elements",
            "title": "Sruja Get Elements",
            "description": "Progressive disclosure: fetch element details for a list of IDs (Detail layer). Prefers declared elements (repo.sruja) and falls back to scanned graph nodes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "ids": { "type": "array", "items": { "type": "string" }, "description": "Element IDs to fetch" },
                    "max_tokens": { "type": "integer", "description": "Maximum estimated tokens for the response (default: 8000)", "minimum": 500 }
                },
                "required": ["ids"]
            }
        }),
        json!({
            "name": "sruja_get_diagnostic_full",
            "title": "Sruja Diagnostic Full Text",
            "description": "Fetch the full text of a truncated diagnostic stored under .sruja/vfs/diagnostics/ (URI from head/tail truncation payloads).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "uri": { "type": "string", "description": "sruja-vfs://diagnostics/<filename> or bare filename" }
                },
                "required": ["uri"]
            }
        }),
        json!({
            "name": "sruja_suggest_context_prune",
            "title": "Sruja Suggest Context Prune",
            "description": "Graph-aware prune suggestion: which session element IDs to compress vs keep based on topology distance to active focus. Host applies compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "active_element_ids": { "type": "array", "items": { "type": "string" }, "description": "Currently focused architecture element IDs" },
                    "session_element_ids": { "type": "array", "items": { "type": "string" }, "description": "Element IDs mentioned in the session context" },
                    "depth": { "type": "integer", "description": "Topology hops from active focus (default 2)", "minimum": 1, "maximum": 4 }
                },
                "required": ["active_element_ids", "session_element_ids"]
            }
        }),
        json!({
            "name": "sruja_get_drift_state",
            "title": "Sruja Drift State Injector",
            "description": "Compact structured drift payload (drift_state/v1) for host context injection—prefer over pasting full drift reports.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_memory_timeline",
            "title": "Sruja Memory Timeline",
            "description": "Chronological memory slice around an anchor event id or ISO timestamp (learnings, events, decisions).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "anchor_id": { "type": "string", "description": "Memory entry id to center on" },
                    "anchor_timestamp": { "type": "string", "description": "ISO-8601 timestamp anchor (if anchor_id omitted)" },
                    "before": { "type": "integer", "description": "Entries before anchor (default 10)", "minimum": 0, "maximum": 500 },
                    "after": { "type": "integer", "description": "Entries after anchor (default 10)", "minimum": 0, "maximum": 500 },
                    "decision_id": { "type": "string", "description": "Optional decision id filter" },
                    "element_id": { "type": "string", "description": "Optional element id filter" }
                }
            }
        }),
        json!({
            "name": "sruja_reindex_memory",
            "title": "Sruja Reindex Memory",
            "description": "Rebuild .sruja/memory.sqlite from agent_memory.json, context_events.jsonl, and .sruja/decisions/*.md.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_add_element",
            "title": "Sruja Add Element",
            "description": "Add a new element (system, container, component, database, person) to the architecture.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "id": { "type": "string", "description": "Unique ID for the element (e.g. MySystem.Api)" },
                    "kind": { "type": "string", "description": "Kind: system, container, component, database, person" },
                    "title": { "type": "string", "description": "Human-readable title" },
                    "description": { "type": "string", "description": "Description of the element" },
                    "technology": { "type": "string", "description": "Technology used (for containers/components)" }
                },
                "required": ["id", "kind", "title"]
            }
        }),
        json!({
            "name": "sruja_add_relationship",
            "title": "Sruja Add Relationship",
            "description": "Add a new relationship between two elements.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "source": { "type": "string", "description": "Source element ID" },
                    "target": { "type": "string", "description": "Target element ID" },
                    "label": { "type": "string", "description": "Relationship label (e.g. HTTPS, SQL)" },
                    "technology": { "type": "string", "description": "Technology used (optional)" }
                },
                "required": ["source", "target"]
            }
        }),
        json!({
            "name": "sruja_get_task_context",
            "title": "Sruja Task Context",
            "description": "Get high-fidelity architectural context for a specific task. Supports selection by element ID, file path, git diff (base/head refs), or search query. Returns focus elements, neighbors, impact analysis, hydrated source code, and a grounding_trace that explains how the focus and evidence were selected.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Specific architectural element ID (e.g. MySystem.Api)" },
                    "file": { "type": "string", "description": "File path to resolve architectural focus from" },
                    "query": { "type": "string", "description": "Search query for semantic architectural lookup" },
                    "base_ref": { "type": "string", "description": "Git base ref for diff-based context (baseline)" },
                    "head_ref": { "type": "string", "description": "Git head ref for diff-based context (current changes)" },
                    "depth": { "type": "integer", "description": "Depth of neighbor expansion (default: 1, max: 4)", "minimum": 1, "maximum": 4 },
                    "max_tokens": { "type": "integer", "description": "Maximum tokens for hydrated source code (default: 10000)" },
                    "enrich": { "type": "boolean", "description": "If true, add optional enrichment (cmd/openai) grounded in the task context. Default: false." },
                    "enrich_provider": { "type": "string", "description": "Enrichment provider: cmd|openai. Default: cmd." },
                    "enrich_cmd": { "type": "string", "description": "External enrichment command (stdin JSON -> stdout markdown)." },
                    "enrich_model": { "type": "string", "description": "Model name for provider=openai (default: gpt-4o-mini)." },
                    "enrich_base_url": { "type": "string", "description": "Base URL for provider=openai (default: https://api.openai.com/v1)." },
                    "enrich_timeout_ms": { "type": "integer", "description": "Timeout for enrichment (ms, default: 15000)." },
                    "enrich_max_bytes": { "type": "integer", "description": "Max bytes to read from enrichment stdout (default: 20000)." },
                    "cache_friendly": { "type": "boolean", "description": "If true, return invariant/tools/volatile JSON for prompt-cache-friendly payloads (default: false)." },
                    "workflow_id": { "type": "string", "description": "Workflow under .sruja/workflows/ for phase-scoped context." },
                    "phase": { "type": "string", "description": "Workflow phase (inception|construction|operations) to tune token budget." }
                }
            }
        }),
        json!({
            "name": "sruja_get_context_events",
            "title": "Sruja Context Events",
            "description": "Read recent append-only context lineage events from .sruja/context_events.jsonl (intent_check, drift, proposal_merge, learn_run, and context_event/v2 decision traces). Newest first.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "limit": { "type": "integer", "description": "Max events to return (default: 50)" },
                    "kind": { "type": "string", "description": "Optional filter on event kind" },
                    "details_substring": { "type": "string", "description": "Optional substring filter on JSON details" },
                    "decision_id": { "type": "string", "description": "Optional filter: decision_id field or details contains this id" },
                    "trace_id": { "type": "string", "description": "Optional filter: trace_id field or details contains" },
                    "element_id": { "type": "string", "description": "Optional filter: elements array or details mentions this architecture id" },
                    "decision_lineage_only": { "type": "boolean", "description": "If true, only decision/workflow lineage kinds (decision_opened, context_retrieved, ...)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_workflow",
            "title": "Sruja Workflow",
            "description": "Read a workflow manifest and its phase artifact paths from .sruja/workflows/<id>/manifest.json.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "workflow_id": { "type": "string", "description": "Workflow id under .sruja/workflows/" }
                },
                "required": ["workflow_id"]
            }
        }),
        json!({
            "name": "sruja_workflow_gate_check",
            "title": "Sruja Workflow Gate Check",
            "description": "Check whether a workflow phase gate allows construction-time code generation. Returns allowed, phase, and missing items. Missing workflow falls back to allowed=true.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "workflow_id": { "type": "string", "description": "Workflow id under .sruja/workflows/" }
                },
                "required": ["workflow_id"]
            }
        }),
        json!({
            "name": "sruja_workflow_summary",
            "title": "Sruja Workflow Summary",
            "description": "Get a complete structured summary of a workflow, including status, phase, profile, required/completed files, test results, design reviews, and readiness checklists.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "workflow_id": { "type": "string", "description": "Workflow ID under .sruja/workflows/" }
                },
                "required": ["workflow_id"]
            }
        }),
        json!({
            "name": "sruja_workflow_next_steps",
            "title": "Sruja Workflow Next Steps",
            "description": "Compute actionable next steps and recommendations for a workflow to proceed to the next phase, based on its profile and current status.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "workflow_id": { "type": "string", "description": "Workflow ID under .sruja/workflows/" }
                },
                "required": ["workflow_id"]
            }
        }),
        json!({
            "name": "sruja_get_decisions",
            "title": "Sruja Decision Records",
            "description": "List Decision Record files (.sruja/decisions/*.md) with YAML front matter (generalized ADRs).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_record_context_event",
            "title": "Sruja Record Context Event",
            "description": "Append one context_event/v1 or v2 JSON object to .sruja/context_events.jsonl (same contract as `sruja event append`).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "event": { "type": "object", "description": "Full ContextEventRecord JSON (schema_version, timestamp, kind, outcome, ...)" }
                },
                "required": ["event"]
            }
        }),
        json!({
            "name": "sruja_create_decision_record",
            "title": "Sruja Create Decision Record",
            "description": "Create a proposed Decision Record under .sruja/decisions/ and emit decision_opened event.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "title": { "type": "string" },
                    "record_type": { "type": "string", "description": "architecture | product | operational | security | agent | exception" },
                    "scope": { "type": "string", "description": "repo | workflow | system | organization (default: repo)" }
                },
                "required": ["title", "record_type"]
            }
        }),
        json!({
            "name": "sruja_link_decision_to_element",
            "title": "Sruja Link Decision To Element",
            "description": "Append an architecture element id to a Decision Record YAML front matter.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "decision_id": { "type": "string" },
                    "element_id": { "type": "string" }
                },
                "required": ["decision_id", "element_id"]
            }
        }),
        json!({
            "name": "sruja_get_author_evidence",
            "title": "Sruja Author Evidence",
            "description": "Load or build `.sruja/author_evidence.json` (a capped, citeable evidence bundle for grounded architecture authoring).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
        json!({
            "name": "sruja_propose_change",
            "title": "Sruja Propose Change",
            "description": "Propose an architectural change before writing code. Creates a validated proposal for human review.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path" },
                    "description": { "type": "string", "description": "What this change does and why" },
                    "add_elements": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "kind": { "type": "string" },
                                "label": { "type": "string" },
                                "technology": { "type": "string" }
                            },
                            "required": ["id", "kind", "label"]
                        }
                    },
                    "add_relationships": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "source": { "type": "string" },
                                "target": { "type": "string" },
                                "label": { "type": "string" }
                            },
                            "required": ["source", "target"]
                        }
                    },
                    "remove_elements": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                },
                "required": ["description"]
            }
        }),
        json!({
            "name": "sruja_critique",
            "title": "Sruja Adversarial Critique",
            "description": "Adversarial architectural review. Actively finds problems in proposed changes by cross-referencing policies, historical incidents, tribal knowledge, blast radius, and behavioral contracts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path" },
                    "files": { "type": "array", "items": { "type": "string" }, "description": "Changed file paths to critique" },
                    "description": { "type": "string", "description": "What this change does (helps pattern matching)" },
                    "proposal_id": { "type": "string", "description": "Proposal ID if this is an approved proposal" },
                    "base_ref": { "type": "string", "description": "Git base ref for diff-based critique" },
                    "head_ref": { "type": "string", "description": "Git head ref for diff-based critique" }
                }
            }
        }),
        json!({
            "name": "sruja_get_state_machine",
            "title": "Sruja Get State Machine",
            "description": "Get the state machine definition for a component. Returns states, transitions, guards, and actions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Component ID with state machine (e.g. MySystem.Api)" }
                },
                "required": ["element_id"]
            }
        }),
        json!({
            "name": "sruja_get_contract",
            "title": "Sruja Get Contract",
            "description": "Get the API contract (input/output spec) for a component. Ideal for generating client code or stubs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Component ID with contract (e.g. MySystem.Api)" },
                    "contract_name": { "type": "string", "description": "Optional specific contract name if the component has multiple" }
                },
                "required": ["element_id"]
            }
        }),
        json!({
            "name": "sruja_preflight_check",
            "title": "Sruja Preflight Check",
            "description": "Before generating code, check what architectural constraints, policies, boundaries, and known risks apply to the target area. Call this BEFORE writing any code.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "target_files": { "type": "array", "items": { "type": "string" }, "description": "List of files you plan to modify" },
                    "intent": { "type": "string", "description": "What you plan to do (optional)" }
                },
                "required": ["target_files"]
            }
        }),
        json!({
            "name": "sruja_sandbox",
            "title": "Sruja Experiment Sandbox",
            "description": "Create, commit, or discard an isolated git worktree for safe architectural experimentation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "action": { "type": "string", "description": "Action: 'create', 'commit', 'discard', 'list'" },
                    "name": { "type": "string", "description": "Name of the sandbox/experiment." }
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "sruja_evaluate_proposal",
            "title": "Sruja Evaluate Proposal",
            "description": "Evaluate the current state of the codebase against your architectural hypothesis. Calculates the current Context Score and runs an optional 'gate' command (e.g. 'make check').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "gate_command": { "type": "string", "description": "Optional terminal command to run as a regression gate (e.g., 'make check' or 'cargo test')" }
                }
            }
        }),
        json!({
            "name": "sruja_record_learning",
            "title": "Sruja Record Learning",
            "description": "Record an architectural learning, failed hypothesis, or guardrail advice in the Agentic Memory. Used to prevent future agents from repeating mistakes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "context": { "type": "string", "description": "Context of the learning (e.g. 'Refactoring Auth')" },
                    "hypothesis": { "type": "string", "description": "What was being tried" },
                    "outcome": { "type": "string", "description": "Outcome: 'success' or 'failed'" },
                    "reason": { "type": "string", "description": "Why it failed (if applicable)" },
                    "guardrail_advice": { "type": "string", "description": "Explicit advice for future agents (e.g. 'Do not merge X into Y')" },
                    "affected_elements": { "type": "array", "items": { "type": "string" }, "description": "Architectural element IDs affected by this learning" },
                    "hitl_kind": { "type": "string", "description": "Optional: precedent | exception | correction | guardrail (human-in-the-loop classification)" }
                },
                "required": ["context", "hypothesis", "outcome", "guardrail_advice"]
            }
        }),
        json!({
            "name": "sruja_bm25_search",
            "title": "Sruja BM25 Search",
            "description": "Search ingested context documents (.sruja/context/) using BM25 keyword retrieval. Best for exact terms, acronyms, API identifiers, and error codes that embedding search may miss.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "query": { "type": "string", "description": "Search query (keywords, terms, identifiers)" },
                    "max_results": { "type": "integer", "description": "Maximum results to return (default: 5)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "sruja_hybrid_query",
            "title": "Sruja Hybrid Query",
            "description": "Preferred default for most natural-language architecture questions: classifies query complexity and routes to graph-only, semantic-only, or hybrid retrieval.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "query": { "type": "string", "description": "Natural language query about the architecture" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "sruja_agent_run",
            "title": "Sruja Agent Run",
            "description": "Run Sruja's agent loop (observe→plan→optional apply→verify) and return structured JSON output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "run_id": { "type": "string", "description": "Optional run ID for tracing (defaults to auto-generated)" },
                    "goal": { "type": "string", "description": "Natural language goal for the agent" },
                    "file": { "type": "string", "description": "Optional file focus (exactly one of file/element_id/query)" },
                    "element_id": { "type": "string", "description": "Optional element id focus (exactly one of file/element_id/query)" },
                    "query": { "type": "string", "description": "Optional query focus (exactly one of file/element_id/query)" },
                    "mode": { "type": "string", "description": "plan|apply (default: plan)" },
                    "ai_mode": { "type": "string", "description": "standard|conservative|aggressive (default: standard)" },
                    "max_steps": { "type": "integer", "description": "Optional max steps override" },
                    "max_runtime_ms_per_step": { "type": "integer", "description": "Optional per-step timeout override (ms)" },
                    "enrich": { "type": "boolean", "description": "Optional enrichment grounded in facts (default: false)" },
                    "enrich_provider": { "type": "string", "description": "cmd|openai (or configured default)" },
                    "enrich_cmd": { "type": "string", "description": "External enrichment command (stdin JSON -> stdout markdown)" },
                    "enrich_model": { "type": "string", "description": "Model name for provider=openai" },
                    "enrich_base_url": { "type": "string", "description": "Base URL for provider=openai (OpenAI-compatible supported)" },
                    "enrich_timeout_ms": { "type": "integer", "description": "Enrichment timeout (ms)" },
                    "enrich_max_bytes": { "type": "integer", "description": "Max bytes to read from enrichment output" },
                    "continue_on_error": { "type": "boolean", "description": "If true, continue verification after errors" }
                },
                "required": ["goal"]
            }
        }),
        json!({
            "name": "sruja_verify_task",
            "title": "Sruja Verify Task",
            "description": "Run verification steps for a task profile (coding, bugfix, review, arch) and return structured results. Use after code changes to ensure lint, tests, drift, and intent checks pass.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "profile": { "type": "string", "description": "Verification profile: coding, bugfix, review, arch (default: coding)" },
                    "file": { "type": "string", "description": "Optional file focus (used by bugfix profile)" },
                    "max_runtime_ms": { "type": "integer", "description": "Max runtime per step in milliseconds (default: 30000)" }
                }
            }
        }),
        json!({
            "name": "sruja_get_boundaries",
            "title": "Sruja Get Boundaries",
            "description": "Get architectural boundaries and allowed connections for a component or file. Falls back to inferred boundaries if no .sruja exists.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "element_id": { "type": "string", "description": "Optional component ID to filter boundaries (e.g. MySystem.Api)" },
                    "file": { "type": "string", "description": "Optional file path to resolve target component from" }
                }
            }
        }),
        json!({
            "name": "sruja_suggest_fix",
            "title": "Sruja Suggest Fix",
            "description": "Given a violation (or all active violations), returns structured suggestion options to resolve it (DSL fixes, code refactoring).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "violation": {
                        "type": "object",
                        "description": "Optional specific violation object to generate suggestions for. If omitted, checks the entire repo and generates fixes for all active violations."
                    }
                }
            }
        }),
        json!({
            "name": "sruja_check_violations",
            "title": "Sruja Check Violations",
            "description": "Validate changes against architectural boundaries and policies. Returns list of violations. If files array is provided, validates those changed files. Otherwise, checks the entire repo.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "files": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional list of changed file paths to validate"
                    }
                }
            }
        }),
        json!({
            "name": "sruja_classify",
            "title": "Sruja Classify",
            "description": "Generate or set .sruja/classification.json for a repository. Without a classification argument, runs heuristic classification. With a classification argument, writes the provided JSON directly — use this when you (the AI agent) have analyzed the codebase and determined the layers, boundaries, and rules.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "force": { "type": "boolean", "description": "Overwrite existing classification.json (default: false)" },
                    "classification": {
                        "type": "object",
                        "description": "Classification JSON to write. Use this when you have analyzed the codebase and determined the architecture. Schema: { schema_version, project_type, summary: { crates?, source_files }, layers: [{ name, members }], boundaries: [{ from, to, allowed, reason }], forbidden_patterns: [] }",
                        "properties": {
                            "schema_version": { "type": "string" },
                            "project_type": { "type": "string" },
                            "summary": { "type": "object" },
                            "layers": { "type": "array" },
                            "boundaries": { "type": "array" },
                            "forbidden_patterns": { "type": "array" }
                        }
                    }
                }
            }
        }),
        json!({
            "name": "sruja_sync_ide_rules",
            "title": "Sruja Sync IDE Rules",
            "description": "Generate IDE context files (.cursorrules, copilot-instructions.md, llms-architecture.txt) from the current classification. Run this after sruja_classify to update IDE context.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "max_tokens": { "type": "integer", "description": "Maximum tokens for generated content (default: 10000)" },
                    "check": { "type": "boolean", "description": "If true, verify files match instead of writing (default: false)" }
                }
            }
        }),
        json!({
            "name": "sruja_verify_architecture",
            "title": "Sruja Verify Architecture",
            "description": "Unified passive validation check. Performs lint check on DSL, detects architectural drift, and critiques recent changes against policies and intent under a single token-friendly call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" },
                    "files": { "type": "array", "items": { "type": "string" }, "description": "Optional list of changed files to target the critique and drift checks" }
                }
            }
        }),
        json!({
            "name": "sruja_get_quick_context",
            "title": "Sruja Quick Context",
            "description": "Get a compact repository summary for zero-setup AI context. Returns top modules by centrality, entrypoints, data stores, and auto-discovered context (CI, compose, terraform). Token budget: ~300 tokens.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Repository root path (defaults to .)" }
                }
            }
        }),
    ]
}
