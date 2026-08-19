use dioxus::prelude::*;
use whatsapp_api_types::domain::contact::{Contact, CreateContactRequest, UpdateContactRequest};

use crate::api;
use crate::state::app_state::{CONTACTS, CURRENT_VIEW, CurrentView};

#[component]
pub fn ContactsView() -> Element {
    let _ = use_effect(move || {
        spawn(async move {
            if let Ok(list) = api::client::list_contacts().await {
                CONTACTS.with_mut(|c| *c = list);
            }
        });
    });

    let mut search = use_signal(String::new);
    let mut show_modal = use_signal(|| false);
    let mut editing = use_signal(|| None::<Contact>);

    let query = search.read().trim().to_lowercase();
    let filtered: Vec<_> = CONTACTS()
        .into_iter()
        .filter(|c| {
            query.is_empty()
                || c.name.to_lowercase().contains(&query)
                || c.phone.to_lowercase().contains(&query)
                || c.email.as_deref().unwrap_or("").to_lowercase().contains(&query)
        })
        .collect();

    rsx! {
        main { class: "main-view",
            header { class: "view-header",
                div { class: "view-header-info",
                    h2 { "Contacts" }
                    p { class: "view-header-subtitle", "Saved people you message regularly." }
                }
                button {
                    class: "view-primary-button",
                    onclick: move |_| {
                        editing.set(None);
                        show_modal.set(true);
                    },
                    "＋ Add contact"
                }
            }
            div { class: "view-toolbar",
                input {
                    class: "view-search",
                    value: search(),
                    placeholder: "Search by name, phone, or email…",
                    oninput: move |ev| search.set(ev.value()),
                }
            }
            if filtered.is_empty() {
                div { class: "empty-state",
                    div { class: "empty-icon", "👤" }
                    h3 { "No contacts yet" }
                    p { "Save a contact so you can start chats and add them to groups without memorizing phone numbers." }
                    button {
                        class: "modal-primary",
                        onclick: move |_| {
                            editing.set(None);
                            show_modal.set(true);
                        },
                        "Add your first contact"
                    }
                }
            } else {
                table { class: "data-table",
                    thead {
                        tr {
                            th { "Name" }
                            th { "Phone" }
                            th { "Email" }
                            th { "Notes" }
                            th { class: "actions-cell", "Actions" }
                        }
                    }
                    tbody {
                        for contact in filtered {
                            ContactRow {
                                contact: contact.clone(),
                                on_edit: move |_| {
                                    editing.set(Some(contact.clone()));
                                    show_modal.set(true);
                                },
                            }
                        }
                    }
                }
            }
            if show_modal() {
                ContactModal {
                    contact: editing(),
                    on_close: move |_| show_modal.set(false),
                }
            }
        }
    }
}

#[component]
fn ContactRow(contact: Contact, on_edit: EventHandler<()>) -> Element {
    rsx! {
        tr { key: "{contact.id}",
            td { "{contact.name}" }
            td { "{contact.phone}" }
            td { "{contact.email.as_deref().unwrap_or(\"-\")}" }
            td { "{contact.notes.as_deref().unwrap_or(\"-\")}" }
            td { class: "actions-cell",
                button {
                    class: "table-action",
                    onclick: move |_| on_edit.call(()),
                    "Edit"
                }
                button {
                    class: "table-action danger",
                    onclick: move |_| {
                        let id = contact.id.clone();
                        spawn(async move {
                            if api::client::delete_contact(&id).await.is_ok() {
                                CONTACTS.with_mut(|list| list.retain(|c| c.id != id));
                            }
                        });
                    },
                    "Delete"
                }
                button {
                    class: "table-action",
                    onclick: move |_| {
                        let phone = contact.phone.clone();
                        spawn(async move {
                            if let Some(jid) = whatsapp_api_types::domain::automation::phone_to_jid(&phone) {
                                let _ = api::client::ensure_chat(&jid, None).await;
                                crate::state::app_state::ACTIVE_CHAT.with_mut(|c| *c = Some(jid));
                                CURRENT_VIEW.with_mut(|v| *v = CurrentView::Chats);
                            }
                        });
                    },
                    "Chat"
                }
            }
        }
    }
}

#[component]
fn ContactModal(contact: Option<Contact>, on_close: EventHandler<()>) -> Element {
    let is_edit = contact.is_some();
    let mut name = use_signal(|| contact.as_ref().map(|c| c.name.clone()).unwrap_or_default());
    let mut phone = use_signal(|| contact.as_ref().map(|c| c.phone.clone()).unwrap_or_default());
    let mut email = use_signal(|| contact.as_ref().and_then(|c| c.email.clone()).unwrap_or_default());
    let mut notes = use_signal(|| contact.as_ref().and_then(|c| c.notes.clone()).unwrap_or_default());
    let mut error = use_signal(|| None::<String>);

    let close = move |_| {
        on_close.call(());
    };

    let submit = move |ev: dioxus::events::FormEvent| {
        ev.prevent_default();
        error.set(None);
        let name_val = name.read().trim().to_string();
        let phone_val = phone.read().trim().to_string();
        if name_val.is_empty() {
            error.set(Some("Name is required.".into()));
            return;
        }
        if whatsapp_api_types::domain::automation::phone_to_jid(&phone_val).is_none() {
            error.set(Some("Phone number must include country code, e.g. 6281234567890.".into()));
            return;
        }
        let email_read = email.read();
        let email_val = email_read.trim();
        let email_opt = if email_val.is_empty() { None } else { Some(email_val.to_string()) };
        drop(email_read);
        let notes_read = notes.read();
        let notes_val = notes_read.trim();
        let notes_opt = if notes_val.is_empty() { None } else { Some(notes_val.to_string()) };

        if let Some(existing) = contact.clone() {
            let id = existing.id.clone();
            let req = UpdateContactRequest {
                name: Some(name_val),
                phone: Some(phone_val),
                email: email_opt,
                notes: notes_opt,
            };
            spawn(async move {
                match api::client::update_contact(&id, &req).await {
                    Ok(updated) => {
                        CONTACTS.with_mut(|list| {
                            if let Some(item) = list.iter_mut().find(|c| c.id == id) {
                                *item = updated;
                            }
                        });
                        on_close.call(());
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        } else {
            let req = CreateContactRequest {
                name: name_val,
                phone: phone_val,
                email: email_opt,
                notes: notes_opt,
            };
            spawn(async move {
                match api::client::create_contact(&req).await {
                    Ok(created) => {
                        CONTACTS.with_mut(|list| list.push(created));
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
            div { class: "modal-panel",
                div { class: "modal-header",
                    h3 { if is_edit { "Edit contact" } else { "Add contact" } }
                    button { class: "icon-button", onclick: close, "✕" }
                }
                div { class: "modal-body",
                    form { onsubmit: submit,
                        div { class: "form-row",
                            label { "Name" }
                            input {
                                class: "modal-input",
                                value: name(),
                                placeholder: "Alice",
                                oninput: move |ev| name.set(ev.value()),
                            }
                        }
                        div { class: "form-row",
                            label { "Phone" }
                            input {
                                class: "modal-input",
                                value: phone(),
                                placeholder: "628123456789",
                                oninput: move |ev| phone.set(ev.value()),
                            }
                            p { class: "modal-hint", "Include country code. Spaces, dashes and + are ignored." }
                        }
                        div { class: "form-row",
                            label { "Email" }
                            input {
                                class: "modal-input",
                                value: email(),
                                placeholder: "alice@example.com",
                                oninput: move |ev| email.set(ev.value()),
                            }
                        }
                        div { class: "form-row",
                            label { "Notes" }
                            input {
                                class: "modal-input",
                                value: notes(),
                                placeholder: "VIP customer",
                                oninput: move |ev| notes.set(ev.value()),
                            }
                        }
                        if let Some(err) = error() {
                            div { class: "modal-error", "{err}" }
                        }
                        div { class: "modal-actions",
                            button { class: "modal-secondary", onclick: close, "Cancel" }
                            button { class: "modal-primary", r#type: "submit", if is_edit { "Save changes" } else { "Add contact" } }
                        }
                    }
                }
            }
        }
    }
}
