// src/ui/sidebar.rs
#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app_state::ActiveSidebarTab;
use crate::ui::controls::ControlPanel;

fn toggle_tab(active_tab: &mut Signal<ActiveSidebarTab>, tab: ActiveSidebarTab) {
    if *active_tab.read() == tab {
        active_tab.set(ActiveSidebarTab::None);
    } else {
        active_tab.set(tab);
    }
}

pub fn RuneLiteSidebar() -> Element {
    let mut active_tab = use_context::<Signal<ActiveSidebarTab>>();

    let current_tab = *active_tab.read();

    rsx! {
        div {
            class: "runelite-sidebar-wrapper",

            if current_tab != ActiveSidebarTab::None {
                div {
                    class: "runelite-panel",

                    match current_tab {
                        ActiveSidebarTab::ControlPanel => rsx! {
                            ControlPanel {}
                        },

                        ActiveSidebarTab::QuestLog => rsx! {
                            div {
                                class: "quest-log-placeholder",
                                h3 { "Quest Log" }
                                p { "Awaiting MOR_PLAN.md parser..." }
                            }
                        },

                        ActiveSidebarTab::Automations => rsx! {
                            div {
                                class: "automations-placeholder",
                                h3 { "Global Automations" }
                                p { "Workspace scripts go here." }
                            }
                        },

                        ActiveSidebarTab::None => rsx! {}
                    }
                }
            }

            div {
                class: "runelite-icon-strip",

                button {
                    class: if current_tab == ActiveSidebarTab::ControlPanel {
                        "runelite-icon active"
                    } else {
                        "runelite-icon"
                    },
                    title: "Settings & Workspace",
                    onclick: move |_| {
                        toggle_tab(&mut active_tab, ActiveSidebarTab::ControlPanel);
                    },
                    "⚙"
                }

                button {
                    class: if current_tab == ActiveSidebarTab::QuestLog {
                        "runelite-icon active"
                    } else {
                        "runelite-icon"
                    },
                    title: "Project Blueprint (ToC)",
                    onclick: move |_| {
                        toggle_tab(&mut active_tab, ActiveSidebarTab::QuestLog);
                    },
                    "📜"
                }

                button {
                    class: if current_tab == ActiveSidebarTab::Automations {
                        "runelite-icon active"
                    } else {
                        "runelite-icon"
                    },
                    title: "Automations",
                    onclick: move |_| {
                        toggle_tab(&mut active_tab, ActiveSidebarTab::Automations);
                    },
                    "⚡"
                }
            }
        }
    }
}
