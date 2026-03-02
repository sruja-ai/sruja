//! Chat panel component
//!
//! Main thread shows top-level messages. Each message can have a thread; key decisions
//! and summaries from child threads surface in the main channel.

use crate::app::ServerContext;
use dioxus::prelude::*;
use sruja_chat::{Message, NewMessage};
use sruja_extract::{ExtractedContent, Extraction};
use std::sync::Arc;

fn avatar_initials(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphabetic())
        .take(2)
        .flat_map(|c| c.to_uppercase())
        .collect()
}

fn extraction_title(ext: &Extraction) -> String {
    match &ext.content {
        ExtractedContent::Decision { title, .. } => title.clone(),
        ExtractedContent::Requirement { title, .. } => title.clone(),
        ExtractedContent::Constraint { source, target, .. } => format!("{source} → {target}"),
        ExtractedContent::Component { name, .. } => name.clone(),
        _ => "Item".to_string(),
    }
}

fn render_message(
    msg: &Message,
    current_pid: Signal<String>,
    reply_count: usize,
    thread_extractions: &[Extraction],
    show_reply_button: bool,
    mut on_reply: impl FnMut(String) + 'static,
) -> Element {
    let is_own = *current_pid.read() == msg.author.id;
    let time_str = msg.timestamp.format("%H:%M").to_string();
    let initials = avatar_initials(&msg.author.name);
    let is_agent = matches!(msg.author.kind, sruja_chat::ParticipantKind::Agent(_));
    let msg_id = msg.id.clone();

    rsx! {
        div {
            class: if is_own { "message message-own" } else { "message" },

            div {
                class: "message-avatar",
                title: "{msg.author.name}",
                span { class: "message-avatar-initials", "{initials}" }
            }

            div {
                class: "message-body",
                div {
                    class: "message-meta",
                    span {
                        class: if is_agent { "message-author message-author-agent" } else { "message-author" },
                        "{msg.author.name}"
                    }
                    span { class: "message-time", "{time_str}" }
                }
                div {
                    class: "message-content",
                    "{msg.content}"
                }
                if reply_count > 0 || !thread_extractions.is_empty() {
                    div {
                        class: "message-thread-summary",
                        if reply_count > 0 {
                            if show_reply_button {
                                button {
                                    class: "thread-reply-btn",
                                    onclick: move |_| on_reply(msg_id.clone()),
                                    "↳ {reply_count} replies"
                                }
                            } else {
                                span { class: "thread-reply-count", "↳ {reply_count} replies" }
                            }
                        }
                        if !thread_extractions.is_empty() {
                            div {
                                class: "thread-decisions",
                                span { class: "thread-decisions-label", "Decisions: " }
                                for ext in thread_extractions.iter().take(3) {
                                    span { class: "thread-decision-pill", "{extraction_title(ext)}" }
                                }
                                if thread_extractions.len() > 3 {
                                    span { class: "thread-decision-more", "+{thread_extractions.len() - 3} more" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ChatPanel(
    session_id: Signal<String>,
    participant_id: Signal<String>,
    extractions: Signal<Vec<Extraction>>,
) -> Element {
    let server = use_context::<ServerContext>();
    let server_clone: Arc<sruja_chat::ChatServer> = server.0.clone();
    let mut input = use_signal(String::new);
    let mut all_messages = use_signal(Vec::<Message>::new);
    let mut expanded_thread = use_signal(|| Option::<String>::None);

    let srv_for_effect = server_clone.clone();
    use_effect(move || {
        let sid = session_id.read().clone();
        if sid.is_empty() {
            all_messages.set(vec![]);
            return;
        }
        let srv = srv_for_effect.clone();
        let mut msgs = all_messages;
        spawn(async move {
            if let Ok(hist) = srv.get_history(&sid).await {
                msgs.set(hist);
            }
        });
    });

    let main_messages: Vec<Message> = all_messages
        .read()
        .iter()
        .filter(|m| m.parent_message_id.is_none())
        .cloned()
        .collect();

    let in_thread = expanded_thread.read().clone();
    let thread_replies: Vec<Message> = if let Some(ref parent_id) = in_thread {
        all_messages
            .read()
            .iter()
            .filter(|m| m.parent_message_id.as_deref() == Some(parent_id.as_str()))
            .cloned()
            .collect()
    } else {
        vec![]
    };

    let ex_list = extractions.read().clone();
    let num_replies = thread_replies.len();

    let main_with_meta: Vec<(Message, usize, Vec<Extraction>, String)> = main_messages
        .iter()
        .map(|m| {
            let mid = m.id.clone();
            let rc = all_messages
                .read()
                .iter()
                .filter(|x| x.parent_message_id.as_deref() == Some(mid.as_str()))
                .count();
            let te: Vec<Extraction> = ex_list
                .iter()
                .filter(|e| e.thread_root_message_id.as_deref() == Some(mid.as_str()))
                .cloned()
                .collect();
            (m.clone(), rc, te, mid)
        })
        .collect();

    rsx! {
        div {
            class: "chat-panel",

            if in_thread.is_some() {
                div {
                    class: "thread-header",
                    button {
                        class: "thread-back-btn",
                        onclick: move |_| expanded_thread.set(None),
                        "← Back to main"
                    }
                }
            }

            div {
                class: "message-list",
                if let Some(ref parent_id) = in_thread {
                    if let Some(parent) = all_messages.read().iter().find(|m| m.id == *parent_id) {
                        {render_message(
                            parent,
                            participant_id,
                            num_replies,
                            &[],
                            false,
                            |_| {},
                        )}
                        div { class: "thread-replies-header", "Replies" }
                        for reply in thread_replies.iter() {
                            {render_message(
                                reply,
                                participant_id,
                                0,
                                &[],
                                false,
                                |_| {},
                            )}
                        }
                    }
                } else {
                    for item in main_with_meta.iter() {
                        {{
                            let m = &item.0;
                            let rc = item.1;
                            let te = &item.2;
                            let pid = item.3.clone();
                            render_message(
                                m,
                                participant_id,
                                rc,
                                te,
                                true,
                                move |_| expanded_thread.set(Some(pid.clone())),
                            )
                        }}
                    }
                }
            }

            div {
                class: "message-input",

                input {
                    r#type: "text",
                    placeholder: if in_thread.is_some() {
                        "Reply in thread..."
                    } else {
                        "Discuss architecture decisions, requirements, constraints..."
                    },
                    value: "{input}",
                    oninput: move |e: Event<FormData>| *input.write() = e.value(),
                    onkeypress: {
                        let server = server_clone.clone();
                        move |e: Event<KeyboardData>| {
                            if e.key() == Key::Enter && !input.read().is_empty() {
                                let parent = expanded_thread.read().clone();
                                send_message(
                                    input,
                                    all_messages,
                                    extractions,
                                    session_id,
                                    participant_id,
                                    server.clone(),
                                    parent,
                                );
                            }
                        }
                    },
                }

                button {
                    onclick: {
                        let server = server_clone.clone();
                        move |_| {
                            let parent = expanded_thread.read().clone();
                            send_message(
                                input,
                                all_messages,
                                extractions,
                                session_id,
                                participant_id,
                                server.clone(),
                                parent,
                            );
                        }
                    },
                    "Send"
                }
            }
        }
    }
}

fn send_message(
    mut input: Signal<String>,
    messages: Signal<Vec<Message>>,
    extractions: Signal<Vec<Extraction>>,
    session_id: Signal<String>,
    participant_id: Signal<String>,
    server: Arc<sruja_chat::ChatServer>,
    parent_message_id: Option<String>,
) {
    if input.read().is_empty() {
        return;
    }

    let content = input.read().clone();
    let sid = session_id.read().clone();
    let pid = participant_id.read().clone();
    input.write().clear();

    if sid.is_empty() || pid.is_empty() {
        return;
    }

    let mut msgs = messages;
    let mut ex_signal = extractions;
    spawn(async move {
        let _ = server
            .send_message(
                &sid,
                NewMessage {
                    author_id: pid,
                    content,
                    parent_message_id,
                },
            )
            .await;
        if let Ok(hist) = server.get_history(&sid).await {
            msgs.set(hist);
        }
        if let Ok(ex) = server.get_extractions(&sid).await {
            ex_signal.set(ex);
        }
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        if let Ok(hist) = server.get_history(&sid).await {
            msgs.set(hist);
        }
        if let Ok(ex) = server.get_extractions(&sid).await {
            ex_signal.set(ex);
        }
    });
}
