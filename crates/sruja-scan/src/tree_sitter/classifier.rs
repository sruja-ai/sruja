use crate::graph::NodeKind;
use crate::tree_sitter::languages::ParsedFile;

/// A context for classification, containing all gathered evidence.
pub struct ClassificationContext<'a> {
    pub path_str: String,
    pub name_lower: String,
    pub content_lower: String,
    pub parsed: &'a ParsedFile,
}

/// A signal represents a single piece of evidence.
#[derive(Debug, Clone)]
pub struct Signal {
    pub name: &'static str,
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

    fn add_rule<F>(&mut self, name: &'static str, weight: i32, kind: NodeKind, matcher: F)
    where
        F: Fn(&ClassificationContext) -> bool + Send + Sync + 'static,
    {
        self.rules.push(Rule {
            signal: Signal { name, weight, kind },
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
        self.add_rule("go_main_server", 80, NodeKind::Service, |ctx| {
            ctx.path_str.ends_with(".go") && ctx.path_str.contains("/cmd/") && ctx.name_lower == "main" && 
            ["http.", "grpc.", "serve", "listen"].iter().any(|s| ctx.content_lower.contains(s))
        });

        // Java Spring
        self.add_rule("java_spring_boot", 95, NodeKind::Service, |ctx| {
            ctx.path_str.ends_with(".java") && 
            ["@springbootapplication", "@restcontroller"].iter().any(|s| ctx.content_lower.contains(s))
        });

        // JS/TS Server
        self.add_rule("js_express_app", 90, NodeKind::Service, |ctx| {
            (ctx.path_str.ends_with(".js") || ctx.path_str.ends_with(".ts")) &&
            ["express()", "app.listen(", "nestfactory.create"].iter().any(|s| ctx.content_lower.contains(s))
        });

        // Python Web
        self.add_rule("python_web_framework", 90, NodeKind::Service, |ctx| {
            ctx.path_str.ends_with(".py") &&
            ["flask(", "fastapi(", "django"].iter().any(|s| ctx.content_lower.contains(s))
        });

        // Databases & Storage
        self.add_rule("prisma_schema", 100, NodeKind::Database, |ctx| {
            ctx.path_str.ends_with(".prisma")
        });
        self.add_rule("redis_client", 80, NodeKind::Database, |ctx| {
            ["redis.createclient", "ioredis", "redis-go", "redis-py"].iter().any(|s| ctx.content_lower.contains(s))
        });
        self.add_rule("postgres_driver", 70, NodeKind::Database, |ctx| {
            ["pg.", "postgres.", "sqlx", "gorm"].iter().any(|s| ctx.content_lower.contains(s)) &&
            ctx.content_lower.contains("connect")
        });
        self.add_rule("orm_entity", 70, NodeKind::Database, |ctx| {
            ["mongoose.model", "sequelize.define", "@entity", "drizzle-orm"].iter().any(|s| ctx.content_lower.contains(s))
        });
        self.add_rule("db_directory", 40, NodeKind::Database, |ctx| {
            ctx.path_str.contains("/db/") || ctx.path_str.contains("/database/") || ctx.path_str.contains("/migrations/")
        });

        // Queues & Messaging
        self.add_rule("kafka_client", 90, NodeKind::Queue, |ctx| {
            ["kafkajs", "confluent", "sarama", "aiokafka"].iter().any(|s| ctx.content_lower.contains(s))
        });
        self.add_rule("rabbitmq_client", 90, NodeKind::Queue, |ctx| {
            ["amqp", "pika", "stomp"].iter().any(|s| ctx.content_lower.contains(s))
        });
        self.add_rule("event_bus", 60, NodeKind::Queue, |ctx| {
            ctx.name_lower.contains("eventbus") || ctx.name_lower.contains("pubsub") || ctx.name_lower.contains("queue")
        });

        // External APIs & Integrations
        self.add_rule("nextjs_api_route", 100, NodeKind::ExternalApi, |ctx| {
            ctx.path_str.contains("/pages/api/") || (ctx.path_str.contains("/app/") && ctx.path_str.ends_with("/route.ts"))
        });
        self.add_rule("external_gateway", 60, NodeKind::ExternalApi, |ctx| {
            ctx.path_str.contains("/external/") || ctx.name_lower.ends_with("gateway") || ctx.name_lower.ends_with("client")
        });
        self.add_rule("stripe_integration", 90, NodeKind::ExternalApi, |ctx| {
            ctx.content_lower.contains("stripe") && (ctx.content_lower.contains("checkout") || ctx.content_lower.contains("payment"))
        });

        // CLI Exclusion (Negative signals or early exit)
        self.add_rule("cli_tool_match", -100, NodeKind::Module, |ctx| {
            ["cobra.", "commander", "yargs", "argparse", "click."].iter().any(|s| ctx.content_lower.contains(s)) ||
            (ctx.path_str.contains("/cmd/") && !ctx.path_str.contains("/cmd/server") && (ctx.content_lower.contains("flag.") || ctx.content_lower.contains("args")))
        });
    }
}

impl Default for ClassificationEngine {
    fn default() -> Self {
        Self::new()
    }
}
