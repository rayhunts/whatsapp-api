pub mod api;
pub mod components;
pub mod state;

use dioxus::prelude::*;

use crate::components::{chat_window::ChatWindow, login::Login, settings::SettingsPanel, sidebar::Sidebar};
use crate::state::app_state::{CONNECTION, UiConnection};
use crate::state::ws::spawn_ws;

#[component]
pub fn App() -> Element {
    let _ = use_effect(spawn_ws);

    rsx! {
        style { {include_str!("./style.css")} }
        div { class: "app",
            match CONNECTION() {
                UiConnection::WaitingForQr | UiConnection::Connecting => rsx! { Login {} },
                _ => rsx! {
                    Sidebar {}
                    ChatWindow {}
                    SettingsPanel {}
                },
            }
        }
    }
}