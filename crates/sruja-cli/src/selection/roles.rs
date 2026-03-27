//! Architectural role detection for components
//!
//! Classifies components by their architectural role to prioritize selection.

use sruja_scan::{Graph, Node, NodeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchitecturalRole {
    /// Main entry points (main.rs, server.ts, app.py, index.ts)
    EntryPoint,
    /// API surface (routes, handlers, controllers)
    ApiSurface,
    /// Data persistence (databases, models, schemas, migrations)
    DataStore,
    /// Integration points with external systems
    IntegrationHub,
    /// Core business logic and domain
    CoreDomain,
    /// Infrastructure (config, logging, middleware)
    Infrastructure,
    /// Generated code (proto, pb, graphql-gen)
    Generated,
    /// Unknown or uncategorized
    Unknown,
}

impl ArchitecturalRole {
    pub fn all() -> Vec<Self> {
        vec![
            Self::EntryPoint,
            Self::ApiSurface,
            Self::DataStore,
            Self::IntegrationHub,
            Self::CoreDomain,
            Self::Infrastructure,
            Self::Generated,
            Self::Unknown,
        ]
    }

    pub fn priority(&self) -> u8 {
        match self {
            Self::EntryPoint => 100,
            Self::ApiSurface => 90,
            Self::DataStore => 90,
            Self::IntegrationHub => 80,
            Self::CoreDomain => 70,
            Self::Infrastructure => 50,
            Self::Generated => 10,
            Self::Unknown => 30,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EntryPoint => "entry_point",
            Self::ApiSurface => "api_surface",
            Self::DataStore => "data_store",
            Self::IntegrationHub => "integration_hub",
            Self::CoreDomain => "core_domain",
            Self::Infrastructure => "infrastructure",
            Self::Generated => "generated",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for ArchitecturalRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detect the architectural role of a node
pub fn detect_architectural_role(node: &Node, graph: &Graph) -> ArchitecturalRole {
    if is_entry_point(node, graph) {
        return ArchitecturalRole::EntryPoint;
    }

    if is_generated(node) {
        return ArchitecturalRole::Generated;
    }

    if is_api_surface(node, graph) {
        return ArchitecturalRole::ApiSurface;
    }

    if is_data_store(node, graph) {
        return ArchitecturalRole::DataStore;
    }

    if is_integration_hub(node, graph) {
        return ArchitecturalRole::IntegrationHub;
    }

    if is_infrastructure(node) {
        return ArchitecturalRole::Infrastructure;
    }

    if is_core_domain(node, graph) {
        return ArchitecturalRole::CoreDomain;
    }

    ArchitecturalRole::Unknown
}

fn is_entry_point(node: &Node, graph: &Graph) -> bool {
    let path = node.path.as_deref().unwrap_or(&node.id);
    let label = node.label.to_lowercase();
    let path_lower = path.to_lowercase();

    if matches!(node.kind, NodeKind::Service) {
        return true;
    }

    let entry_patterns = [
        "main.rs",
        "main.go",
        "main.py",
        "main.js",
        "main.ts",
        "server.ts",
        "server.js",
        "server.py",
        "server.go",
        "app.ts",
        "app.js",
        "app.py",
        "app.go",
        "index.ts",
        "index.js",
        "cmd/",
        "/cmd/",
        "bootstrap",
        "start",
        "run",
    ];

    for pattern in &entry_patterns {
        if path_lower.contains(pattern) || label.contains(pattern) {
            let in_deg = graph.edges.iter().filter(|e| e.target == node.id).count();
            let out_deg = graph.edges.iter().filter(|e| e.source == node.id).count();
            if out_deg >= in_deg {
                return true;
            }
        }
    }

    false
}

fn is_api_surface(node: &Node, graph: &Graph) -> bool {
    let path = node.path.as_deref().unwrap_or(&node.id);
    let label = node.label.to_lowercase();
    let path_lower = path.to_lowercase();

    if matches!(node.kind, NodeKind::ExternalApi) {
        return true;
    }

    let api_patterns = [
        "/api/",
        "/routes/",
        "/handlers/",
        "/controllers/",
        "/endpoints/",
        "/views/",
        "/resources/",
        "controller",
        "handler",
        "route",
        "endpoint",
        "router",
        "view",
        "resource",
        "controller_test",
        "_api",
        "api_",
        "rest_",
        "graphql",
        "@controller",
        "@route",
        "@get",
        "@post",
        "@put",
        "@delete",
        "http_",
        "http handler",
        "servehttp",
    ];

    for pattern in &api_patterns {
        if path_lower.contains(pattern) || label.contains(pattern) {
            return true;
        }
    }

    let has_external_deps = graph.edges.iter().filter(|e| e.source == node.id).any(|e| {
        graph
            .nodes
            .iter()
            .find(|n| n.id == e.target)
            .map(|n| matches!(n.kind, NodeKind::ExternalApi | NodeKind::Service))
            .unwrap_or(false)
    });

    if has_external_deps {
        let label_lower = label.to_lowercase();
        if label_lower.contains("handler") || label_lower.contains("controller") {
            return true;
        }
    }

    false
}

fn is_data_store(node: &Node, _graph: &Graph) -> bool {
    if matches!(node.kind, NodeKind::Database) {
        return true;
    }

    let path = node.path.as_deref().unwrap_or(&node.id);
    let label = node.label.to_lowercase();
    let path_lower = path.to_lowercase();

    let db_patterns = [
        "/db/",
        "/database/",
        "/models/",
        "/model/",
        "/entities/",
        "/entity/",
        "/schema/",
        "/schemas/",
        "/migrations/",
        "/migration/",
        "/repositories/",
        "/repository/",
        "/dao/",
        "/dal/",
        "/persistence/",
        "/store/",
        "database",
        "repository",
        "repo",
        "model",
        "entity",
        "schema",
        "migration",
        "dao",
        "store",
        "sql",
        "query",
        "postgres",
        "mysql",
        "mongodb",
        "redis",
        "dynamodb",
        "firestore",
        "supabase",
        "prisma",
        "sequelize",
        "typeorm",
        "sqlalchemy",
        "diesel",
        "gorm",
        "ent",
        "knex",
        "objection",
    ];

    for pattern in &db_patterns {
        if path_lower.contains(pattern) || label.contains(pattern) {
            return true;
        }
    }

    false
}

fn is_integration_hub(node: &Node, graph: &Graph) -> bool {
    let external_deps: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.source == node.id)
        .filter_map(|e| {
            graph
                .nodes
                .iter()
                .find(|n| n.id == e.target)
                .filter(|n| matches!(n.kind, NodeKind::ExternalApi | NodeKind::Service))
        })
        .collect();

    if external_deps.len() >= 3 {
        return true;
    }

    let path = node.path.as_deref().unwrap_or(&node.id);
    let label = node.label.to_lowercase();
    let path_lower = path.to_lowercase();

    let integration_patterns = [
        "/integrations/",
        "/clients/",
        "/adapters/",
        "/external/",
        "/third-party/",
        "/services/",
        "/connectors/",
        "client",
        "adapter",
        "connector",
        "integration",
        "external",
        "third-party",
        "wrapper",
        "kafka",
        "rabbitmq",
        "sqs",
        "sns",
        "pubsub",
        "stripe",
        "twilio",
        "sendgrid",
        "mailchimp",
        "aws",
        "gcp",
        "azure",
        "cloudflare",
        "oauth",
        "auth0",
        "firebase",
    ];

    for pattern in &integration_patterns {
        if path_lower.contains(pattern) || label.contains(pattern) {
            return true;
        }
    }

    false
}

fn is_generated(node: &Node) -> bool {
    let path = node.path.as_deref().unwrap_or(&node.id);
    let label = node.label.to_lowercase();
    let path_lower = path.to_lowercase();

    let generated_patterns = [
        "/generated/",
        "/gen/",
        "/auto/",
        ".pb.",
        ".pb.go",
        "_pb2",
        "_pb.rs",
        ".grpc.",
        "_grpc.",
        "graphql-gen",
        "gql-gen",
        "generated",
        ".generated.",
        "_generated.",
        "-generated.",
        "node_modules",
        "vendor",
        "target/",
        "dist/",
        "build/",
        ".next/",
        "out/",
    ];

    for pattern in &generated_patterns {
        if path_lower.contains(pattern) || label.contains(pattern) {
            return true;
        }
    }

    false
}

fn is_infrastructure(node: &Node) -> bool {
    let path = node.path.as_deref().unwrap_or(&node.id);
    let label = node.label.to_lowercase();
    let path_lower = path.to_lowercase();

    let infra_patterns = [
        "/config/",
        "/configs/",
        "/configuration/",
        "/logging/",
        "/logger/",
        "/log/",
        "/middleware/",
        "/interceptors/",
        "/filters/",
        "/utils/",
        "/util/",
        "/helpers/",
        "/helper/",
        "/common/",
        "/shared/",
        "/lib/",
        "/libs/",
        "/infrastructure/",
        "/infra/",
        "config",
        "configuration",
        "settings",
        "options",
        "logging",
        "logger",
        "log",
        "tracing",
        "middleware",
        "interceptor",
        "filter",
        "util",
        "utils",
        "helper",
        "helpers",
        "common",
        "shared",
        "lib",
        "di",
        "container",
        "inject",
        "wire",
        "setup",
        "init",
        "bootstrap",
    ];

    for pattern in &infra_patterns {
        if path_lower.contains(pattern) || label.contains(pattern) {
            return true;
        }
    }

    false
}

fn is_core_domain(node: &Node, graph: &Graph) -> bool {
    let path = node.path.as_deref().unwrap_or(&node.id);
    let label = node.label.to_lowercase();
    let path_lower = path.to_lowercase();

    let domain_patterns = [
        "/domain/",
        "/core/",
        "/business/",
        "/logic/",
        "/service/",
        "/services/",
        "/usecase/",
        "/usecases/",
        "/application/",
        "/app/",
        "/features/",
        "/modules/",
        "service",
        "usecase",
        "use_case",
        "interactor",
        "domain",
        "entity",
        "aggregate",
        "valueobject",
        "command",
        "query",
        "handler",
        "executor",
        "manager",
        "processor",
        "engine",
    ];

    for pattern in &domain_patterns {
        if path_lower.contains(pattern) || label.contains(pattern) {
            return true;
        }
    }

    let in_degree = graph.edges.iter().filter(|e| e.target == node.id).count();
    let out_degree = graph.edges.iter().filter(|e| e.source == node.id).count();

    if in_degree > 2 && out_degree > 0 && out_degree < 10 {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeKind};

    fn make_node(id: &str, path: &str, kind: NodeKind) -> Node {
        Node {
            id: id.into(),
            kind,
            label: id.into(),
            technology: None,
            path: Some(path.into()),
            metadata: Default::default(),
            canonical_id: None,
            aliases: Vec::new(),
            owner: None,
            domain: None,
            criticality: None,
            sources: Vec::new(),
            confidence: None,
        }
    }

    fn make_graph(nodes: Vec<Node>, edges: Vec<(String, String)>) -> Graph {
        Graph {
            nodes,
            edges: edges
                .into_iter()
                .map(|(s, t)| Edge {
                    source: s,
                    target: t,
                    kind: EdgeKind::DependsOn,
                    evidence: vec![],
                })
                .collect(),
            metadata: Default::default(),
            confidence: None,
        }
    }

    #[test]
    fn test_entry_point_detection() {
        let nodes = vec![make_node("main", "src/main.rs", NodeKind::Module)];
        let graph = make_graph(nodes, vec![]);

        let role = detect_architectural_role(&graph.nodes[0], &graph);
        assert_eq!(role, ArchitecturalRole::EntryPoint);
    }

    #[test]
    fn test_api_surface_detection() {
        let nodes = vec![make_node(
            "user_controller",
            "src/api/user_controller.rs",
            NodeKind::Module,
        )];
        let graph = make_graph(nodes, vec![]);

        let role = detect_architectural_role(&graph.nodes[0], &graph);
        assert_eq!(role, ArchitecturalRole::ApiSurface);
    }

    #[test]
    fn test_data_store_detection() {
        let nodes = vec![make_node(
            "user_repo",
            "src/repositories/user_repo.rs",
            NodeKind::Module,
        )];
        let graph = make_graph(nodes, vec![]);

        let role = detect_architectural_role(&graph.nodes[0], &graph);
        assert_eq!(role, ArchitecturalRole::DataStore);
    }

    #[test]
    fn test_database_kind() {
        let nodes = vec![make_node("postgres", "postgres", NodeKind::Database)];
        let graph = make_graph(nodes, vec![]);

        let role = detect_architectural_role(&graph.nodes[0], &graph);
        assert_eq!(role, ArchitecturalRole::DataStore);
    }

    #[test]
    fn test_generated_detection() {
        let nodes = vec![make_node(
            "api_pb2",
            "generated/api_pb2.py",
            NodeKind::Module,
        )];
        let graph = make_graph(nodes, vec![]);

        let role = detect_architectural_role(&graph.nodes[0], &graph);
        assert_eq!(role, ArchitecturalRole::Generated);
    }

    #[test]
    fn test_role_priority() {
        assert!(ArchitecturalRole::EntryPoint.priority() > ArchitecturalRole::Generated.priority());
        assert!(
            ArchitecturalRole::ApiSurface.priority() > ArchitecturalRole::Infrastructure.priority()
        );
    }
}
