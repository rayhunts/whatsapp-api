use dioxus::prelude::*;
use whatsapp_api_types::domain::message::{Message, MessageDirection, MessageStatus};

#[component]
pub fn MessageBubble(msg: Message) -> Element {
    let time = js_sys::Date::new(&js_sys::Number::from(msg.timestamp_ms as f64))
        .to_locale_time_string("en-GB");
    let sender_name = msg.sender_name.clone();
    let text = msg.text.clone();
    let status_label = if msg.direction == MessageDirection::Outgoing {
        Some(status_text(msg.status))
    } else {
        None
    };
    rsx! {
        div { class: if msg.from_me { "bubble out" } else { "bubble in" },
            if !msg.from_me && !sender_name.is_empty() {
                div { class: "bubble-author", "{sender_name}" }
            }
            div { class: "bubble-text", "{text}" }
            div { class: "bubble-meta",
                span { class: "bubble-time", "{time}" }
                if let Some(label) = status_label {
                    span { class: "bubble-status", "{label}" }
                }
            }
            if let Some(err) = msg.error {
                div { class: "bubble-error", "{err}" }
            }
        }
    }
}

fn status_text(status: MessageStatus) -> &'static str {
    match status {
        MessageStatus::Pending => "pending",
        MessageStatus::Sent => "sent",
        MessageStatus::Delivered => "delivered",
        MessageStatus::Read => "read",
        MessageStatus::Failed => "failed",
    }
}
