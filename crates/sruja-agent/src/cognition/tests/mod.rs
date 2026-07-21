pub(crate) use super::*;
pub(crate) use crate::llm::{CompletionRequest, CompletionResponse, LlmClient, LlmError};
pub(crate) use crate::memory::AgenticMemory;
pub(crate) use crate::tool::ToolRegistry;
pub(crate) use super::parsing::parse_critique_from_response;
pub(crate) use crate::verify::{VerifyOptions, VerifyStep};
use async_trait::async_trait;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

mod checkpoint;
mod complexity;
mod critic;
mod loop_tests;
mod memory;
mod parse;
mod subagent;

// --- Ensemble critic helpers ---

struct DropGuard(Arc<AtomicUsize>);
impl Drop for DropGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

struct PersonaScriptedLlm {
    responses: Vec<PersonaResponse>,
    max_concurrent: Arc<AtomicUsize>,
    active: Arc<AtomicUsize>,
    last_system_prompt: Arc<Mutex<String>>,
    last_user_prompt: Arc<Mutex<String>>,
}

#[derive(Debug, Clone)]
struct PersonaResponse {
    system_prompt_contains: &'static str,
    approved: bool,
    score: f64,
    issues: Vec<String>,
}

impl PersonaScriptedLlm {
    fn new(responses: Vec<PersonaResponse>) -> Arc<Self> {
        Arc::new(Self {
            responses,
            max_concurrent: Arc::new(AtomicUsize::new(0)),
            active: Arc::new(AtomicUsize::new(0)),
            last_system_prompt: Arc::new(Mutex::new(String::new())),
            last_user_prompt: Arc::new(Mutex::new(String::new())),
        })
    }

    #[allow(dead_code)]
    fn max_concurrent(&self) -> usize {
        self.max_concurrent.load(Ordering::SeqCst)
    }

    fn received_system_prompt(&self) -> String {
        self.last_system_prompt.lock().unwrap().clone()
    }

    fn received_user_prompt(&self) -> String {
        self.last_user_prompt.lock().unwrap().clone()
    }
}

#[async_trait]
impl LlmClient for PersonaScriptedLlm {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        let prev_max = self.max_concurrent.load(Ordering::SeqCst);
        if active > prev_max {
            self.max_concurrent.store(active, Ordering::SeqCst);
        }
        let _guard = DropGuard(self.active.clone());

        tokio::task::yield_now().await;

        let sys = req
            .messages
            .first()
            .map(|m| m.content.as_str())
            .unwrap_or("");
        *self.last_system_prompt.lock().unwrap() = sys.to_string();
        let user = req
            .messages
            .get(1)
            .map(|m| m.content.as_str())
            .unwrap_or("");
        *self.last_user_prompt.lock().unwrap() = user.to_string();
        let content = sys
            .lines()
            .find_map(|_l| {
                self.responses
                    .iter()
                    .find(|r| sys.contains(r.system_prompt_contains))
                    .map(|r| {
                        format!(
                            r#"{{"approved":{},"score":{},"issues":{:?},"suggestions":[]}}"#,
                            r.approved, r.score, r.issues
                        )
                    })
            })
            .unwrap_or_else(|| {
                r#"{"approved":false,"score":0.0,"issues":[],"suggestions":[]}"#.to_string()
            });

        Ok(CompletionResponse {
            content,
            tool_calls: Vec::new(),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            model: "scripted-ensemble".into(),
            finish_reason: crate::llm::FinishReason::Stop,
        })
    }

    fn default_model(&self) -> &str {
        "scripted-ensemble"
    }
}

#[test]
fn default_config_is_tdd_and_review() {
    let config = AgentConfig::default();
    assert!(config.tdd);
    assert!(config.review_every_change);
}

#[test]
fn extract_ids_works() {
    let ids = extract_element_ids("See Sruja.CLI and Sruja.Graph.KnowledgeGraph for details.");
    assert!(ids.contains(&"Sruja.CLI".to_string()));
    assert!(ids.contains(&"Sruja.Graph.KnowledgeGraph".to_string()));
}

// --- Loop spine helpers ---

struct ScriptedLlm {
    critique_calls: AtomicUsize,
    reject_first: usize,
}

impl ScriptedLlm {
    fn approve_after(reject_first: usize) -> Arc<Self> {
        Arc::new(Self {
            critique_calls: AtomicUsize::new(0),
            reject_first,
        })
    }
}

#[async_trait]
impl LlmClient for ScriptedLlm {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let sys = req
            .messages
            .first()
            .map(|m| m.content.as_str())
            .unwrap_or("");
        let content = if sys.contains("reviewing a change") {
            let n = self.critique_calls.fetch_add(1, Ordering::SeqCst);
            let approved = n >= self.reject_first;
            if approved {
                r#"{"approved":true,"score":0.9,"issues":[],"suggestions":[]}"#.to_string()
            } else {
                r#"{"approved":false,"score":0.2,"issues":["tests missing"],"suggestions":["add tests"]}"#
                        .to_string()
            }
        } else if sys.contains("decomposing work into concrete subtasks") {
            r#"{"subtasks":[{"id":"s1","description":"implement feature","tier":"mid","kind":"implement","files":[],"acceptance_criteria":["it works"]}],"risks":[]}"#
                    .to_string()
        } else if sys.contains("executing a specific subtask")
            || sys.contains("autonomous coding agent")
        {
            "Done writing the hello module. The file was created successfully and contains the expected content.".to_string()
        } else if sys.contains("understand codebases thoroughly") {
            "Understood the goal.".to_string()
        } else {
            "{}".to_string()
        };

        Ok(CompletionResponse {
            content,
            tool_calls: Vec::new(),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            model: "scripted".into(),
            finish_reason: crate::llm::FinishReason::Stop,
        })
    }

    fn default_model(&self) -> &str {
        "scripted"
    }
}

fn loop_test_agent(llm: Arc<dyn LlmClient>) -> Agent {
    let config = AgentConfig {
        tdd: false,
        ..Default::default()
    };
    Agent::builder()
        .llm(llm)
        .tools(ToolRegistry::new())
        .config(config)
        .build()
        .expect("agent builds")
}

// --- ACT phase helpers ---

struct ActingLlm {
    execute_calls: AtomicUsize,
}

#[async_trait]
impl LlmClient for ActingLlm {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let sys = req
            .messages
            .first()
            .map(|m| m.content.as_str())
            .unwrap_or("");
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        };
        let content = if sys.contains("reviewing a change") {
            r#"{"approved":true,"score":0.9,"issues":[],"suggestions":[]}"#.to_string()
        } else if sys.contains("autonomous coding agent")
            || sys.contains("executing a specific subtask")
        {
            let n = self.execute_calls.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                return Ok(CompletionResponse {
                    content: "Writing the file.".into(),
                    tool_calls: vec![crate::llm::ToolCall {
                        id: "call_1".into(),
                        name: "file_write".into(),
                        arguments: serde_json::json!({
                            "path": "src/hello.rs",
                            "content": "fn main() { println!(\"hello from the loop\"); }\n"
                        }),
                    }],
                    usage: usage.clone(),
                    model: "acting".into(),
                    finish_reason: crate::llm::FinishReason::ToolCalls,
                });
            }
            "Done writing the hello module. The file was created successfully and contains the expected content.".to_string()
        } else if sys.contains("decomposing work into concrete subtasks") {
            r#"{"subtasks":[{"id":"s1","description":"write hello module","tier":"mid","kind":"implement","files":["src/hello.rs"],"acceptance_criteria":["file exists"]}],"risks":[]}"#
                    .to_string()
        } else if sys.contains("understand codebases thoroughly") {
            "Understood.".to_string()
        } else {
            "{}".to_string()
        };

        Ok(CompletionResponse {
            content,
            tool_calls: Vec::new(),
            usage,
            model: "acting".into(),
            finish_reason: crate::llm::FinishReason::Stop,
        })
    }

    fn default_model(&self) -> &str {
        "acting"
    }
}

// --- Guardrail helpers ---

struct StuckLlm;

#[async_trait]
impl LlmClient for StuckLlm {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Ok(CompletionResponse {
            content: String::new(),
            tool_calls: vec![crate::llm::ToolCall {
                id: "t1".into(),
                name: "shell_command".into(),
                arguments: serde_json::json!({"command": "echo stuck"}),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            model: "stuck".into(),
            finish_reason: crate::llm::FinishReason::ToolCalls,
        })
    }

    fn default_model(&self) -> &str {
        "stuck"
    }
}

struct ConvergingLlm {
    call_count: AtomicUsize,
    fail_first: usize,
}

impl ConvergingLlm {
    fn after(fail_first: usize) -> Arc<Self> {
        Arc::new(Self {
            call_count: AtomicUsize::new(0),
            fail_first,
        })
    }
}

#[async_trait]
impl LlmClient for ConvergingLlm {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        };
        if n < self.fail_first {
            Ok(CompletionResponse {
                content: String::new(),
                tool_calls: vec![crate::llm::ToolCall {
                    id: format!("t{n}"),
                    name: "shell_command".into(),
                    arguments: serde_json::json!({"command": "echo pre" }),
                }],
                usage,
                model: "converging".into(),
                finish_reason: crate::llm::FinishReason::ToolCalls,
            })
        } else {
            Ok(CompletionResponse {
                content: "Done! Implemented the feature.".into(),
                tool_calls: vec![],
                usage,
                model: "converging".into(),
                finish_reason: crate::llm::FinishReason::Stop,
            })
        }
    }

    fn default_model(&self) -> &str {
        "converging"
    }
}

// --- Usage ---

#[test]
fn usage_estimated_cost_is_nonzero() {
    let usage = Usage {
        prompt_tokens: 1_000_000,
        completion_tokens: 500_000,
        total_tokens: 1_500_000,
    };
    let cost = usage.estimated_cost_usd();
    assert!(
        (cost - 0.45).abs() < 0.001,
        "expected ~$0.45, got ${cost:.4}"
    );
}

// --- Checkpoint helpers ---

fn test_comprehension() -> Comprehension {
    Comprehension {
        goal: String::new(),
        summary: String::new(),
        cited_elements: Vec::new(),
        key_findings: Vec::new(),
        risks: Vec::new(),
        usage: Usage::default(),
        retrieved_learning_ids: Vec::new(),
        complexity: TaskComplexity::default(),
        pre_conditions: vec![],
    }
}

// --- Sub-agent helpers ---

struct SummarizingLlm;
#[async_trait]
impl LlmClient for SummarizingLlm {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Ok(CompletionResponse {
            content: "Reviewed MySystem.ApiContainer: it depends on MySystem.Database. \
                      No unused imports found; change is safe."
                .to_string(),
            tool_calls: Vec::new(),
            usage: Usage {
                prompt_tokens: 5,
                completion_tokens: 5,
                total_tokens: 10,
            },
            model: "scripted".into(),
            finish_reason: crate::llm::FinishReason::Stop,
        })
    }
    fn default_model(&self) -> &str {
        "scripted"
    }
}

fn isolated_agent() -> Agent {
    Agent::builder()
        .llm(Arc::new(SummarizingLlm))
        .tools(ToolRegistry::new())
        .config(AgentConfig::default())
        .build()
        .expect("agent builds")
}
