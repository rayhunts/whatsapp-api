use dioxus::prelude::*;
use whatsapp_api_types::domain::contact::{BroadcastDelay, BroadcastRequest, ContactGroup};

use crate::api;
use crate::state::app_state::{CONTACT_GROUPS, CURRENT_VIEW, CurrentView};

#[component]
fn TargetOption(group: ContactGroup, checked: bool, on_toggle: EventHandler<()>) -> Element {
    rsx! {
        label { class: "target-option",
            input {
                r#type: "checkbox",
                checked: checked,
                onchange: move |_| on_toggle.call(()),
            }
            span { "{group.name}" }
            span { class: "target-count", "{group.contact_ids.len()}" }
        }
    }
}


#[component]
pub fn BroadcastView() -> Element {
    let _ = use_effect(move || {
        spawn(async move {
            if let Ok(list) = api::client::list_contact_groups().await {
                CONTACT_GROUPS.with_mut(|g| *g = list);
            }
        });
    });

    let mut selected_group_ids = use_signal(Vec::<String>::new);
    let mut manual_numbers = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut use_delay = use_signal(|| false);
    let mut delay_min = use_signal(|| 1000u64);
    let mut delay_max = use_signal(|| 3000u64);
    let mut sending = use_signal(|| false);
    let mut result = use_signal(|| None::<Result<String, String>>);

    let estimated = selected_group_ids.read().len() + manual_numbers.read().lines().filter(|l| !l.trim().is_empty()).count();

    let submit = move |ev: dioxus::events::FormEvent| {
        ev.prevent_default();
        result.set(None);
        let msg = message.read().trim().to_string();
        if msg.is_empty() {
            result.set(Some(Err("Message is required.".into())));
            return;
        }
        let group_ids: Vec<String> = selected_group_ids.read().clone();
        let to: Vec<String> = manual_numbers
            .read()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        if group_ids.is_empty() && to.is_empty() {
            result.set(Some(Err("Select at least one group or enter a phone number.".into())));
            return;
        }
        let delay = if use_delay() {
            Some(BroadcastDelay {
                min: delay_min(),
                max: delay_max(),
            })
        } else {
            None
        };
        let req = BroadcastRequest {
            group_ids,
            to,
            message: msg,
            delay_ms: delay,
        };
        sending.set(true);
        spawn(async move {
            match api::client::broadcast(&req).await {
                Ok(resp) => {
                    result.set(Some(Ok(format!("Queued {} messages.", resp.total))));
                    message.set(String::new());
                    manual_numbers.set(String::new());
                    selected_group_ids.set(Vec::new());
                }
                Err(e) => result.set(Some(Err(e.to_string()))),
            }
            sending.set(false);
        });
    };

    let mut toggle_group = move |id: String| {
        selected_group_ids.with_mut(|ids| {
            if let Some(pos) = ids.iter().position(|i| i == &id) {
                ids.remove(pos);
            } else {
                ids.push(id);
            }
        });
    };

    rsx! {
        main { class: "main-view",
            header { class: "view-header",
                div { class: "view-header-info",
                    h2 { "Broadcast" }
                    p { class: "view-header-subtitle", "Send one message to many recipients individually." }
                }
            }
            form { class: "broadcast-form", onsubmit: submit,
                div { class: "broadcast-section",
                    h3 { "Message" }
                    textarea {
                        class: "modal-textarea",
                        value: message(),
                        placeholder: "Type the message you want to send…",
                        oninput: move |ev| message.set(ev.value()),
                    }
                }
                div { class: "broadcast-section",
                    h3 { "Targets" }
                    div { class: "form-row",
                        label { "Contact groups" }
                        if CONTACT_GROUPS().is_empty() {
                            p { class: "modal-hint",
                                "No groups yet. "
                                button {
                                    class: "inline-link",
                                    r#type: "button",
                                    onclick: move |_| CURRENT_VIEW.with_mut(|v| *v = CurrentView::Groups),
                                    "Create a group"
                                }
                                " first."
                            }
                        } else {
                            div { class: "target-picker",
                                for group in CONTACT_GROUPS() {
                                    TargetOption {
                                        group: group.clone(),
                                        checked: selected_group_ids.read().contains(&group.id),
                                        on_toggle: move |_| toggle_group(group.id.clone()),
                                    }
                                }
                            }
                        }
                    }
                    div { class: "form-row",
                        label { "Or enter phone numbers / JIDs (one per line)" }
                        textarea {
                            class: "modal-textarea",
                            value: manual_numbers(),
                            placeholder: "628123456789\ngroup-id@g.us",
                            oninput: move |ev| manual_numbers.set(ev.value()),
                        }
                        p { class: "modal-hint", "Phone numbers need a country code. Group JIDs must end with @g.us." }
                    }
                }
                div { class: "broadcast-section",
                    div { class: "form-row check-row",
                        input {
                            r#type: "checkbox",
                            checked: use_delay(),
                            onchange: move |ev| use_delay.set(ev.checked()),
                        }
                        label { "Add random delay between messages" }
                    }
                    if use_delay() {
                        div { class: "delay-row",
                            div { class: "form-row",
                                label { "Min (ms)" }
                                input {
                                    class: "modal-input",
                                    r#type: "number",
                                    value: delay_min().to_string(),
                                    oninput: move |ev| {
                                        if let Ok(v) = ev.value().parse() {
                                            delay_min.set(v);
                                        }
                                    },
                                }
                            }
                            div { class: "form-row",
                                label { "Max (ms)" }
                                input {
                                    class: "modal-input",
                                    r#type: "number",
                                    value: delay_max().to_string(),
                                    oninput: move |ev| {
                                        if let Ok(v) = ev.value().parse() {
                                            delay_max.set(v);
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
                if let Some(res) = result() {
                    div {
                        class: if res.is_ok() { "broadcast-result success" } else { "broadcast-result error" },
                        "{res.as_ref().unwrap_or(&String::new())}"
                    }
                }
                div { class: "broadcast-actions",
                    span { class: "broadcast-estimate", "Estimated recipients: {estimated}" }
                    button {
                        class: "modal-primary",
                        r#type: "submit",
                        disabled: sending(),
                        if sending() { "Sending…" } else { "Send broadcast" }
                    }
                }
            }
        }
    }
}
