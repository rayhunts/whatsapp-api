use dioxus::prelude::*;

use crate::state::app_state::{CONNECTION, LAST_ERROR, QR_IMAGE, UiConnection};

#[component]
pub fn Login() -> Element {
    let qr = QR_IMAGE();
    let error = LAST_ERROR();
    rsx! {
        div { class: "login",
            div { class: "login-card",
                h1 { "WhatsApp API" }
                p { class: "subtitle", "Link a device to start messaging" }
                match CONNECTION() {
                    UiConnection::Connecting => rsx! { p { class: "hint", "Connecting to WhatsApp…" } },
                    _ => rsx! { p { class: "hint", "Waiting for a QR code…" } },
                }
                if let Some(image) = qr {
                    img { class: "qr", src: "{image}", alt: "WhatsApp pairing QR code" }
                    p { class: "hint", "Open WhatsApp on your phone → Settings → Linked devices → Link a device, then scan this QR code." }
                }
                if let Some(err) = error {
                    p { class: "error", "{err}" }
                }
            }
        }
    }
}
