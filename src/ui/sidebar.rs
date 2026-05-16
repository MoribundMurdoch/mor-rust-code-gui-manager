// src/ui/sidebar.rs
#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app_state::{ActiveSidebarTab, SyncLogEntries};
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
    let sync_logs = use_context::<SyncLogEntries>().0;

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

                                h3 { "Layout Tips" }

                                p {
                                    "RuneLite-style interaction goal:"
                                }

                                ul {
                                    li { "Hold Alt to move eligible widgets." }
                                    li { "While holding Alt, click and drag a widget to reposition it." }
                                    li { "Sidebar tabs open, close, and swap panels from the icon strip." }
                                    li { "Inspect Mode is for safe read-only exploration." }
                                    li { "Automate Mode is for approved preview/apply actions." }
                                }

                                p {
                                    class: "sidebar-note",
                                    "Alt-drag widget movement is a planned interaction pattern, not fully implemented yet."
                                }
                            }
                        },

                        ActiveSidebarTab::Automations => rsx! {
                            div {
                                class: "automations-placeholder",
                                h3 { "Global Automations" }
                                p { "Workspace scripts go here." }
                            }
                        },

                        ActiveSidebarTab::SyncLog => rsx! {
                            div {
                                class: "sync-log-sidebar-panel",

                                h3 { "FileSystem Sync Log" }

                                div {
                                    class: "runelite-log-window",

                                    for log in sync_logs.read().iter() {
                                        div {
                                            class: "log-entry",
                                            "{log}"
                                        }
                                    }
                                }
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
                    title: "Project Blueprint",
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

                button {
                    class: if current_tab == ActiveSidebarTab::SyncLog {
                        "runelite-icon active"
                    } else {
                        "runelite-icon"
                    },
                    title: "FileSystem Sync Log",
                    onclick: move |_| {
                        toggle_tab(&mut active_tab, ActiveSidebarTab::SyncLog);
                    },
                    "☷"
                }
            }
        }
    }
}