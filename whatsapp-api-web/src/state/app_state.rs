use std::collections::HashMap;

use dioxus::prelude::*;
use whatsapp_api_types::domain::automation::Automation;
use whatsapp_api_types::domain::chat::{Chat, ChatKind};
use whatsapp_api_types::domain::contact::{Contact, ContactGroup};
use whatsapp_api_types::domain::message::Message;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiConnection {
    Connecting,
    WaitingForQr,
    Connected,
    Disconnected,
    Reconnecting,
}

/// Sidebar filter for chat categories.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChatFilter {
    All,
    Private,
    Groups,
    StatusBroadcast,
}

impl ChatFilter {
    pub const ALL: [Self; 4] = [Self::All, Self::Private, Self::Groups, Self::StatusBroadcast];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Private => "Private",
            Self::Groups => "Groups",
            Self::StatusBroadcast => "Status & Broadcast",
        }
    }

    pub fn matches(self, kind: ChatKind) -> bool {
        match self {
            Self::All => true,
            Self::Private => kind == ChatKind::Private,
            Self::Groups => kind == ChatKind::Group,
            Self::StatusBroadcast => {
                kind == ChatKind::Status || kind == ChatKind::Broadcast || kind == ChatKind::Newsletter
            }
        }
    }
}

/// Primary views accessible from the persistent rail navigation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CurrentView {
    Chats,
    Contacts,
    Groups,
    Broadcast,
    Automations,
}

impl CurrentView {
    pub const ALL: [Self; 5] = [Self::Chats, Self::Contacts, Self::Groups, Self::Broadcast, Self::Automations];

    pub fn label(self) -> &'static str {
        match self {
            Self::Chats => "Chats",
            Self::Contacts => "Contacts",
            Self::Groups => "Groups",
            Self::Broadcast => "Broadcast",
            Self::Automations => "Automations",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Chats => "💬",
            Self::Contacts => "👤",
            Self::Groups => "👥",
            Self::Broadcast => "📢",
            Self::Automations => "⚙",
        }
    }
}

pub static CONNECTION: GlobalSignal<UiConnection> = Signal::global(|| UiConnection::Connecting);
pub static QR_IMAGE: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static CHATS: GlobalSignal<Vec<Chat>> = Signal::global(Vec::new);
pub static MESSAGES: GlobalSignal<HashMap<String, Vec<Message>>> = Signal::global(HashMap::new);
pub static ACTIVE_CHAT: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static LAST_ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static API_BASE_URL: GlobalSignal<String> = Signal::global(load_api_base_url);
pub static SHOW_SETTINGS: GlobalSignal<bool> = Signal::global(|| false);
pub static SHOW_NEW_CHAT: GlobalSignal<bool> = Signal::global(|| false);
pub static CHAT_FILTER: GlobalSignal<ChatFilter> = Signal::global(|| ChatFilter::All);
pub static CURRENT_VIEW: GlobalSignal<CurrentView> = Signal::global(|| CurrentView::Chats);
pub static AUTOMATIONS: GlobalSignal<Vec<Automation>> = Signal::global(Vec::new);
pub static CONTACTS: GlobalSignal<Vec<Contact>> = Signal::global(Vec::new);
pub static CONTACT_GROUPS: GlobalSignal<Vec<ContactGroup>> = Signal::global(Vec::new);

const API_BASE_URL_KEY: &str = "whatsapp_api_base_url";

fn load_api_base_url() -> String {
    local_storage_get(API_BASE_URL_KEY).unwrap_or_default()
}

pub fn save_api_base_url(url: &str) {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        let _ = local_storage_remove(API_BASE_URL_KEY);
    } else {
        let _ = local_storage_set(API_BASE_URL_KEY, trimmed);
    }
    API_BASE_URL.with_mut(|u| *u = trimmed.to_string());
}

fn local_storage_get(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok().flatten()
}

fn local_storage_set(key: &str, value: &str) -> Result<(), wasm_bindgen::JsValue> {
    let window = web_sys::window().ok_or(wasm_bindgen::JsValue::UNDEFINED)?;
    let storage = window.local_storage()?.ok_or(wasm_bindgen::JsValue::UNDEFINED)?;
    storage.set_item(key, value)
}

fn local_storage_remove(key: &str) -> Result<(), wasm_bindgen::JsValue> {
    let window = web_sys::window().ok_or(wasm_bindgen::JsValue::UNDEFINED)?;
    let storage = window.local_storage()?.ok_or(wasm_bindgen::JsValue::UNDEFINED)?;
    storage.remove_item(key)
}
