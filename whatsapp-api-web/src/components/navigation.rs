use dioxus::prelude::*;

use crate::state::app_state::{CURRENT_VIEW, CurrentView, SHOW_SETTINGS};

#[component]
pub fn NavigationRail() -> Element {
    rsx! {
        nav { class: "navigation-rail",
            div { class: "rail-brand",
                span { class: "rail-brand-icon", "✦" }
            }
            ul { class: "rail-items",
                for view in CurrentView::ALL {
                    li { class: "rail-item",
                        button {
                            class: if CURRENT_VIEW() == view { "rail-button active" } else { "rail-button" },
                            title: "{view.label()}",
                            onclick: move |_| CURRENT_VIEW.with_mut(|v| *v = view),
                            span { class: "rail-icon", "{view.icon()}" }
                            span { class: "rail-label", "{view.label()}" }
                        }
                    }
                }
            }
            div { class: "rail-footer",
                button {
                    class: "rail-button",
                    title: "Settings",
                    onclick: move |_| SHOW_SETTINGS.with_mut(|s| *s = true),
                    span { class: "rail-icon", "⚙" }
                    span { class: "rail-label", "Settings" }
                }
            }
        }
    }
}
