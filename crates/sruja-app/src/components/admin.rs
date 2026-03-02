//! Admin panel for organization-level agent management.
//!
//! Admins add agent definitions via this form. Regular developers only select
//! from existing agents in the toolbar.

use crate::app::ServerContext;
use dioxus::prelude::*;
use sruja_chat::{AgentDefinition, CreateAgentDefinition};

#[component]
pub fn AdminPanel(
    agent_definitions: Signal<Vec<AgentDefinition>>,
    on_close: EventHandler<()>,
) -> Element {
    let server = use_context::<ServerContext>();
    let mut name = use_signal(String::new);
    let mut role = use_signal(|| "Subsystem Expert".to_string());
    let mut system_prompt =
        use_signal(|| "You are an architecture reviewer. Respond concisely.".to_string());
    let mut knowledge_context = use_signal(String::new);
    let mut model = use_signal(|| "openai/gpt-4o-mini".to_string());
    let create_status = use_signal(|| Option::<String>::None);

    rsx! {
        div {
            class: "admin-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "admin-panel",
                onclick: move |e: Event<MouseData>| e.stop_propagation(),

                div {
                    class: "admin-header",
                    h3 { "Manage Agents" }
                    button {
                        class: "admin-close",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                p {
                    class: "admin-hint",
                    "Add agent definitions. Developers select from these when adding experts to a session."
                }

                div {
                    class: "admin-form",
                    label { "Name" }
                    input {
                        class: "admin-input",
                        placeholder: "e.g. Architecture Reviewer",
                        value: "{name.read()}",
                        oninput: move |e: Event<FormData>| *name.write() = e.value(),
                    }
                    label { "Role" }
                    input {
                        class: "admin-input",
                        placeholder: "e.g. Subsystem Expert",
                        value: "{role.read()}",
                        oninput: move |e: Event<FormData>| *role.write() = e.value(),
                    }
                    label { "System prompt" }
                    textarea {
                        class: "admin-textarea",
                        placeholder: "Core instructions and persona",
                        value: "{system_prompt.read()}",
                        oninput: move |e: Event<FormData>| *system_prompt.write() = e.value(),
                        rows: "3",
                    }
                    label { "Knowledge context (optional)" }
                    textarea {
                        class: "admin-textarea",
                        placeholder: "Domain docs, architecture context",
                        value: "{knowledge_context.read()}",
                        oninput: move |e: Event<FormData>| *knowledge_context.write() = e.value(),
                        rows: "2",
                    }
                    label { "Model" }
                    input {
                        class: "admin-input",
                        placeholder: "e.g. openai/gpt-4o-mini",
                        value: "{model.read()}",
                        oninput: move |e: Event<FormData>| *model.write() = e.value(),
                    }
                    button {
                        class: "admin-submit",
                        onclick: {
                            let server = server.0.clone();
                            let mut status = create_status;
                            let mut def_list = agent_definitions;
                            move |_| {
                                let n = name.read().clone();
                                let r = role.read().clone();
                                let p = system_prompt.read().clone();
                                let k = knowledge_context.read().clone();
                                let m = model.read().clone();
                                if n.is_empty() || r.is_empty() || p.is_empty() || m.is_empty() {
                                    status.set(Some("Name, role, prompt, and model are required.".to_string()));
                                    return;
                                }
                                let srv = server.clone();
                                let ctx = if k.is_empty() { None } else { Some(k) };
                                spawn(async move {
                                    let input = CreateAgentDefinition {
                                        name: n,
                                        role: r,
                                        system_prompt: p,
                                        knowledge_context: ctx,
                                        model: m,
                                        memory_limit_messages: None,
                                    };
                                    match srv.create_agent_definition(input).await {
                                        Ok(_) => {
                                            def_list.set(srv.list_agent_definitions().await);
                                            status.set(Some("Agent created.".to_string()));
                                        }
                                        Err(e) => status.set(Some(format!("Error: {}", e))),
                                    }
                                });
                            }
                        },
                        "Create Agent"
                    }
                }

                if let Some(ref msg) = *create_status.read() {
                    div { class: "admin-status", "{msg}" }
                }

                div {
                    class: "admin-list",
                    h4 {
                        { format!("Existing agents ({})", agent_definitions.read().len()) }
                    }
                    ul {
                        for d in agent_definitions.read().iter() {
                            li { "{d.name} — {d.role} ({d.model})" }
                        }
                    }
                }
            }
        }
    }
}
