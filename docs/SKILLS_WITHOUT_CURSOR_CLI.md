# Using Sruja Skills Without Cursor CLI

Sruja **skills** (e.g. `sruja-architecture-agent`) are instructions for an **AI agent**: they define how to discover architecture, map C4 levels, ask questions instead of guessing, and generate valid `.sruja` files. The Sruja **CLI** by itself does **not** read or execute skills—it only does **static analysis** (scan, drift, quickstart, why, discover context).

So:

- **With Cursor CLI:** The `agent` loads the skill and uses it while generating architecture. You get skill-guided behavior (contextual questions, C4 mapping, ask-don’t-guess).
- **Without Cursor CLI:** You only get static analysis unless you use one of the options below to “feed” the skill to an AI.

## Options to Use the Skill Without Cursor CLI

### 1. Cursor CLI (recommended if you use Cursor)

Install the skill and run the agent in the repo:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent
cd /path/to/your/repo
agent -p "Use sruja-architecture-agent. Run sruja discover --context -r . then list 2–3 contextual questions, then generate architecture.sruja and run sruja lint until it passes."
```

See [LOCAL_CURSOR_CLI_TESTING.md](../evaluation/real-world-test/LOCAL_CURSOR_CLI_TESTING.md) and `run_discovery_agent_test.sh`.

### 2. Sruja CLI: generate a prompt file (any LLM)

The CLI can **assemble the skill + repo context** into a single prompt file. You then send that prompt to **any** LLM (OpenAI, Anthropic, local model, etc.) and save the model output as `architecture.sruja`, then run `sruja lint`.

```bash
# From the repo you want to capture (or pass -r)
sruja generate -r . --prompt-only -o capture_prompt.txt

# Then use your LLM (example: OpenAI-style API)
# Paste capture_prompt.txt into your LLM tool, or:
#   curl ... (your API) with content from capture_prompt.txt
# Save the model’s reply (DSL only) to architecture.sruja

sruja lint architecture.sruja
```

- **Skill path:** Use `--skill-path /path/to/sruja-architecture-agent/SKILL.md`, or set `SRUJA_SKILL_PATH`, or leave default (see `sruja generate --help`).
- The prompt file contains: full skill text + repo context (from `sruja discover --context`) + instructions to output only valid Sruja DSL and list open questions if uncertain.
- No API key is required for `generate --prompt-only`; you use your own LLM however you like.

### 3. Another agent (OpenCode, etc.)

Any editor or CLI that can load the same skill format can use Sruja skills. Copy the skill into the repo or install it globally (if the tool supports that), then run the agent with a prompt that references the skill. See [LOCAL_OPENCODE_CLI_TESTING.md](../evaluation/real-world-test/LOCAL_OPENCODE_CLI_TESTING.md) for an OpenCode example.

### 4. MCP + any MCP client

Sruja’s MCP server exposes tools (e.g. run analyze, drift, scan). An MCP client (Cursor or another) can call those tools. The **skill** still has to be loaded by the client so the AI follows Sruja’s rules when generating or editing `.sruja` files. So “without Cursor CLI” you’d need another MCP client that also supports loading Sruja’s skill (e.g. as system prompt or context).

## Summary

| Goal                         | Use |
|-----------------------------|-----|
| Static analysis only        | `sruja quickstart`, `sruja drift`, `sruja scan`, `sruja why`, `sruja discover --context` (no skill, no agent). |
| Skill-guided capture        | Cursor CLI + skill, or `sruja generate --prompt-only` + your LLM, or another agent that loads the skill. |
| Same rules as in the skill  | The prompt produced by `sruja generate --prompt-only` contains the full skill text so any LLM can follow it. |

So **without Cursor CLI** you can still use Sruja skills by running `sruja generate -r . --prompt-only -o prompt.txt` and then using that prompt with any LLM to produce `architecture.sruja`, then validating with `sruja lint`.
