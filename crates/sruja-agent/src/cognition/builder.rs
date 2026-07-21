use std::sync::Arc;

use crate::llm::{LlmClient, ModelRouter};
use crate::memory::AgenticMemory;
use crate::tool::{FileGuard, ToolRegistry};
use super::hook::{Hook, HookRegistry};
use super::tool_tracing::ToolCallTracer;
use super::config::AgentConfig;
use super::agent::Agent;
use super::errors::AgentError;

#[derive(Default)]
pub struct AgentBuilder {
    llm: Option<Arc<dyn LlmClient>>,
    tools: ToolRegistry,
    guard: FileGuard,
    hooks: Vec<Box<dyn Hook>>,
    config: AgentConfig,
    repo_root: Option<std::path::PathBuf>,
    memory: Option<std::sync::Arc<dyn crate::memory::Memory + Send + Sync>>,
    #[cfg(feature = "mcp-client")]
    mcp_manager: Option<crate::tool::mcp::McpClientManager>,
    tool_call_tracer: Option<Box<dyn ToolCallTracer>>,
    trace_run_id: Option<String>,
    trace_id: Option<String>,
    preloaded_files: std::collections::HashMap<String, String>,
    preloaded_arch_context: String,
}

impl AgentBuilder {
    /// Set the LLM client (the brain).
    pub fn llm(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Set the tool registry (the hands).
    pub fn tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = tools;
        self
    }

    /// Attach the file guard (created automatically if not set).
    pub fn guard(mut self, guard: FileGuard) -> Self {
        self.guard = guard;
        self
    }

    /// Register a lifecycle hook.
    pub fn hook(mut self, hook: Box<dyn Hook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Set the agent configuration.
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable memory: provide the repo root where `.sruja/agent_memory.json` lives.
    ///
    /// This constructs the default in-memory backend. For indexed search
    /// (FTS5+BM25), use [`memory_backend`] instead.
    pub fn memory(mut self, repo_root: impl Into<std::path::PathBuf>) -> Self {
        let repo = repo_root.into();
        let mem = crate::memory::AgenticMemory::load(&repo).unwrap_or_default();
        self.memory = Some(std::sync::Arc::new(std::sync::Mutex::new(mem)));
        self.repo_root = Some(repo);
        self
    }

    /// Set a custom memory backend (e.g. FTS5+BM25 indexed search).
    ///
    /// The `repo_root` is used for resolving `.sruja/` paths (decisions,
    /// runbooks, and as the write target for memory persistence).
    pub fn memory_backend(
        mut self,
        repo_root: impl Into<std::path::PathBuf>,
        backend: std::sync::Arc<dyn crate::memory::Memory + Send + Sync>,
    ) -> Self {
        self.repo_root = Some(repo_root.into());
        self.memory = Some(backend);
        self
    }

    /// Register MCP tools from a loop manifest.
    ///
    /// Connects to all enabled MCP servers, lists their tools,
    /// and registers them with the tool registry. Returns a
    /// future that resolves on successful tool registration.
    ///
    /// This is an async builder step; await the future before calling `build`.
    #[cfg(feature = "mcp-client")]
    pub async fn with_mcp(
        mut self,
        manifest: &crate::manifest::LoopManifest,
        repo_root: impl Into<std::path::PathBuf>,
    ) -> Result<Self, AgentError> {
        use crate::tool::mcp::McpClientManager;

        let repo_root = repo_root.into();
        let (manager, mcp_tools) = McpClientManager::from_manifest(manifest, &repo_root)
            .await
            .map_err(|e| AgentError::Mcp(format!("initialization failed: {}", e)))?;

        for tool in mcp_tools {
            self.tools.register(tool);
        }

        self.mcp_manager = Some(manager);
        Ok(self)
    }

    /// Set trace context for tool-call event attribution (U5).
    ///
    /// When all three are provided and `config.enable_tool_call_tracing` is
    /// true, every agent->tool dispatch emits `tool_call`/`tool_result`
    /// context events to `context_events.jsonl`.
    pub fn trace_context(mut self, run_id: impl Into<String>, trace_id: impl Into<String>) -> Self {
        self.trace_run_id = Some(run_id.into());
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Set the tool-call tracer for context event attribution (U5).
    ///
    /// The tracer is called before and after every tool dispatch when
    /// `config.enable_tool_call_tracing` is true and trace context is set.
    pub fn tool_call_tracer(mut self, tracer: Box<dyn ToolCallTracer>) -> Self {
        self.tool_call_tracer = Some(tracer);
        self
    }

    /// Set pre-loaded target file contents.
    ///
    /// When `--file <path>` is specified on the CLI, the file is read once
    /// and injected into the comprehension user prompt. This eliminates
    /// redundant `file_read` tool calls that models often repeat on the
    /// same file in small chunks.
    pub fn preloaded_files(mut self, files: std::collections::HashMap<String, String>) -> Self {
        self.preloaded_files = files;
        self
    }

    /// Set pre-loaded architecture context.
    ///
    /// When architecture context (repomap, topology) is pre-loaded, it's
    /// injected into the comprehension prompt so the agent doesn't need to
    /// call MCP tools for basic architecture context. Saves tokens and
    /// makes the agent more efficient.
    pub fn preloaded_arch_context(mut self, context: String) -> Self {
        self.preloaded_arch_context = context;
        self
    }

    /// Build the agent.
    pub fn build(self) -> Result<Agent, AgentError> {
        let llm_arc = self.llm.ok_or(AgentError::NoLlm)?;

        // Wrap in ModelRouter if a spend cap is configured.
        let llm: Arc<dyn LlmClient> = if let Some(cap) = self.config.spend_cap_usd {
            let rc = crate::llm::router::RouterConfig {
                spend_cap_usd: Some(cap),
                ..Default::default()
            };
            Arc::new(ModelRouter::with_config(llm_arc, rc))
        } else {
            llm_arc
        };

        // Wrap in circuit breaker for per-model failure detection and
        // fast-fail. This prevents cascading failures when a provider is
        // unhealthy — the circuit opens after 3 consecutive failures for
        // a model and rejects further calls for 30s.
        let llm: Arc<dyn LlmClient> = Arc::new(crate::llm::CircuitBreakerClient::new(llm));

        // Wire the guard and dry_run into the tools.
        let mut tools = self.tools;
        tools.set_guard(self.guard.clone());
        if self.config.dry_run {
            tools.set_dry_run(true);
        }

        // Memory: use the provided backend, or fall back to in-memory JSON.
        let memory = self.memory.or_else(|| {
            self.repo_root.as_ref().map(|repo| {
                let mem = AgenticMemory::load(repo).unwrap_or_default();
                std::sync::Arc::new(std::sync::Mutex::new(mem))
                    as std::sync::Arc<dyn crate::memory::Memory + Send + Sync>
            })
        });

        #[cfg(feature = "mcp-client")]
        let mcp_manager = self.mcp_manager;

        Ok(Agent {
            llm,
            tools,
            guard: self.guard,
            hooks: HookRegistry::new(self.hooks),
            config: self.config,
            repo_root: self.repo_root,
            memory,
            #[cfg(feature = "mcp-client")]
            mcp_manager,
            tool_call_tracer: self.tool_call_tracer,
            trace_run_id: self.trace_run_id,
            trace_id: self.trace_id,
            preloaded_files: self.preloaded_files,
            preloaded_arch_context: self.preloaded_arch_context,
        })
    }
}
