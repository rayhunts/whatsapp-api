use dioxus::prelude::*;
use whatsapp_api_types::domain::chat::{Chat, ChatKind};
use whatsapp_api_types::domain::contact::ContactGroup;

use crate::state::app_state::{
    ACTIVE_CHAT, CHAT_FILTER, CHATS, CONTACT_GROUPS, CONTACTS, CURRENT_VIEW, ChatFilter, CurrentView,
    SHOW_NEW_CHAT,
};

#[component]
pub fn Sidebar() -> Element {
    let view = CURRENT_VIEW();

    rsx! {
        aside { class: "sidebar",
            match view {
                CurrentView::Chats => rsx! { ChatSidebar {} },
                CurrentView::Contacts => rsx! { ContactSidebar {} },
                CurrentView::Groups => rsx! { GroupSidebar {} },
                CurrentView::Broadcast => rsx! { BroadcastSidebar {} },
                CurrentView::Automations => rsx! { AutomationSidebar {} },
            }
            NewChatModal {}
        }
    }
}

#[component]
fn ChatSidebar() -> Element {
    let chats = CHATS();
    let active = ACTIVE_CHAT();
    let filter = CHAT_FILTER();

    let filtered: Vec<_> = chats
        .iter()
        .filter(|c| filter.matches(c.kind))
        .cloned()
        .collect();

    rsx! {
        header { class: "sidebar-header",
            div { class: "sidebar-brand",
                span { class: "brand-icon", "✦" }
                h2 { "Chats" }
            }
            button {
                class: "icon-button new-chat-button",
                title: "New chat",
                onclick: move |_| SHOW_NEW_CHAT.with_mut(|s| *s = true),
                "＋"
            }
        }
        div { class: "filter-tabs",
            for tab in ChatFilter::ALL {
                button {
                    class: if filter == tab { "filter-tab active" } else { "filter-tab" },
                    onclick: move |_| CHAT_FILTER.with_mut(|f| *f = tab),
                    "{tab.label()}"
                }
            }
        }
        ul { class: "chat-list",
            if filtered.is_empty() {
                li { class: "chat-list-empty",
                    p { "No chats in this category yet." }
                }
            } else {
                for chat in filtered {
                    SidebarChatItem {
                        chat: chat.clone(),
                        active: active.as_deref() == Some(chat.jid.as_str()),
                    }
                }
            }
        }
    }
}

#[component]
fn SidebarChatItem(chat: Chat, active: bool) -> Element {
    let jid = chat.jid.clone();
    let name = chat.name.clone();
    let preview = chat
        .last_message
        .as_ref()
        .map(|m| {
            if m.from_me {
                format!("You: {}", m.text)
            } else {
                m.text.clone()
            }
        })
        .unwrap_or_default();
    let time = chat.last_message.as_ref().map(|m| {
        js_sys::Date::new(&js_sys::Number::from(m.timestamp_ms as f64))
            .to_locale_time_string("en-GB")
    });
    let (icon, kind_label, badge_class) = kind_meta(chat.kind);

    rsx! {
        li {
            class: if active { "chat-item active" } else { "chat-item" },
            onclick: move |_| {
                ACTIVE_CHAT.with_mut(|c| *c = Some(jid.clone()));
                CURRENT_VIEW.with_mut(|v| *v = CurrentView::Chats);
            },
            div { class: "chat-item-avatar", "{icon}" }
            div { class: "chat-item-body",
                div { class: "chat-item-row",
                    div { class: "chat-item-name", "{name}" }
                    if let Some(t) = time {
                        div { class: "chat-item-time", "{t}" }
                    }
                }
                div { class: "chat-item-row",
                    div { class: "chat-item-preview", "{preview}" }
                    span { class: "chat-kind-badge {badge_class}", "{kind_label}" }
                }
            }
        }
    }
}

fn kind_meta(kind: ChatKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        ChatKind::Group => ("👥", "Group", "kind-group"),
        ChatKind::Private => ("👤", "Private", "kind-private"),
        ChatKind::Broadcast => ("📢", "Broadcast", "kind-broadcast"),
        ChatKind::Status => ("🟢", "Status", "kind-status"),
        ChatKind::Newsletter => ("📰", "Newsletter", "kind-newsletter"),
        ChatKind::Unknown => ("💬", "Chat", "kind-unknown"),
    }
}

#[component]
fn ContactSidebar() -> Element {
    let groups = CONTACT_GROUPS();
    let contacts = CONTACTS();

    rsx! {
        header { class: "sidebar-header",
            div { class: "sidebar-brand",
                span { class: "brand-icon", "👤" }
                h2 { "Contacts" }
            }
        }
        div { class: "sidebar-section",
            h4 { "Labels" }
            ul { class: "context-list",
                li { class: "context-item active",
                    span { class: "context-icon", "👤" }
                    span { class: "context-label", "All contacts" }
                    span { class: "context-count", "{contacts.len()}" }
                }
                for group in groups {
                    li { class: "context-item",
                        span { class: "context-icon", "🏷" }
                        span { class: "context-label", "{group.name}" }
                        span { class: "context-count", "{group.contact_ids.len()}" }
                    }
                }
            }
        }
    }
}

#[component]
fn GroupSidebar() -> Element {
    let groups = CONTACT_GROUPS();

    rsx! {
        header { class: "sidebar-header",
            div { class: "sidebar-brand",
                span { class: "brand-icon", "👥" }
                h2 { "Groups" }
            }
        }
        div { class: "sidebar-section",
            h4 { "Contact groups" }
            if groups.is_empty() {
                p { class: "sidebar-empty", "No groups yet. Create one to broadcast to many contacts at once." }
            } else {
                ul { class: "context-list",
                    for group in groups {
                        SidebarGroupItem { group: group.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn SidebarGroupItem(group: ContactGroup) -> Element {
    rsx! {
        li { class: "context-item",
            span { class: "context-icon", "👥" }
            span { class: "context-label", "{group.name}" }
            span { class: "context-count", "{group.contact_ids.len()}" }
        }
    }
}

#[component]
fn BroadcastSidebar() -> Element {
    rsx! {
        header { class: "sidebar-header",
            div { class: "sidebar-brand",
                span { class: "brand-icon", "📢" }
                h2 { "Broadcast" }
            }
        }
        div { class: "sidebar-section",
            p { class: "sidebar-empty",
                "Choose contact groups or enter phone numbers. Each recipient receives an individual message."
            }
        }
    }
}

#[component]
fn AutomationSidebar() -> Element {
    rsx! {
        header { class: "sidebar-header",
            div { class: "sidebar-brand",
                span { class: "brand-icon", "⚙" }
                h2 { "Automations" }
            }
        }
        div { class: "sidebar-section",
            p { class: "sidebar-empty",
                "Create scheduled messages, auto-replies, forwarders, and webhooks. Schedulers run every minute."
            }
        }
    }
}

#[component]
fn NewChatModal() -> Element {
    use crate::api;
    use whatsapp_api_types::domain::automation::{phone_to_jid, phones_to_jids};

    let show = SHOW_NEW_CHAT();
    let mut tab = use_signal(|| NewChatTab::Private);
    let mut phone = use_signal(String::new);
    let mut phones_text = use_signal(String::new);
    let mut group_jid = use_signal(String::new);
    let mut first_message = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    if !show {
        return rsx! {};
    }

    let close = move |_| {
        SHOW_NEW_CHAT.with_mut(|s| *s = false);
        phone.set(String::new());
        phones_text.set(String::new());
        group_jid.set(String::new());
        first_message.set(String::new());
        error.set(None);
    };

    let on_submit = move |ev: dioxus::events::FormEvent| {
        ev.prevent_default();
        error.set(None);
        let msg = first_message.read().trim().to_string();
        let current_tab = *tab.read();
        spawn(async move {
            let result: Result<(), String> = async move {
                match current_tab {
                    NewChatTab::Private => {
                        let jid = phone_to_jid(&phone.read()).ok_or_else(|| {
                            "Invalid phone number. Include country code, e.g. 6281234567890.".to_string()
                        })?;
                        api::client::ensure_chat(&jid, None).await.ok();
                        if !msg.is_empty() {
                            api::client::send_text(&jid, &msg).await.map_err(|e| e.to_string())?;
                        }
                        ACTIVE_CHAT.with_mut(|c| *c = Some(jid));
                    }
                    NewChatTab::Broadcast => {
                        let lines: Vec<String> = phones_text
                            .read()
                            .lines()
                            .map(|l| l.trim().to_string())
                            .filter(|l| !l.is_empty())
                            .collect();
                        let jids = phones_to_jids(&lines);
                        if jids.is_empty() {
                            return Err("Enter at least one valid phone number.".to_string());
                        }
                        for jid in &jids {
                            api::client::ensure_chat(jid, None).await.ok();
                        }
                        if !msg.is_empty() {
                            for jid in &jids {
                                api::client::send_text(jid, &msg).await.map_err(|e| e.to_string())?;
                            }
                        }
                        if let Some(first) = jids.first() {
                            ACTIVE_CHAT.with_mut(|c| *c = Some(first.clone()));
                        }
                    }
                    NewChatTab::Group => {
                        let jid = group_jid.read().trim().to_string();
                        if jid.is_empty() || !jid.contains('@') {
                            return Err("Enter a valid group JID, e.g. group-id@g.us.".to_string());
                        }
                        api::client::ensure_chat(&jid, None).await.ok();
                        if !msg.is_empty() {
                            api::client::send_text(&jid, &msg).await.map_err(|e| e.to_string())?;
                        }
                        ACTIVE_CHAT.with_mut(|c| *c = Some(jid));
                    }
                }
                Ok(())
            }.await;
            match result {
                Ok(()) => {
                    SHOW_NEW_CHAT.with_mut(|s| *s = false);
                    phone.set(String::new());
                    phones_text.set(String::new());
                    group_jid.set(String::new());
                    first_message.set(String::new());
                }
                Err(err) => {
                    error.set(Some(err));
                }
            }
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-backdrop", onclick: close }
            div { class: "modal-panel",
                div { class: "modal-header",
                    h3 { "New chat" }
                    button { class: "icon-button", onclick: close, "✕" }
                }
                div { class: "modal-body",
                    div { class: "tab-bar",
                        for t in NewChatTab::ALL {
                            button {
                                class: if *tab.read() == t { "tab active" } else { "tab" },
                                onclick: move |_| tab.set(t),
                                "{t.label()}"
                            }
                        }
                    }
                    form { onsubmit: on_submit,
                        match *tab.read() {
                            NewChatTab::Private => rsx! {
                                label { class: "modal-label", "Phone number" }
                                input {
                                    class: "modal-input",
                                    value: phone(),
                                    placeholder: "+62 812-3456-7890",
                                    oninput: move |ev| phone.set(ev.value()),
                                }
                                p { class: "modal-hint", "Include country code. Spaces, dashes and + are ignored." }
                            },
                            NewChatTab::Broadcast => rsx! {
                                label { class: "modal-label", "Phone numbers (one per line)" }
                                textarea {
                                    class: "modal-textarea",
                                    value: phones_text(),
                                    placeholder: "+62 812-3456-7890\n+1 415-555-2671",
                                    oninput: move |ev| phones_text.set(ev.value()),
                                }
                                p { class: "modal-hint", "Each line becomes a separate private chat. Messages are sent individually." }
                            },
                            NewChatTab::Group => rsx! {
                                label { class: "modal-label", "Group JID" }
                                input {
                                    class: "modal-input",
                                    value: group_jid(),
                                    placeholder: "group-id@g.us",
                                    oninput: move |ev| group_jid.set(ev.value()),
                                }
                                p { class: "modal-hint", "Paste the group JID (ends with @g.us)." }
                            },
                        }
                        label { class: "modal-label", "First message (optional)" }
                        input {
                            class: "modal-input",
                            value: first_message(),
                            placeholder: "Type a message…",
                            oninput: move |ev| first_message.set(ev.value()),
                        }
                        if let Some(err) = error() {
                            div { class: "modal-error", "{err}" }
                        }
                        div { class: "modal-actions",
                            button { class: "modal-secondary", onclick: close, "Cancel" }
                            button { class: "modal-primary", r#type: "submit", "Start chat" }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NewChatTab {
    Private,
    Broadcast,
    Group,
}

impl NewChatTab {
    const ALL: [Self; 3] = [Self::Private, Self::Broadcast, Self::Group];

    fn label(self) -> &'static str {
        match self {
            Self::Private => "Private",
            Self::Broadcast => "Broadcast",
            Self::Group => "Group",
        }
    }
}
