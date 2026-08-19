use dioxus::prelude::*;

use crate::state::app_state::{
    API_BASE_URL, CONNECTION, SHOW_SETTINGS, UiConnection, save_api_base_url,
};

const BACKEND_DOCS_URL: &str = "http://localhost:8081/docs#description/introduction";

#[component]
pub fn SettingsPanel() -> Element {
    let show = SHOW_SETTINGS();
    let mut url_input = use_signal(|| API_BASE_URL.read().clone());

    if !show {
        return rsx! {};
    }

    rsx! {
        div { class: "settings-overlay",
            div { class: "settings-backdrop",
                onclick: move |_| SHOW_SETTINGS.with_mut(|s| *s = false),
            }
            div { class: "settings-panel",
                div { class: "settings-header",
                    h3 { "Settings" }
                    button {
                        class: "icon-button",
                        onclick: move |_| SHOW_SETTINGS.with_mut(|s| *s = false),
                        "✕"
                    }
                }
                div { class: "settings-body",
                    section { class: "settings-section",
                        h4 { "Connection" }
                        div { class: "settings-row",
                            span { "Status" }
                            ConnectionBadge {}
                        }
                    }
                    section { class: "settings-section",
                        h4 { "API Configuration" }
                        p { class: "settings-hint",
                            "Leave empty to use the same host that serves this dashboard. Set a custom URL when running the API on a different origin."
                        }
                        label { class: "settings-label", "API base URL" }
                        input {
                            class: "settings-input",
                            value: url_input(),
                            placeholder: "http://localhost:8081",
                            oninput: move |ev| url_input.set(ev.value()),
                        }
                        div { class: "settings-actions",
                            button {
                                class: "settings-save",
                                onclick: move |_| save_api_base_url(&url_input()),
                                "Save API URL"
                            }
                        }
                    }
                    section { class: "settings-section",
                        h4 { "Documentation" }
                        p { class: "settings-hint",
                            "A WhatsApp Web API server and GUI built in Rust."
                        }
                        a {
                            class: "settings-link",
                            href: BACKEND_DOCS_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "Open backend docs ↗"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ConnectionBadge() -> Element {
    let (label, class) = match CONNECTION() {
        UiConnection::Connected => ("Connected", "badge badge-success"),
        UiConnection::Connecting => ("Connecting", "badge badge-pending"),
        UiConnection::WaitingForQr => ("Waiting for QR", "badge badge-pending"),
        UiConnection::Disconnected => ("Disconnected", "badge badge-error"),
        UiConnection::Reconnecting => ("Reconnecting", "badge badge-pending"),
    };
    rsx! {
        span { class: "{class}", "{label}" }
    }
}
