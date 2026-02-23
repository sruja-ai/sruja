//! Left sidebar: sessions (Slack-style channels), load context, agents.
//!
//! Architecture collaboration workspace switcher and session list.

use crate::app::ServerContext;
use crate::components::AdminPanel;
use dioxus::prelude::*;
use sruja_chat::{AgentDefinition, SessionInfo};
use std::path::Path;
use std::sync::Arc;

fn session_id_str(s: &SessionInfo) -> String {
    s.id.clone()
}

#[component]
pub fn Sidebar(
    session_id: Signal<String>,
    participant_id: Signal<String>,
    sessions: Signal<Vec<SessionInfo>>,
    agent_definitions: Signal<Vec<AgentDefinition>>,
    load_status: Signal<Option<String>>,
    on_refresh_sessions: EventHandler<()>,
) -> Element {
    let server = use_context::<ServerContext>();
    let server_clone: Arc<sruja_chat::ChatServer> = server.0.clone();
    let server_auto_load = server_clone.clone();
    let mut repo_path = use_signal(|| ".".to_string());
    let mut did_auto_load = use_signal(|| false);

    // Restore workspace repo path and auto-index on first load
    use_effect(move || {
        if *did_auto_load.read() {
            return;
        }
        did_auto_load.set(true);
        let srv = server_auto_load.clone();
        let mut repo_signal = repo_path;
        let mut status_signal = load_status;
        spawn(async move {
            if let Ok(ws) = srv.load_workspace().await {
                if let Some(ref path) = ws.repo_path {
                    if !path.is_empty() {
                        repo_signal.set(path.clone());
                        let path_buf = Path::new(path);
                        match srv.load_repo_context(path_buf).await {
                            Ok(count) => status_signal.set(Some(format!("Auto-loaded {} items", count))),
                            Err(e) => status_signal.set(Some(format!("Auto-load error: {}", e))),
                        }
                    }
                }
            }
        });
    });
    let mut selected_agent_id = use_signal(String::new);
    let mut admin_open = use_signal(|| false);

    rsx! {
        aside {
            class: "sidebar",

            div {
                class: "sidebar-header",
                img {
                    class: "sidebar-logo",
                    src: asset!("/assets/sruja-logo.png"),
                    alt: "Sruja",
                }
                div {
                    class: "sidebar-workspace",
                    span { class: "sidebar-workspace-name", "Architecture" }
                    span { class: "sidebar-workspace-label", "Workspace" }
                }
            }

            div {
                class: "sidebar-section",
                div {
                    class: "sidebar-section-header",
                    span { class: "sidebar-section-title", "Sessions" }
                    button {
                        class: "sidebar-btn-icon",
                        title: "New session",
                        onclick: {
                            let server = server_clone.clone();
                            let mut sess_list = sessions;
                            let mut sid_sig = session_id;
                            let mut pid_sig = participant_id;
                            let on_refresh = on_refresh_sessions;
                            move |_| {
                                let server = server.clone();
                                spawn(async move {
                                    let sid = server.create_session("Architecture Discussion", "You").await;
                                    if let Ok(participants) = server.get_participants(&sid).await {
                                        if let Some(p) = participants.first() {
                                            pid_sig.set(p.id.clone());
                                        }
                                    }
                                    sid_sig.set(sid.clone());
                                    sess_list.set(server.list_sessions().await);
                                    on_refresh.call(());
                                });
                            }
                        },
                        "+"
                    }
                }
                div {
                    class: "sidebar-sessions",
                    for sess in sessions.read().iter() {
                        button {
                            class: if session_id.read().as_str() == session_id_str(sess) { "sidebar-session-item sidebar-session-active" } else { "sidebar-session-item" },
                            onclick: {
                                let srv = server_clone.clone();
                                let sess_id = session_id_str(sess);
                                move |_| {
                                    session_id.set(sess_id.clone());
                                    let server = srv.clone();
                                    let sid = sess_id.clone();
                                    let mut pid = participant_id;
                                    spawn(async move {
                                        if let Ok(parts) = server.get_participants(&sid).await {
                                            if let Some(p) = parts.first() {
                                                pid.set(p.id.clone());
                                            }
                                        }
                                    });
                                }
                            },
                            span { class: "sidebar-session-icon", "#" }
                            span { class: "sidebar-session-name", "{sess.topic}" }
                            span { class: "sidebar-session-count", "{sess.message_count}" }
                        }
                    }
                }
            }

            div {
                class: "sidebar-section",
                div {
                    class: "sidebar-section-header",
                    span { class: "sidebar-section-title", "Workspace" }
                }
                div {
                    class: "sidebar-load",
                    input {
                        r#type: "text",
                        class: "sidebar-input",
                        placeholder: "Repo path (e.g. .)",
                        value: "{repo_path}",
                        oninput: move |e: Event<FormData>| *repo_path.write() = e.value(),
                    }
                    button {
                        class: "sidebar-btn-primary",
                        onclick: {
                            let server = server_clone.clone();
                            move |_| {
                                let path = repo_path.read().clone();
                                let srv = server.clone();
                                spawn(async move {
                                    let path_buf = Path::new(&path);
                                    match srv.load_repo_context(path_buf).await {
                                        Ok(count) => load_status.set(Some(format!("Loaded {} items", count))),
                                        Err(e) => load_status.set(Some(format!("Error: {}", e))),
                                    }
                                });
                            }
                        },
                        "Load Context"
                    }
                }
                if let Some(ref status) = *load_status.read() {
                    div { class: "sidebar-status", "{status}" }
                }
            }

            div {
                class: "sidebar-section",
                div {
                    class: "sidebar-section-header",
                    span { class: "sidebar-section-title", "Agents" }
                    button {
                        class: "sidebar-btn-icon",
                        title: "Manage agents",
                        onclick: move |_| {
                            let open = *admin_open.read();
                            admin_open.set(!open);
                        },
                        "⚙"
                    }
                }
                div {
                    class: "sidebar-agents",
                    select {
                        class: "sidebar-select",
                        value: "{selected_agent_id.read()}",
                        onchange: move |e: Event<FormData>| selected_agent_id.set(e.value()),
                        option { value: "", "Add agent..." }
                        for d in agent_definitions.read().iter() {
                            option { value: "{d.id}", "{d.name}" }
                        }
                    }
                    button {
                        class: "sidebar-btn-secondary",
                        disabled: selected_agent_id.read().is_empty() || session_id.read().is_empty(),
                        onclick: {
                            let server = server_clone.clone();
                            let mut sess_list = sessions;
                            let sel = selected_agent_id;
                            move |_| {
                                let def_id = sel.read().clone();
                                if def_id.is_empty() { return; }
                                let sid = session_id.read().clone();
                                if sid.is_empty() { return; }
                                let srv = server.clone();
                                spawn(async move {
                                    let _ = srv.join_agent_from_definition(&sid, &def_id).await;
                                    sess_list.set(srv.list_sessions().await);
                                });
                            }
                        },
                        "Add"
                    }
                }
            }

            if *admin_open.read() {
                AdminPanel {
                    agent_definitions: agent_definitions,
                    on_close: move |_| admin_open.set(false),
                }
            }
        }
    }
}
