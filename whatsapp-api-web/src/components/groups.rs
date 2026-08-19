use dioxus::prelude::*;
use whatsapp_api_types::domain::contact::{Contact, ContactGroup, CreateContactGroupRequest, UpdateContactGroupRequest};

use crate::api;
use crate::state::app_state::{CONTACTS, CONTACT_GROUPS, CURRENT_VIEW, CurrentView};

#[component]
pub fn GroupsView() -> Element {
    let _ = use_effect(move || {
        spawn(async move {
            if let Ok(list) = api::client::list_contact_groups().await {
                CONTACT_GROUPS.with_mut(|g| *g = list);
            }
            if let Ok(list) = api::client::list_contacts().await {
                CONTACTS.with_mut(|c| *c = list);
            }
        });
    });

    let mut show_modal = use_signal(|| false);
    let mut editing = use_signal(|| None::<ContactGroup>);

    rsx! {
        main { class: "main-view",
            header { class: "view-header",
                div { class: "view-header-info",
                    h2 { "Groups" }
                    p { class: "view-header-subtitle", "Organize contacts so you can broadcast to many people at once." }
                }
                button {
                    class: "view-primary-button",
                    onclick: move |_| {
                        editing.set(None);
                        show_modal.set(true);
                    },
                    "＋ New group"
                }
            }
            if CONTACT_GROUPS().is_empty() {
                div { class: "empty-state",
                    div { class: "empty-icon", "👥" }
                    h3 { "No groups yet" }
                    p { "Create a group from your contacts, then use it as a broadcast target." }
                    button {
                        class: "modal-primary",
                        onclick: move |_| {
                            editing.set(None);
                            show_modal.set(true);
                        },
                        "Create your first group"
                    }
                }
            } else {
                div { class: "card-list",
                    for group in CONTACT_GROUPS() {
                        GroupCard {
                            group: group.clone(),
                            on_edit: move |_| {
                                editing.set(Some(group.clone()));
                                show_modal.set(true);
                            },
                        }
                    }
                }
            }
            if show_modal() {
                GroupModal {
                    group: editing(),
                    on_close: move |_| show_modal.set(false),
                }
            }
        }
    }
}

#[component]
fn MemberOption(contact: Contact, checked: bool, on_toggle: EventHandler<()>) -> Element {
    rsx! {
        label { class: "member-option",
            input {
                r#type: "checkbox",
                checked: checked,
                onchange: move |_| on_toggle.call(()),
            }
            span { class: "member-name", "{contact.name}" }
            span { class: "member-phone", "{contact.phone}" }
        }
    }
}

#[component]
fn GroupCard(group: ContactGroup, on_edit: EventHandler<()>) -> Element {
    let group_id = group.id.clone();
    let member_names: Vec<_> = CONTACTS()
        .iter()
        .filter(|c| group.contact_ids.contains(&c.id))
        .map(|c| c.name.clone())
        .collect();

    rsx! {
        div { class: "card",
            div { class: "card-header",
                div { class: "card-title-row",
                    h3 { "{group.name}" }
                    span { class: "card-count", "{group.contact_ids.len()} members" }
                }
                p { class: "card-members",
                    if member_names.is_empty() {
                        "No members yet."
                    } else {
                        "{member_names.join(\", \")}"
                    }
                }
            }
            div { class: "card-actions",
                button {
                    class: "card-action",
                    onclick: move |_| on_edit.call(()),
                    "Edit"
                }
                button {
                    class: "card-action",
                    onclick: move |_| {
                        CURRENT_VIEW.with_mut(|v| *v = CurrentView::Broadcast);
                    },
                    "Broadcast"
                }
                button {
                    class: "card-action danger",
                    onclick: move |_| {
                        let id = group_id.clone();
                        spawn(async move {
                            if api::client::delete_contact_group(&id).await.is_ok() {
                                CONTACT_GROUPS.with_mut(|list| list.retain(|g| g.id != id));
                            }
                        });
                    },
                    "Delete"
                }
            }
        }
    }
}

#[component]
fn GroupModal(group: Option<ContactGroup>, on_close: EventHandler<()>) -> Element {
    let is_edit = group.is_some();
    let mut name = use_signal(|| group.as_ref().map(|g| g.name.clone()).unwrap_or_default());
    let mut selected_ids = use_signal(|| group.as_ref().map(|g| g.contact_ids.clone()).unwrap_or_default());
    let mut search = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let query = search.read().trim().to_lowercase();
    let available: Vec<_> = CONTACTS()
        .into_iter()
        .filter(|c| {
            query.is_empty()
                || c.name.to_lowercase().contains(&query)
                || c.phone.to_lowercase().contains(&query)
        })
        .collect();

    let close = move |_| {
        on_close.call(());
    };

    let mut toggle_member = move |id: String| {
        selected_ids.with_mut(|ids| {
            if let Some(pos) = ids.iter().position(|i| i == &id) {
                ids.remove(pos);
            } else {
                ids.push(id);
            }
        });
    };

    let submit = move |ev: dioxus::events::FormEvent| {
        ev.prevent_default();
        error.set(None);
        let name_val = name.read().trim().to_string();
        if name_val.is_empty() {
            error.set(Some("Group name is required.".into()));
            return;
        }
        let ids: Vec<String> = selected_ids.read().clone();

        if let Some(existing) = group.clone() {
            let id = existing.id.clone();
            let req = UpdateContactGroupRequest {
                name: Some(name_val),
                contact_ids: Some(ids),
            };
            spawn(async move {
                match api::client::update_contact_group(&id, &req).await {
                    Ok(updated) => {
                        CONTACT_GROUPS.with_mut(|list| {
                            if let Some(item) = list.iter_mut().find(|g| g.id == id) {
                                *item = updated;
                            }
                        });
                        on_close.call(());
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        } else {
            let req = CreateContactGroupRequest {
                name: name_val,
                contact_ids: ids,
            };
            spawn(async move {
                match api::client::create_contact_group(&req).await {
                    Ok(created) => {
                        CONTACT_GROUPS.with_mut(|list| list.push(created));
                        on_close.call(());
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        }
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-backdrop", onclick: close }
            div { class: "modal-panel modal-panel-wide",
                div { class: "modal-header",
                    h3 { if is_edit { "Edit group" } else { "New group" } }
                    button { class: "icon-button", onclick: close, "✕" }
                }
                div { class: "modal-body",
                    form { onsubmit: submit,
                        div { class: "form-row",
                            label { "Group name" }
                            input {
                                class: "modal-input",
                                value: name(),
                                placeholder: "Customers",
                                oninput: move |ev| name.set(ev.value()),
                            }
                        }
                        div { class: "form-row",
                            label { "Members ({selected_ids().len()} selected)" }
                            input {
                                class: "modal-input",
                                value: search(),
                                placeholder: "Search contacts…",
                                oninput: move |ev| search.set(ev.value()),
                            }
                            div { class: "member-picker",
                                if available.is_empty() {
                                    p { class: "modal-hint", "No contacts match your search." }
                                } else {
                                    for contact in available {
                                        MemberOption {
                                            contact: contact.clone(),
                                            checked: selected_ids.read().contains(&contact.id),
                                            on_toggle: move |_| toggle_member(contact.id.clone()),
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(err) = error() {
                            div { class: "modal-error", "{err}" }
                        }
                        div { class: "modal-actions",
                            button { class: "modal-secondary", onclick: close, "Cancel" }
                            button { class: "modal-primary", r#type: "submit", if is_edit { "Save changes" } else { "Create group" } }
                        }
                    }
                }
            }
        }
    }
}
