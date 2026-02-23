//! Header component

use dioxus::prelude::*;

#[component]
pub fn Header(title: String, subtitle: String) -> Element {
    rsx! {
        header {
            class: "header",

            div {
                class: "header-brand",

                img {
                    class: "header-logo",
                    src: asset!("/assets/sruja-logo.png"),
                    alt: "Sruja Logo",
                }

                div {
                    class: "header-text",
                    span {
                        class: "header-title",
                        "{title}"
                    }
                    span {
                        class: "header-subtitle",
                        "{subtitle}"
                    }
                }
            }
        }
    }
}
