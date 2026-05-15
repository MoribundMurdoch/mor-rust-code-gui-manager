#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app_state::AppMode;

pub fn ModeSwitch() -> Element {
    let mut app_mode = use_context::<Signal<AppMode>>();
    let logger = use_context::<Coroutine<String>>();

    let is_inspect = app_mode.read().is_inspect();
    let is_automate = app_mode.read().is_automate();

    rsx! {
        div {
            class: "safety-switch",

            div {
                class: "safety-switch-title",
                "Safety Switch"
            }

            div {
                class: "safety-switch-buttons",

                button {
                    class: if is_inspect {
                        "safety-mode-btn active"
                    } else {
                        "safety-mode-btn"
                    },

                    onclick: move |_| {
                        app_mode.set(AppMode::Inspect);
                        logger.send(String::from(
                            "[SAFETY] Inspect Mode enabled: file writes are blocked."
                        ));
                    },

                    "Inspect"
                }

                button {
                    class: if is_automate {
                        "safety-mode-btn danger active"
                    } else {
                        "safety-mode-btn danger"
                    },

                    onclick: move |_| {
                        app_mode.set(AppMode::Automate);
                        logger.send(String::from(
                            "[SAFETY] Automate Mode enabled: approved actions may modify files."
                        ));
                    },

                    "Automate"
                }
            }

            div {
                class: "safety-switch-label",
                "{app_mode.read().label()}"
            }

            div {
                class: "safety-switch-description",
                "{app_mode.read().description()}"
            }
        }
    }
}
