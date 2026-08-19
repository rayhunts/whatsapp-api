pub mod api;
pub mod components;
pub mod state;

use dioxus::prelude::*;

use crate::components::{
    automations::AutomationsView, broadcast::BroadcastView, chat_window::ChatWindow,
    contacts::ContactsView, groups::GroupsView, login::Login, navigation::NavigationRail,
    settings::SettingsPanel, sidebar::Sidebar,
};
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
                    NavigationRail {}
                    Sidebar {}
                    MainView {}
                    SettingsPanel {}
                },
            }
        }
    }
}

#[component]
fn MainView() -> Element {
    match crate::state::app_state::CURRENT_VIEW() {
        crate::state::app_state::CurrentView::Chats => rsx! { ChatWindow {} },
        crate::state::app_state::CurrentView::Contacts => rsx! { ContactsView {} },
        crate::state::app_state::CurrentView::Groups => rsx! { GroupsView {} },
        crate::state::app_state::CurrentView::Broadcast => rsx! { BroadcastView {} },
        crate::state::app_state::CurrentView::Automations => rsx! { AutomationsView {} },
    }
}
