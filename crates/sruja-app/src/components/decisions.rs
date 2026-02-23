//! Decision panel component

use crate::app::ServerContext;
use dioxus::prelude::*;
use sruja_extract::{Extraction, ExtractionStatus, Intent};
use std::sync::Arc;

#[component]
pub fn DecisionPanel(session_id: Signal<String>, extractions: Signal<Vec<Extraction>>) -> Element {
    let server = use_context::<ServerContext>();
    let items = extractions.read().clone();

    rsx! {
        div {
            class: "decision-panel",

            h2 {
                class: "decision-panel-title",
                "Extracted Items"
                if !items.is_empty() {
                    span {
                        class: "extraction-count",
                        " ({items.len()})"
                    }
                }
            }

            if items.is_empty() {
                div {
                    class: "empty-state",
                    "Start discussing architecture"
                    br {}
                    "to extract decisions!"
                    br {}
                    br {}
                    span {
                        style: "font-size: 0.75rem; color: var(--text-muted);",
                        "Try: \"We should use Kafka for events\""
                    }
                }
            }

            div {
                class: "extractions-list",
                for ext in items.iter() {
                    ExtractionCard {
                        extraction: ext.clone(),
                        on_confirm: {
                            let extractions = extractions.clone();
                            let sid = session_id.read().clone();
                            let id = ext.id.clone();
                            let srv = Arc::clone(&server.0);
                            move |_| {
                                confirm_extraction(extractions.clone(), &id);
                                if !sid.is_empty() {
                                    let srv2 = Arc::clone(&srv);
                                    let sid2 = sid.clone();
                                    let id2 = id.clone();
                                    spawn(async move {
                                        let _ = srv2.confirm_extraction(&sid2, &id2).await;
                                    });
                                }
                            }
                        },
                        on_reject: {
                            let extractions = extractions.clone();
                            let sid = session_id.read().clone();
                            let id = ext.id.clone();
                            let srv = Arc::clone(&server.0);
                            move |_| {
                                reject_extraction(extractions.clone(), &id);
                                if !sid.is_empty() {
                                    let srv2 = Arc::clone(&srv);
                                    let sid2 = sid.clone();
                                    let id2 = id.clone();
                                    spawn(async move {
                                        let _ = srv2.reject_extraction(&sid2, &id2).await;
                                    });
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

fn confirm_extraction(mut extractions: Signal<Vec<Extraction>>, id: &str) {
    if let Some(ext) = extractions.write().iter_mut().find(|e| e.id == id) {
        ext.status = ExtractionStatus::Confirmed;
    }
}

fn reject_extraction(mut extractions: Signal<Vec<Extraction>>, id: &str) {
    extractions.write().retain(|e| e.id != id);
}

#[component]
fn ExportAdrButton(extraction: Extraction, title: String) -> Element {
    let Some(markdown) = extraction.to_adr_markdown(None) else {
        return rsx! {};
    };
    rsx! {
        button {
            class: "btn-export-adr",
            onclick: move |_| {
                export_adr_to_file(&markdown, &title);
            },
            "Export ADR"
        }
    }
}

fn export_adr_to_file(md: &str, title: &str) {
    let sanitized: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let default_name = format!("ADR-{}.md", sanitized);
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Markdown", &["md"])
        .set_file_name(&default_name)
        .save_file()
    {
        if let Err(e) = std::fs::write(&path, md) {
            tracing::error!("Failed to write ADR file: {}", e);
        }
    }
}

#[component]
fn ExtractionCard(
    extraction: Extraction,
    on_confirm: EventHandler<()>,
    on_reject: EventHandler<()>,
) -> Element {
    let status_class = match extraction.status {
        ExtractionStatus::Confirmed => "confirmed",
        ExtractionStatus::Rejected => "rejected",
        ExtractionStatus::Draft => "draft",
    };

    let (icon, label) = match extraction.intent {
        Intent::Decision => ("📋", "Decision"),
        Intent::Requirement => ("📌", "Requirement"),
        Intent::Constraint => ("🚫", "Constraint"),
        Intent::Policy => ("📜", "Policy"),
        Intent::Risk => ("⚠️", "Risk"),
        Intent::ComponentMention => ("🔧", "Component"),
        _ => ("📝", "Note"),
    };

    let title = match &extraction.content {
        sruja_extract::ExtractedContent::Decision { title, .. } => title.clone(),
        sruja_extract::ExtractedContent::Requirement { title, .. } => title.clone(),
        sruja_extract::ExtractedContent::Constraint { source, target, .. } => {
            format!("{} → {}", source, target)
        }
        sruja_extract::ExtractedContent::Component { name, .. } => name.clone(),
        _ => "Extraction".to_string(),
    };

    let confidence = (extraction.confidence * 100.0) as i32;

    rsx! {
        div {
            class: "extraction-card {status_class}",

            div {
                class: "extraction-header",
                span {
                    class: "extraction-type",
                    "{icon} {label}"
                }
                span {
                    class: "extraction-confidence",
                    "{confidence}%"
                }
            }

            div {
                class: "extraction-title",
                "{title}"
            }

            if extraction.status == ExtractionStatus::Draft {
                div {
                    class: "extraction-actions",
                    button {
                        class: "btn-confirm",
                        onclick: move |_| on_confirm.call(()),
                        "✓ Confirm"
                    }
                    button {
                        class: "btn-reject",
                        onclick: move |_| on_reject.call(()),
                        "✗ Reject"
                    }
                }
            }

            if matches!(extraction.intent, Intent::Decision) {
                ExportAdrButton {
                    extraction: extraction.clone(),
                    title: title.clone(),
                }
            }

            if extraction.status == ExtractionStatus::Confirmed {
                div {
                    class: "extraction-confirmed",
                    "✓ Confirmed"
                }
            }
        }
    }
}
