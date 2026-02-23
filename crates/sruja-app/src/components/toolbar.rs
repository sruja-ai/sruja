//! Compact toolbar: architecture query bar (Ask) and session header actions.

use crate::app::ServerContext;
use dioxus::prelude::*;
use sruja_chat::SessionInfo;
use sruja_graph::QueryResult;
use std::sync::Arc;

#[component]
pub fn Toolbar(
    session_id: Signal<String>,
    sessions: Signal<Vec<SessionInfo>>,
    query_result: Signal<Option<QueryResult>>,
) -> Element {
    let server = use_context::<ServerContext>();
    let server_clone: Arc<sruja_chat::ChatServer> = server.0.clone();
    let mut query_input = use_signal(String::new);

    rsx! {
        div {
            class: "toolbar",

            div {
                class: "toolbar-session-info",
                if let Some(s) = sessions.read().iter().find(|s| s.id == *session_id.read()) {
                    span { class: "toolbar-session-topic", "{s.topic}" }
                    span { class: "toolbar-session-meta", "{s.message_count} messages · {s.participant_count} participants" }
                } else {
                    span { class: "toolbar-session-topic", "Select a session" }
                }
            }

            div {
                class: "toolbar-query",
                input {
                    r#type: "text",
                    placeholder: "Ask: Why did we choose Kafka?",
                    value: "{query_input}",
                    oninput: move |e: Event<FormData>| *query_input.write() = e.value(),
                    onkeypress: {
                        let server = server_clone.clone();
                        let mut qr = query_result;
                        move |e: Event<KeyboardData>| {
                            if e.key() == Key::Enter && !query_input.read().is_empty() {
                                let server = server.clone();
                                let q = query_input.read().clone();
                                query_input.write().clear();
                                spawn(async move {
                                    let graph = server.graph();
                                    let g = graph.read().await;
                                    match g.query(&q) {
                                        Ok(result) => qr.set(Some(result)),
                                        Err(_) => qr.set(None),
                                    }
                                });
                            }
                        }
                    },
                    class: "toolbar-query-input",
                }
                button {
                    class: "toolbar-query-btn",
                    onclick: {
                        let server = server_clone.clone();
                        let mut qr = query_result;
                        move |_| {
                            let q = query_input.read().clone();
                            if q.is_empty() { return; }
                            query_input.write().clear();
                            let server = server.clone();
                            spawn(async move {
                                let graph = server.graph();
                                let g = graph.read().await;
                                match g.query(&q) {
                                    Ok(result) => qr.set(Some(result)),
                                    Err(_) => qr.set(None),
                                }
                            });
                        }
                    },
                    "Ask"
                }
            }
        }
    }
}
