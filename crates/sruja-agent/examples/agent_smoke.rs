//! Smoke test: run the agent against a real LLM.
//!
//! Easiest way to configure:
//!   sruja agent setup
//!
//! Or set env vars manually:
//!
//!   # OpenRouter
//!   export OPENAI_API_KEY="sk-or-..."
//!   export OPENAI_BASE_URL="https://openrouter.ai/api/v1"
//!   export OPENAI_MODEL="anthropic/claude-sonnet-4"
//!   cargo run --example agent_smoke -p sruja-agent
//!
//!   # z.ai (Zhipu / BigModel)
//!   export OPENAI_API_KEY="your-zai-key"
//!   export OPENAI_BASE_URL="https://open.bigmodel.cn/api/paas/v4"
//!   export OPENAI_MODEL="glm-4-flash"
//!   cargo run --example agent_smoke -p sruja-agent

use std::sync::Arc;

use sruja_agent::{
    llm::{LlmClient, OpenAiClient},
    tool::{builtin::tools, ToolRegistry},
    Agent, AgentConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build LLM client from env vars.
    let llm = OpenAiClient::from_env()?;
    println!(
        "LLM: {} @ {}",
        llm.default_model(),
        std::env::var("OPENAI_BASE_URL").unwrap_or_default()
    );

    // 2. Register built-in tools (read-only for this smoke test).
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(tools::FileRead::new()));
    tools.register(Box::new(tools::Glob::new()));
    tools.register(Box::new(tools::Grep::new()));

    // 3. Build agent (no memory, no hooks — just the basics).
    let agent = Agent::builder()
        .llm(Arc::new(llm))
        .tools(tools)
        .config(AgentConfig {
            review_every_change: false, // skip critique for speed
            ..Default::default()
        })
        .build()?;

    // 4. Run a simple comprehension task.
    let goal =
        "List all Rust source files in the current directory and summarize what this crate does.";
    println!("\nGoal: {goal}\n");

    let comprehension = agent.comprehend(goal).await?;

    println!("=== Comprehension ===");
    println!("{}", comprehension.summary);
    println!("\nCited elements: {:?}", comprehension.cited_elements);
    println!(
        "Tokens used: {} prompt + {} completion = {} total",
        comprehension.usage.prompt_tokens,
        comprehension.usage.completion_tokens,
        comprehension.usage.total_tokens,
    );

    // 5. Generate a plan.
    println!("\n=== Planning ===");
    let plan = agent.plan(goal, &comprehension).await?;

    println!("Subtasks ({} total):", plan.subtasks.len());
    for st in &plan.subtasks {
        println!(
            "  [{}] {} ({:?}, {:?})",
            st.id, st.description, st.tier, st.kind
        );
    }
    if !plan.risks.is_empty() {
        println!("Risks: {:?}", plan.risks);
    }

    println!("\nSmoke test passed.");
    Ok(())
}
