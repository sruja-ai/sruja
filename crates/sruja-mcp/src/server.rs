//! MCP HTTP Server

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use sruja_graph::KnowledgeGraph;

use crate::{
    ApiResponse, ArchitectureSummary, DecisionResponse, PolicyViolationResponse, QueryResponse,
};

#[derive(Clone)]
pub struct AppState {
    pub graph: Arc<RwLock<KnowledgeGraph>>,
}

pub struct McpServer {
    graph: Arc<RwLock<KnowledgeGraph>>,
    port: u16,
}

impl McpServer {
    pub fn new(graph: Arc<RwLock<KnowledgeGraph>>) -> Self {
        Self { graph, port: 3000 }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let state = AppState { graph: self.graph };

        let app = Router::new()
            .route("/health", get(health))
            .route("/architecture", get(get_architecture))
            .route("/decisions", get(get_decisions))
            .route("/decision/:id", get(get_decision))
            .route("/policies", get(get_policies))
            .route("/policy/conflicts", get(get_policy_conflicts))
            .route("/query", post(query))
            .route("/stats", get(get_stats))
            .with_state(state)
            .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        tracing::info!("MCP server listening on {}", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health() -> impl IntoResponse {
    Json(ApiResponse::success(serde_json::json!({ "status": "ok" })))
}

async fn get_architecture(State(state): State<AppState>) -> impl IntoResponse {
    let graph = state.graph.read().await;
    let summary = ArchitectureSummary::from(&*graph);
    Json(ApiResponse::success(summary))
}

async fn get_decisions(State(state): State<AppState>) -> impl IntoResponse {
    let graph = state.graph.read().await;
    let decisions: Vec<DecisionResponse> = graph
        .decisions
        .values()
        .map(|d| DecisionResponse::from(d))
        .collect();
    Json(ApiResponse::success(decisions))
}

async fn get_decision(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let graph = state.graph.read().await;
    match graph.get_decision(&id) {
        Some(d) => Json(ApiResponse::success(DecisionResponse::from(d))),
        None => Json(ApiResponse::<DecisionResponse>::error(format!(
            "Decision {} not found",
            id
        ))),
    }
}

async fn get_policies(State(state): State<AppState>) -> impl IntoResponse {
    let graph = state.graph.read().await;
    let policies: Vec<serde_json::Value> = graph
        .policies
        .values()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();
    Json(ApiResponse::success(policies))
}

async fn get_policy_conflicts(State(state): State<AppState>) -> impl IntoResponse {
    let graph = state.graph.read().await;
    let violations = graph.find_policy_violations();
    let responses: Vec<PolicyViolationResponse> = violations
        .into_iter()
        .map(PolicyViolationResponse::from)
        .collect();
    Json(ApiResponse::success(responses))
}

#[derive(Deserialize)]
struct QueryRequest {
    question: String,
}

async fn query(State(state): State<AppState>, Json(req): Json<QueryRequest>) -> impl IntoResponse {
    let graph = state.graph.read().await;
    match graph.query(&req.question) {
        Ok(result) => Json(ApiResponse::success(QueryResponse::from(result))),
        Err(e) => Json(ApiResponse::<QueryResponse>::error(e.to_string())),
    }
}

async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let graph = state.graph.read().await;
    let stats = graph.stats();
    Json(ApiResponse::success(stats))
}

pub async fn run_server(
    graph: Arc<RwLock<KnowledgeGraph>>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    McpServer::new(graph).port(port).run().await
}
