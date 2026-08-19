use dioxus::prelude::*;
use whatsapp_api_types::domain::chat::ChatKind;

use crate::api;
use crate::components::message_bubble::MessageBubble;
use crate::state::app_state::{ACTIVE_CHAT, CHATS, LAST_ERROR, MESSAGES, SHOW_SETTINGS};

#[component]
pub fn ChatWindow() -> Element {
    let active = ACTIVE_CHAT();
    let messages = MESSAGES();
    let chats = CHATS();
    let mut show_info = use_signal(|| false);

    let _ = use_effect(move || {
        let active = ACTIVE_CHAT();
        spawn(async move {
            if let Some(jid) = active
                && let Ok(msgs) = api::client::fetch_messages(&jid).await {
                    MESSAGES.with_mut(|m| {
                        m.insert(jid, msgs);
                    });
                }
        });
    });

    let Some(jid) = active else {
        return rsx! {
            main { class: "chat-window placeholder",
                p { "Select a chat to start messaging" }
            }
        };
    };

    let chat = chats.iter().find(|c| c.jid == jid).cloned();
    let name = chat.as_ref().and_then(|c| {
        if c.name.is_empty() {
            None
        } else {
            Some(c.name.clone())
        }
    }).unwrap_or_else(|| jid.clone());
    let kind = chat.as_ref().map(|c| c.kind);
    let chat_messages = messages.get(&jid).cloned().unwrap_or_default();
    let error = LAST_ERROR();
    let subtitle = kind.map(chat_kind_subtitle).unwrap_or_default();

    rsx! {
        main { class: "chat-window",
            header { class: "chat-header",
                div { class: "chat-header-avatar",
                    "{kind.map(chat_kind_icon).unwrap_or(\"💬\")}"
                }
                div { class: "chat-header-info",
                    h3 { "{name}" }
                    span { class: "chat-header-subtitle", "{subtitle}" }
                }
                div { class: "chat-header-actions",
                    button {
                        class: "icon-button info-button",
                        title: "Chat info",
                        onclick: move |_| show_info.with_mut(|s| *s = !*s),
                        "ℹ"
                    }
                    button {
                        class: "icon-button automation-button",
                        title: "Automations",
                        onclick: move |_| SHOW_SETTINGS.with_mut(|s| *s = true),
                        "⚙"
                    }
                }
            }
            if show_info() {
                ChatInfoPanel { jid: jid.clone(), name: name.clone(), kind }
            }
            div { class: "messages",
                for m in chat_messages {
                    MessageBubble { msg: m }
                }
            }
            if let Some(err) = error {
                div { class: "banner", "{err}" }
            }
            Composer { jid }
        }
    }
}

#[component]
fn ChatInfoPanel(jid: String, name: String, kind: Option<ChatKind>) -> Element {
    let kind_label = kind.map(chat_kind_subtitle).unwrap_or("Unknown");
    rsx! {
        div { class: "chat-info-panel",
            h4 { "Chat info" }
            div { class: "chat-info-row",
                span { class: "chat-info-label", "Name" }
                span { class: "chat-info-value", "{name}" }
            }
            div { class: "chat-info-row",
                span { class: "chat-info-label", "JID" }
                code { class: "chat-info-value jid-code", "{jid}" }
            }
            div { class: "chat-info-row",
                span { class: "chat-info-label", "Type" }
                span { class: "chat-info-value", "{kind_label}" }
            }
            p { class: "chat-info-hint",
                "Use this JID in automations or API calls. For private contacts it is just the phone number with country code followed by @s.whatsapp.net."
            }
        }
    }
}

#[component]
fn Composer(jid: String) -> Element {
    let mut input = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let chat_jid = jid.clone();

    rsx! {
        form {
            class: "composer",
            onsubmit: move |ev| {
                ev.prevent_default();
                let text = input.read().trim().to_string();
                if text.is_empty() {
                    return;
                }
                input.set(String::new());
                error.set(None);
                let chat_jid = chat_jid.clone();
                spawn(async move {
                    sending.with_mut(|s| *s = true);
                    if let Err(err) = api::client::send_text(&chat_jid, &text).await {
                        error.set(Some(format!("failed to send: {err}")));
                    }
                    sending.with_mut(|s| *s = false);
                });
            },
            input {
                class: "composer-input",
                value: input(),
                oninput: move |ev| {
                    input.set(ev.value());
                    error.set(None);
                },
                placeholder: "Type a message…",
            }
            button {
                class: "composer-send",
                r#type: "submit",
                disabled: sending(),
                "Send"
            }
            if let Some(err) = error() {
                span { class: "composer-error", "{err}" }
            }
        }
    }
}

fn chat_kind_icon(kind: ChatKind) -> &'static str {
    match kind {
        ChatKind::Group => "👥",
        ChatKind::Private => "👤",
        ChatKind::Broadcast => "📢",
        ChatKind::Status => "🟢",
        ChatKind::Newsletter => "📰",
        ChatKind::Unknown => "💬",
    }
}

fn chat_kind_subtitle(kind: ChatKind) -> &'static str {
    match kind {
        ChatKind::Group => "WhatsApp Group",
        ChatKind::Private => "Private chat",
        ChatKind::Broadcast => "Broadcast list",
        ChatKind::Status => "Status updates",
        ChatKind::Newsletter => "Newsletter",
        ChatKind::Unknown => "",
    }
}
