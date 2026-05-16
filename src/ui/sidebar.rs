// src/ui/sidebar.rs
#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app_state::{
    ActiveSidebarTab,
    PluginFindReplaceOpen,
    PluginNodeEditorOpen,
    PluginWikiSearchOpen,
    SyncLogEntries,
};
use crate::ui::controls::ControlPanel;
use crate::ui::toc_panel::QuestLogPanel;

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

    let mut find_replace_open = use_context::<PluginFindReplaceOpen>().0;
    let mut wiki_open = use_context::<PluginWikiSearchOpen>().0;
    let mut node_editor_open = use_context::<PluginNodeEditorOpen>().0;

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
                            QuestLogPanel {}
                        },

                        ActiveSidebarTab::Automations => rsx! {
                            div {
                                class: "automations-placeholder",
                                h3 { "Global Automations" }
                                p { "Workspace scripts go here." }
                            }
                        },

                        ActiveSidebarTab::PluginHub => rsx! {
                            div {
                                class: "plugin-hub-container",

                                h3 { "Plugin Hub" }

                                div {
                                    class: "runelite-plugin-item",

                                    div {
                                        class: "plugin-info",

                                        div {
                                            class: "plugin-title",
                                            "Block Find/Replace"
                                        }

                                        div {
                                            class: "plugin-desc",
                                            "Oversized widget to swap multi-line AST chunks safely."
                                        }
                                    }

                                    button {
                                        class: if *find_replace_open.read() {
                                            "plugin-toggle active"
                                        } else {
                                            "plugin-toggle"
                                        },
                                        onclick: move |_| {
                                            let current = *find_replace_open.read();
                                            find_replace_open.set(!current);
                                        },
                                        if *find_replace_open.read() {
                                            "On"
                                        } else {
                                            "Off"
                                        }
                                    }
                                }

                                div {
                                    class: "runelite-plugin-item",

                                    div {
                                        class: "plugin-info",

                                        div {
                                            class: "plugin-title",
                                            "Rust Wiki Link"
                                        }

                                        div {
                                            class: "plugin-desc",
                                            "Floatable search bar for docs.rs and std lib."
                                        }
                                    }

                                    button {
                                        class: if *wiki_open.read() {
                                            "plugin-toggle active"
                                        } else {
                                            "plugin-toggle"
                                        },
                                        onclick: move |_| {
                                            let current = *wiki_open.read();
                                            wiki_open.set(!current);
                                        },
                                        if *wiki_open.read() {
                                            "On"
                                        } else {
                                            "Off"
                                        }
                                    }
                                }

                                div {
                                    class: "runelite-plugin-item",

                                    div {
                                        class: "plugin-info",

                                        div {
                                            class: "plugin-title",
                                            "Rust Node Editor"
                                        }

                                        div {
                                            class: "plugin-desc",
                                            "Visual programming interface for Rust AST generation."
                                        }
                                    }

                                    button {
                                        class: if *node_editor_open.read() {
                                            "plugin-toggle active"
                                        } else {
                                            "plugin-toggle"
                                        },
                                        onclick: move |_| {
                                            let current = *node_editor_open.read();
                                            node_editor_open.set(!current);
                                        },
                                        if *node_editor_open.read() {
                                            "On"
                                        } else {
                                            "Off"
                                        }
                                    }
                                }
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
                    class: if current_tab == ActiveSidebarTab::PluginHub {
                        "runelite-icon active"
                    } else {
                        "runelite-icon"
                    },
                    title: "Plugin Hub",
                    onclick: move |_| {
                        toggle_tab(&mut active_tab, ActiveSidebarTab::PluginHub);
                    },
                    "🧩"
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