//! Main application component
//!
//! Slack-inspired 3-panel layout:
//! - Left sidebar: sessions, workspace, agents
//! - Main: chat
//! - Right panel: extracted decisions, query results

use dioxus::prelude::*;
use sruja_chat::{persistence, ChatServer, SessionInfo};
use sruja_extract::Extraction;
use sruja_graph::QueryResult;
use std::sync::Arc;

use crate::components::{ChatPanel, DecisionPanel, Header, Sidebar, Toolbar};

/// Global server context
#[derive(Clone)]
pub struct ServerContext(pub Arc<ChatServer>);

#[component]
pub fn App() -> Element {
    let server_resource = use_resource(|| async move {
        ChatServer::with_persistence(persistence::default_data_dir())
            .await
            .unwrap_or_else(|_| ChatServer::new())
    });

    let server = match &*server_resource.read_unchecked() {
        Some(srv) => Some(Arc::new(srv.clone())),
        None => None,
    };

    if let Some(srv) = server {
        use_context_provider(|| ServerContext(srv));
        rsx! {
            AppBody {}
        }
    } else {
        rsx! {
            div {
                class: "app app-loading",
                "Loading..."
            }
        }
    }
}

#[component]
fn AppBody() -> Element {
    // Shared state
    let mut extractions = use_signal(Vec::<Extraction>::new);
    let session_id = use_signal(|| String::new());
    let participant_id = use_signal(|| String::new());
    let query_result = use_signal(|| Option::<QueryResult>::None);
    let sessions = use_signal(Vec::<SessionInfo>::new);
    let agent_definitions = use_signal(Vec::<sruja_chat::AgentDefinition>::new);
    let load_status = use_signal(|| Option::<String>::None);

    let ctx = use_context::<ServerContext>();
    let server = ctx.0.clone();
    let server_restore = server.clone();
    let server_save = server.clone();
    let server_list = server.clone();
    let server_ex = server.clone();

    // Restore workspace on mount: load last session
    use_effect(move || {
        let srv = server_restore.clone();
        let mut sid = session_id;
        let mut pid = participant_id;
        spawn(async move {
            if let Ok(ws) = srv.load_workspace().await {
                if let Some(ref sid_str) = ws.last_session_id {
                    sid.set(sid_str.clone());
                    if let Ok(parts) = srv.get_participants(sid_str).await {
                        if let Some(p) = parts.first() {
                            pid.set(p.id.clone());
                        }
                    }
                }
            }
        });
    });

    // Persist last selected session to workspace so it can be restored on next launch.
    use_effect(move || {
        let sid = session_id.read().clone();
        if sid.is_empty() {
            return;
        }
        let srv = server_save.clone();
        spawn(async move {
            if let Ok(mut ws) = srv.load_workspace().await {
                ws.last_session_id = Some(sid);
                let _ = srv.save_workspace(&ws).await;
            }
        });
    });

    use_effect(move || {
        let srv = server_list.clone();
        let mut sess_list = sessions;
        let mut def_list = agent_definitions;
        spawn(async move {
            sess_list.set(srv.list_sessions().await);
            def_list.set(srv.list_agent_definitions().await);
        });
    });

    // Load extractions when session changes
    use_effect(move || {
        let sid = session_id.read().clone();
        if sid.is_empty() {
            extractions.set(vec![]);
            return;
        }
        let srv = server_ex.clone();
        let mut ex = extractions;
        spawn(async move {
            if let Ok(e) = srv.get_extractions(&sid).await {
                ex.set(e);
            }
        });
    });

    rsx! {
        div {
            class: "app",

            Header {
                title: "Sruja",
                subtitle: "Architecture Intelligence",
            }

            div {
                class: "app-body",

                Sidebar {
                    session_id: session_id,
                    participant_id: participant_id,
                    sessions: sessions,
                    agent_definitions: agent_definitions,
                    load_status: load_status,
                    on_refresh_sessions: move |_| {
                        let srv = server.clone();
                        let mut sess_list = sessions;
                        spawn(async move {
                            sess_list.set(srv.list_sessions().await);
                        });
                    },
                }

                main {
                    class: "main-content",

                    Toolbar {
                        session_id: session_id,
                        sessions: sessions,
                        query_result: query_result,
                    }

                    ChatPanel {
                        session_id: session_id,
                        participant_id: participant_id,
                        extractions: extractions,
                    }
                }

                aside {
                    class: "right-panel",

                    DecisionPanel {
                        session_id: session_id,
                        extractions: extractions,
                    }

                    if let Some(ref qr) = *query_result.read() {
                        section {
                            class: "query-result-section",
                            h3 { "Query Result" }
                            div {
                                class: "query-answer",
                                "{qr.answer}"
                            }
                            div {
                                class: "query-confidence",
                                "Confidence: {((qr.confidence * 100.0) as i32)}%"
                            }
                            if !qr.evidence.is_empty() {
                                div {
                                    class: "query-evidence",
                                    "Evidence:"
                                    for ev in qr.evidence.iter() {
                                        div {
                                            class: "query-evidence-item",
                                            "{ev.excerpt}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        style { {include_str!("../styles.css")} }
    }
}
