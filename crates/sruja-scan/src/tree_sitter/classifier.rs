use crate::graph::NodeKind;
use crate::tree_sitter::languages::{DefinitionKind, ParsedFile};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A context for classification, containing all gathered evidence.
pub struct ClassificationContext<'a> {
    pub path_str: String,
    pub content_lower: String,
    #[allow(dead_code)]
    pub parsed: &'a ParsedFile,
}

/// Configuration for the classification engine, allowing for custom rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationConfig {
    pub rules: Vec<CustomRuleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRuleConfig {
    pub name: String,
    pub weight: i32,
    pub kind: NodeKind,
    pub match_patterns: MatchPatterns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPatterns {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_ends_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_contains: Option<Vec<String>>,
}

/// A signal represents a single piece of evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub name: String,
    pub weight: i32,
    pub kind: NodeKind,
}

pub struct ClassificationEngine {
    rules: Vec<Rule>,
}

struct Rule {
    signal: Signal,
    matcher: Box<dyn Fn(&ClassificationContext) -> bool + Send + Sync>,
}

impl ClassificationEngine {
    pub fn new() -> Self {
        let mut engine = Self { rules: Vec::new() };
        engine.register_default_rules();
        engine
    }

    pub fn from_config(config: ClassificationConfig) -> Self {
        let mut engine = Self::new();
        for custom_rule in config.rules {
            engine.register_custom_rule(custom_rule);
        }
        engine
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ClassificationConfig = serde_yaml::from_str(&content)?;
        Ok(Self::from_config(config))
    }

    fn add_rule<F>(&mut self, name: &'static str, weight: i32, kind: NodeKind, matcher: F)
    where
        F: Fn(&ClassificationContext) -> bool + Send + Sync + 'static,
    {
        self.rules.push(Rule {
            signal: Signal {
                name: name.to_string(),
                weight,
                kind,
            },
            matcher: Box::new(matcher),
        });
    }

    fn register_custom_rule(&mut self, config: CustomRuleConfig) {
        let patterns = config.match_patterns;
        let matcher = move |ctx: &ClassificationContext| {
            if let Some(ref suffix) = patterns.path_ends_with {
                if !ctx.path_str.ends_with(suffix) {
                    return false;
                }
            }
            if let Some(ref substring) = patterns.path_contains {
                if !ctx.path_str.contains(substring) {
                    return false;
                }
            }
            if let Some(ref content_substrings) = patterns.content_contains {
                if !content_substrings
                    .iter()
                    .all(|s| ctx.content_lower.contains(&s.to_lowercase()))
                {
                    return false;
                }
            }
            true
        };

        self.rules.push(Rule {
            signal: Signal {
                name: config.name,
                weight: config.weight,
                kind: config.kind,
            },
            matcher: Box::new(matcher),
        });
    }

    pub fn classify(&self, ctx: &ClassificationContext) -> (NodeKind, u8, Vec<Signal>) {
        let mut scores: std::collections::HashMap<NodeKind, i32> = std::collections::HashMap::new();
        let mut triggered_signals = Vec::new();

        for rule in &self.rules {
            if (rule.matcher)(ctx) {
                *scores.entry(rule.signal.kind.clone()).or_insert(0) += rule.signal.weight;
                triggered_signals.push(rule.signal.clone());
            }
        }

        // Find the kind with the highest score
        let (winner_kind, max_score) = scores
            .into_iter()
            .max_by_key(|&(_, score)| score)
            .unwrap_or((NodeKind::Module, 0));

        // Normalize confidence to 0-100
        let confidence = (max_score.clamp(0, 100) as u8).max(10);

        (winner_kind, confidence, triggered_signals)
    }

    fn register_default_rules(&mut self) {
        // Deployment Configs
        self.add_rule("dockerfile", 100, NodeKind::Service, |ctx| {
            ctx.path_str.ends_with("dockerfile")
        });
        self.add_rule("k8s_manifest", 90, NodeKind::Service, |ctx| {
            (ctx.path_str.contains("deployment.") || ctx.path_str.contains("docker-compose."))
                && (ctx.path_str.ends_with(".yaml") || ctx.path_str.ends_with(".yml"))
        });

        // Go Main/Service
        self.add_rule("go_main_server", 85, NodeKind::Service, |ctx| {
            let has_main_func = ctx
                .parsed
                .definitions
                .iter()
                .any(|d| d.name == "main" && d.kind == DefinitionKind::Function);
            ctx.path_str.ends_with(".go")
                && has_main_func
                && ["http.", "grpc.", "serve", "listen"]
                    .iter()
                    .any(|s| ctx.content_lower.contains(s))
        });

        // Rust Web Service
        self.add_rule("rust_web_server", 90, NodeKind::Service, |ctx| {
            let has_main = ctx
                .parsed
                .definitions
                .iter()
                .any(|d| d.name == "main" && d.kind == DefinitionKind::Function);
            ctx.path_str.ends_with(".rs")
                && has_main
                && [
                    "axum::",
                    "actix_web::",
                    "rocket::",
                    "warp::",
                    "tonic::",
                    "serve(",
                ]
                .iter()
                .any(|s| ctx.content_lower.contains(s))
        });

        // Java Spring
        self.add_rule("java_spring_boot", 95, NodeKind::Service, |ctx| {
            let has_class = ctx
                .parsed
                .definitions
                .iter()
                .any(|d| d.kind == DefinitionKind::Class);
            ctx.path_str.ends_with(".java")
                && has_class
                && ["@springbootapplication", "@restcontroller"]
                    .iter()
                    .any(|s| ctx.content_lower.contains(s))
        });

        // JS/TS Server
        self.add_rule("js_express_app", 90, NodeKind::Service, |ctx| {
            (ctx.path_str.ends_with(".js") || ctx.path_str.ends_with(".ts"))
                && ["express()", "app.listen(", "nestfactory.create"]
                    .iter()
                    .any(|s| ctx.content_lower.contains(s))
        });

        // Python Web
        self.add_rule("python_web_framework", 90, NodeKind::Service, |ctx| {
            ctx.path_str.ends_with(".py")
                && ["flask(", "fastapi(", "django"]
                    .iter()
                    .any(|s| ctx.content_lower.contains(s))
        });

        // Databases & Storage
        self.add_rule("prisma_schema", 100, NodeKind::Database, |ctx| {
            ctx.path_str.ends_with(".prisma")
        });
        self.add_rule("redis_client", 80, NodeKind::Database, |ctx| {
            ["redis.createclient", "ioredis", "redis-go", "redis-py"]
                .iter()
                .any(|s| ctx.content_lower.contains(s))
        });
        self.add_rule("postgres_driver", 70, NodeKind::Database, |ctx| {
            ["pg.", "postgres.", "sqlx", "gorm"]
                .iter()
                .any(|s| ctx.content_lower.contains(s))
                && ctx.content_lower.contains("connect")
        });
        self.add_rule("orm_entity", 70, NodeKind::Database, |ctx| {
            [
                "mongoose.model",
                "sequelize.define",
                "@entity",
                "drizzle-orm",
            ]
            .iter()
            .any(|s| ctx.content_lower.contains(s))
        });

        // Queues & Messaging
        self.add_rule("kafka_client", 90, NodeKind::Queue, |ctx| {
            ["kafkajs", "confluent", "sarama", "aiokafka"]
                .iter()
                .any(|s| ctx.content_lower.contains(s))
        });
        self.add_rule("rabbitmq_client", 90, NodeKind::Queue, |ctx| {
            ["amqp", "pika", "stomp"]
                .iter()
                .any(|s| ctx.content_lower.contains(s))
        });

        // External APIs & Integrations
        self.add_rule("stripe_integration", 90, NodeKind::ExternalApi, |ctx| {
            ctx.content_lower.contains("stripe")
                && (ctx.content_lower.contains("checkout") || ctx.content_lower.contains("payment"))
        });

        // CLI Exclusion (Negative signals or early exit)
        self.add_rule("cli_tool_match", -100, NodeKind::Module, |ctx| {
            ["cobra.", "commander", "yargs", "argparse", "click."]
                .iter()
                .any(|s| ctx.content_lower.contains(s))
                || (ctx.path_str.contains("/cmd/")
                    && !ctx.path_str.contains("/cmd/server")
                    && (ctx.content_lower.contains("flag.") || ctx.content_lower.contains("args")))
        });
    }
}

impl Default for ClassificationEngine {
    fn default() -> Self {
        Self::new()
    }
}
