use dioxus::prelude::*;
use whatsapp_api_types::domain::automation::{
    phones_to_jids, Automation, AutomationConfig, AutomationKind, CreateAutomationRequest, MatchType,
    UpdateAutomationRequest,
};

use crate::api;
use crate::state::app_state::{
    API_BASE_URL, AUTOMATIONS, CONNECTION, SHOW_SETTINGS, UiConnection, save_api_base_url,
};

const BACKEND_DOCS_URL: &str = "http://localhost:8081/docs#description/introduction";

#[component]
pub fn SettingsPanel() -> Element {
    let show = SHOW_SETTINGS();
    let mut url_input = use_signal(|| API_BASE_URL.read().clone());

    let _ = use_effect(move || {
        if SHOW_SETTINGS() {
            spawn(async move {
                if let Ok(list) = api::client::list_automations().await {
                    AUTOMATIONS.with_mut(|a| *a = list);
                }
            });
        }
    });

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
                    h3 { "Settings & API" }
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
                        h4 { "Automations" }
                        p { class: "settings-hint",
                            "Create scheduled messages, auto-replies, forwarders and webhooks. Schedulers run every minute."
                        }
                        AutomationList {}
                        AutomationForm {}
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

#[component]
fn AutomationList() -> Element {
    let automations = AUTOMATIONS();

    rsx! {
        if automations.is_empty() {
            p { class: "settings-hint", "No automations yet." }
        } else {
            ul { class: "automation-list",
                for a in automations {
                    AutomationItem { automation: a.clone() }
                }
            }
        }
    }
}

#[component]
fn AutomationItem(automation: Automation) -> Element {
    let id = automation.id.clone();
    let id2 = automation.id.clone();
    let id3 = automation.id.clone();
    let enabled = automation.enabled;

    let toggle = move |_| {
        let id_api = id.clone();
        let id_update = id.clone();
        spawn(async move {
            let req = UpdateAutomationRequest {
                name: None,
                enabled: Some(!enabled),
                config: None,
            };
            if let Ok(updated) = api::client::update_automation(&id_api, &req).await {
                AUTOMATIONS.with_mut(|list| {
                    if let Some(item) = list.iter_mut().find(|a| a.id == id_update) {
                        *item = updated;
                    }
                });
            }
        });
    };

    let trigger = move |_| {
        let id_api = id2.clone();
        spawn(async move {
            let _ = api::client::trigger_automation(&id_api).await;
        });
    };

    let delete = move |_| {
        let id_api = id3.clone();
        let id_retain = id3.clone();
        spawn(async move {
            if api::client::delete_automation(&id_api).await.is_ok() {
                AUTOMATIONS.with_mut(|list| list.retain(|a| a.id != id_retain));
            }
        });
    };

    rsx! {
        li { class: "automation-item",
            div { class: "automation-main",
                span { class: "automation-name", "{automation.name}" }
                span { class: "automation-kind", "{kind_label(&automation.config)}" }
                span { class: if automation.enabled { "badge badge-success" } else { "badge badge-error" },
                    if automation.enabled { "Enabled" } else { "Disabled" }
                }
            }
            div { class: "automation-actions",
                button {
                    class: "automation-action",
                    onclick: toggle,
                    if automation.enabled { "Disable" } else { "Enable" }
                }
                if matches!(automation.config, AutomationConfig::ScheduledMessage { .. }) {
                    button {
                        class: "automation-action",
                        onclick: trigger,
                        "Run now"
                    }
                }
                button {
                    class: "automation-action danger",
                    onclick: delete,
                    "Delete"
                }
            }
        }
    }
}

#[component]
fn AutomationForm() -> Element {
    let mut kind = use_signal(|| AutomationKind::ScheduledMessage);
    let mut name = use_signal(String::new);
    let mut phones = use_signal(String::new);
    let mut target_jid = use_signal(String::new);
    let mut source_jid = use_signal(String::new);
    let mut text = use_signal(String::new);
    let mut schedule = use_signal(String::new);
    let mut pattern = use_signal(String::new);
    let mut response = use_signal(String::new);
    let mut match_type = use_signal(|| MatchType::Contains);
    let mut cooldown = use_signal(|| 60u64);
    let mut webhook_url = use_signal(String::new);
    let mut on_message = use_signal(|| true);
    let mut on_sent = use_signal(|| true);
    let mut on_connected = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let submit = move |ev: dioxus::events::FormEvent| {
        ev.prevent_default();
        error.set(None);
        let current_kind = *kind.read();
        let cfg_result: Result<AutomationConfig, String> = match current_kind {
            AutomationKind::ScheduledMessage => {
                let jids = phones_to_jids(
                    &phones.read().lines().map(|l| l.to_string()).collect::<Vec<_>>(),
                );
                if jids.is_empty() {
                    Err("Enter at least one valid phone number.".into())
                } else if schedule.read().trim().is_empty() {
                    Err("Schedule is required.".into())
                } else {
                    Ok(AutomationConfig::ScheduledMessage {
                        to: jids,
                        text: text.read().trim().to_string(),
                        schedule: schedule.read().trim().to_string(),
                    })
                }
            }
            AutomationKind::AutoReply => {
                let chat_filter = {
                    let t = target_jid.read().trim().to_string();
                    if t.is_empty() { None } else { Some(t) }
                };
                Ok(AutomationConfig::AutoReply {
                    chat_filter,
                    match_type: *match_type.read(),
                    pattern: pattern.read().trim().to_string(),
                    response: response.read().trim().to_string(),
                    cooldown_secs: *cooldown.read(),
                })
            }
            AutomationKind::Forwarder => {
                let from = source_jid.read().trim().to_string();
                let to = target_jid.read().trim().to_string();
                if from.is_empty() || to.is_empty() {
                    Err("Source and target JIDs are required.".into())
                } else {
                    Ok(AutomationConfig::Forwarder {
                        from,
                        to,
                        filter: None,
                    })
                }
            }
            AutomationKind::Webhook => {
                let url = webhook_url.read().trim().to_string();
                if url.is_empty() {
                    Err("Webhook URL is required.".into())
                } else {
                    Ok(AutomationConfig::Webhook {
                        url,
                        on_message: on_message(),
                        on_sent: on_sent(),
                        on_connected: on_connected(),
                    })
                }
            }
        };
        let cfg = match cfg_result {
            Ok(c) => c,
            Err(e) => {
                error.set(Some(e));
                return;
            }
        };
        let req = CreateAutomationRequest {
            name: name.read().trim().to_string(),
            enabled: true,
            config: cfg,
        };
        spawn(async move {
            match api::client::create_automation(&req).await {
                Ok(created) => {
                    AUTOMATIONS.with_mut(|list| list.push(created));
                    name.set(String::new());
                    phones.set(String::new());
                    target_jid.set(String::new());
                    source_jid.set(String::new());
                    text.set(String::new());
                    schedule.set(String::new());
                    pattern.set(String::new());
                    response.set(String::new());
                    webhook_url.set(String::new());
                }
                Err(e) => error.set(Some(e.to_string())),
            }
        });
    };

    rsx! {
        form { class: "automation-form", onsubmit: submit,
            div { class: "form-row",
                label { "Name" }
                input {
                    class: "settings-input",
                    value: name(),
                    placeholder: "Friendly name",
                    oninput: move |ev| name.set(ev.value()),
                }
            }
            div { class: "form-row",
                label { "Type" }
                select {
                    class: "settings-input",
                    onchange: move |ev| {
                        kind.set(match ev.value().as_str() {
                            "auto_reply" => AutomationKind::AutoReply,
                            "forwarder" => AutomationKind::Forwarder,
                            "webhook" => AutomationKind::Webhook,
                            _ => AutomationKind::ScheduledMessage,
                        });
                    },
                    option { value: "scheduled_message", selected: matches!(kind(), AutomationKind::ScheduledMessage), "Scheduled message" }
                    option { value: "auto_reply", selected: matches!(kind(), AutomationKind::AutoReply), "Auto-reply" }
                    option { value: "forwarder", selected: matches!(kind(), AutomationKind::Forwarder), "Forwarder" }
                    option { value: "webhook", selected: matches!(kind(), AutomationKind::Webhook), "Webhook" }
                }
            }
            match kind() {
                AutomationKind::ScheduledMessage => rsx! {
                    div { class: "form-row",
                        label { "Phone numbers (one per line)" }
                        textarea {
                            class: "settings-input",
                            value: phones(),
                            placeholder: "+62 812-3456-7890\n+1 415-555-2671",
                            oninput: move |ev| phones.set(ev.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "Message" }
                        input {
                            class: "settings-input",
                            value: text(),
                            placeholder: "Hello from automation",
                            oninput: move |ev| text.set(ev.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "Schedule (cron or ISO-8601)" }
                        input {
                            class: "settings-input",
                            value: schedule(),
                            placeholder: "0 8 * * *",
                            oninput: move |ev| schedule.set(ev.value()),
                        }
                    }
                },
                AutomationKind::AutoReply => rsx! {
                    div { class: "form-row",
                        label { "Chat JID filter (optional)" }
                        input {
                            class: "settings-input",
                            value: target_jid(),
                            placeholder: "Leave empty to match all chats",
                            oninput: move |ev| target_jid.set(ev.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "Match type" }
                        select {
                            class: "settings-input",
                            onchange: move |ev| {
                                match_type.set(match ev.value().as_str() {
                                    "exact" => MatchType::Exact,
                                    "regex" => MatchType::Regex,
                                    _ => MatchType::Contains,
                                });
                            },
                            option { value: "contains", selected: matches!(match_type(), MatchType::Contains), "Contains" }
                            option { value: "exact", selected: matches!(match_type(), MatchType::Exact), "Exact" }
                            option { value: "regex", selected: matches!(match_type(), MatchType::Regex), "Regex" }
                        }
                    }
                    div { class: "form-row",
                        label { "Pattern" }
                        input {
                            class: "settings-input",
                            value: pattern(),
                            placeholder: "price",
                            oninput: move |ev| pattern.set(ev.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "Response" }
                        input {
                            class: "settings-input",
                            value: response(),
                            placeholder: "Our price list is available at ...",
                            oninput: move |ev| response.set(ev.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "Cooldown (seconds)" }
                        input {
                            class: "settings-input",
                            r#type: "number",
                            value: cooldown().to_string(),
                            oninput: move |ev| {
                                if let Ok(v) = ev.value().parse() {
                                    cooldown.set(v);
                                }
                            },
                        }
                    }
                },
                AutomationKind::Forwarder => rsx! {
                    div { class: "form-row",
                        label { "From JID" }
                        input {
                            class: "settings-input",
                            value: source_jid(),
                            placeholder: "source-group@g.us",
                            oninput: move |ev| source_jid.set(ev.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "To JID" }
                        input {
                            class: "settings-input",
                            value: target_jid(),
                            placeholder: "target-group@g.us",
                            oninput: move |ev| target_jid.set(ev.value()),
                        }
                    }
                },
                AutomationKind::Webhook => rsx! {
                    div { class: "form-row",
                        label { "URL" }
                        input {
                            class: "settings-input",
                            value: webhook_url(),
                            placeholder: "https://example.com/webhook",
                            oninput: move |ev| webhook_url.set(ev.value()),
                        }
                    }
                    div { class: "form-row check-row",
                        input {
                            r#type: "checkbox",
                            checked: on_message(),
                            onchange: move |ev| on_message.set(ev.checked()),
                        }
                        label { "On incoming message" }
                    }
                    div { class: "form-row check-row",
                        input {
                            r#type: "checkbox",
                            checked: on_sent(),
                            onchange: move |ev| on_sent.set(ev.checked()),
                        }
                        label { "On sent message" }
                    }
                    div { class: "form-row check-row",
                        input {
                            r#type: "checkbox",
                            checked: on_connected(),
                            onchange: move |ev| on_connected.set(ev.checked()),
                        }
                        label { "On connected" }
                    }
                },
            }
            if let Some(err) = error() {
                div { class: "modal-error", "{err}" }
            }
            div { class: "form-actions",
                button { class: "modal-primary", r#type: "submit", "Create automation" }
            }
        }
    }
}

fn kind_label(config: &AutomationConfig) -> &'static str {
    match config {
        AutomationConfig::ScheduledMessage { .. } => "Scheduled message",
        AutomationConfig::AutoReply { .. } => "Auto-reply",
        AutomationConfig::Forwarder { .. } => "Forwarder",
        AutomationConfig::Webhook { .. } => "Webhook",
    }
}
