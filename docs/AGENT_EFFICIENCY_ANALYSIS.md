# Sruja Agent Efficiency Analysis

## Current State

The sruja agent system has been tested with multiple LLM providers and real coding tasks. Here's what works and what needs improvement.

## What Works Well

### 1. Multi-Provider Support
- **Z.AI (GLM)**: Works with glm-4-flash model
- **XIMIMO**: Works with mimo-v2.5-pro model  
- **OpenRouter**: Works with google/gemini-2.5-flash, anthropic/claude-sonnet-4, meta-llama/llama-4-maverick

### 2. Deterministic Plan/Apply Workflow
- `sruja agent plan` generates reproducible plans from repo evidence
- `sruja agent apply` executes verification steps
- Plans are grounded in architecture context (drift, intent, focus)

### 3. Memory System
- `sruja agent history` shows learning history
- `sruja agent record` records new learnings
- `sruja agent clusters` shows thematic groups
- `sruja agent curate` suggests merges/deletions
- `sruja agent distill` records task outcomes
- `sruja agent session-summary` writes handoff summaries
- `sruja agent propose-fact` proposes architectural facts

### 4. Architecture Integration
- Agent understands codebase structure via sruja graph
- Plans respect layer boundaries and architecture rules
- Verification includes drift, intent, and compliance checks

## Efficiency Issues

### 1. Agent Loop Authentication
**Problem**: `sruja agent loop` requires `OPENAI_API_KEY` environment variable but authentication fails even when set.

**Root Cause**: The agent loop reads API key from environment variables (`OPENAI_API_KEY`, `SRUJA_ENRICH_API_KEY`) but doesn't read from `.sruja/config.toml` where `sruja agent setup` writes the configuration.

**Impact**: The autonomous coding loop cannot be used with configured providers.

**Fix**: Modify `agent_loop.rs` to read API key from `.sruja/config.toml` when environment variables are not set.

### 2. Plan Execution Limitations
**Problem**: `sruja agent apply` runs verification commands but doesn't execute code changes.

**Root Cause**: The apply command is designed to run sruja verification steps, not arbitrary code modifications.

**Impact**: Agent cannot autonomously write code changes.

**Fix**: Extend apply command to support code modification steps or integrate with external coding agents.

### 3. Provider Configuration Duplication
**Problem**: Provider config is stored in `.sruja/config.toml` but agent loop requires environment variables.

**Root Cause**: Two separate configuration systems:
- `sruja agent setup` writes to `.sruja/config.toml`
- `sruja agent loop` reads from environment variables

**Impact**: Users must configure both systems separately.

**Fix**: Unify configuration - agent loop should read from `.sruja/config.toml` first, then fall back to environment variables.

### 4. Missing LLM Integration in Plan Generation
**Problem**: `sruja agent plan` generates deterministic plans without LLM enrichment by default.

**Root Cause**: Plan generation is deterministic (based on repo evidence) and doesn't use LLM for narrative.

**Impact**: Plans are technically correct but may lack context or explanation.

**Fix**: Use `--enrich` flag to add LLM-generated narrative sections.

## Recommendations

### 1. Unify Configuration System
```rust
// In agent_loop.rs, add config file reading:
fn resolve_api_key() -> Result<String, CliError> {
    // 1. Try environment variables
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        return Ok(key);
    }
    if let Ok(key) = std::env::var("SRUJA_ENRICH_API_KEY") {
        return Ok(key);
    }
    
    // 2. Try .sruja/config.toml
    let config_path = Path::new(".sruja/config.toml");
    if config_path.exists() {
        let config = std::fs::read_to_string(config_path)?;
        let toml: toml::Value = toml::from_str(&config)?;
        if let Some(key) = toml.get("integrations")
            .and_then(|i| i.get("api_key"))
            .and_then(|k| k.as_str()) {
            return Ok(key.to_string());
        }
    }
    
    Err(CliError::validation("No API key found"))
}
```

### 2. Add Code Modification Support
Extend the plan/apply workflow to support code changes:

```json
{
  "steps": [
    {
      "id": "step_1",
      "kind": "file_edit",
      "file": "crates/sruja-cli/src/cli/commands.rs",
      "line_range": [805, 809],
      "new_content": "/// Agentic memory: learnings, guardrails, failed hypotheses\n/// For architecture work only. Use `sruja agent plan` for coding tasks.\nAgent { ... }"
    }
  ]
}
```

### 3. Improve Agent Loop Integration
Make agent loop work with configured providers:

```rust
// In agent_loop.rs:
let api_key = resolve_api_key()?;
let base_url = options.base_url
    .or(manifest.base_url.as_deref())
    .or_else(|| read_config_value("integrations.base_url"))
    .unwrap_or("https://api.openai.com/v1");
let model = options.model
    .or(manifest.model.as_deref())
    .or_else(|| read_config_value("integrations.model"))
    .unwrap_or("gpt-4o-mini");
```

### 4. Add Progress Feedback
Improve user experience with progress indicators:

```rust
// In agent_loop.rs:
println!("🤖 Starting agent loop...");
println!("📋 Goal: {}", options.goal);
println!("🔧 Model: {}", model);
println!("🔄 Max iterations: {}", max_iterations);
println!();

// Add progress bars for long-running operations
let progress = indicatif::ProgressBar::new(max_iterations as u64);
```

### 5. Enhance Memory Integration
Make agent loop automatically record learnings:

```rust
// After each iteration:
if !dry_run {
    agent.distill(DistillOptions {
        goal: options.goal,
        outcome: if success { "success" } else { "failed" },
        detail: Some(&iteration_summary),
        elements: Some(&affected_elements),
        ..Default::default()
    })?;
}
```

## Testing Results

### Provider Performance
| Provider | Model | Plan Generation | Apply Execution | Notes |
|----------|-------|-----------------|-----------------|-------|
| Z.AI | glm-4-flash | ✅ Works | ✅ Works | Fast, deterministic |
| XIMIMO | mimo-v2.5-pro | ✅ Works | ✅ Works | Good quality |
| OpenRouter | google/gemini-2.5-flash | ✅ Works | ✅ Works | Best quality |
| OpenRouter | anthropic/claude-sonnet-4 | ✅ Works | ✅ Works | High quality |
| OpenRouter | meta-llama/llama-4-maverick | ✅ Works | ✅ Works | Good quality |

### Task Completion
| Task | Plan Quality | Apply Success | Verification | Notes |
|------|--------------|---------------|--------------|-------|
| Add CLI comment | High | ✅ | ✅ | Deterministic plan |
| Architecture analysis | High | ✅ | ✅ | Comprehensive evidence |
| Code refactoring | Medium | ❌ | ✅ | No code modification support |

## Conclusion

The sruja agent system has a solid foundation with:
- Multi-provider LLM support
- Deterministic plan/apply workflow
- Architecture-aware verification
- Comprehensive memory system

To work as a complete AI agent efficiently, it needs:
1. Unified configuration system
2. Code modification support in apply
3. Better agent loop integration
4. Progress feedback
5. Automatic memory recording

The agent is currently best suited for:
- Architecture analysis and verification
- Plan generation for coding tasks
- Learning recording and retrieval
- Compliance checking

For autonomous code writing, it needs integration with external coding agents or extension of the apply command to support file modifications.
