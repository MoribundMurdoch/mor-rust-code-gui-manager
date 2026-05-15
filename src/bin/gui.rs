#![allow(non_snake_case)]
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use futures_util::StreamExt;

use mor_rust_code_gui_manager::app_state::{
    ActiveWorkspacePath, AppMode, GraphDepth, GraphViewMode,
};
use mor_rust_code_gui_manager::ui::controls::ControlPanel;
use mor_rust_code_gui_manager::ui::graph::ModuleTree;
use mor_rust_code_gui_manager::ui::mode_switch::ModeSwitch;

#[derive(Clone, Debug)]
pub struct SyncEvent(String);

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let mut sync_logs = use_signal(|| {
        vec![
            SyncEvent(String::from(
                "[SYSTEM] Initializing Mor Rust Code GUI Manager...",
            )),
            SyncEvent(String::from("[SYSTEM] Applying Purple Ink aesthetics...")),
            SyncEvent(String::from("[SYSTEM] Standing by for filesystem watcher.")),
        ]
    });

    let mut active_path = use_signal(|| {
    std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| String::from("Awaiting filesystem initialization..."))
});

    // Global safety switch state.
    // The app always boots in read-only Inspect Mode.
    let app_mode = use_signal(|| AppMode::Inspect);

    // Visualization control state.
    let mut view_mode = use_signal(|| "Hierarchy".to_string());
    let mut depth = use_signal(|| 3i32);

    // Bind the coroutine to `logger` so controls and drop events can send UI log messages.
    let logger = use_coroutine(|mut rx: UnboundedReceiver<String>| async move {
        while let Some(event_msg) = rx.next().await {
            sync_logs.write().push(SyncEvent(event_msg));
        }
    });

    use_context_provider(|| ActiveWorkspacePath(active_path));
    use_context_provider(|| logger);
    use_context_provider(|| app_mode);
    use_context_provider(|| GraphViewMode(view_mode));
    use_context_provider(|| GraphDepth(depth));

    const PURPLE_INK_CSS: &str = include_str!("../../assets/purple_ink.css");

    rsx! {
        style { "{PURPLE_INK_CSS}" }

        div {
            id: "mor-manager-layout",
            prevent_default: "ondragover",
            ondragover: move |_| {},

            // Active drag-and-drop project target.
            ondrop: move |evt| {
                if let Some(file_engine) = evt.files() {
                    if let Some(dropped_path) = file_engine.files().first() {
                        active_path.set(dropped_path.clone());
                        logger.send(format!("[SYSTEM] Target dropped: {}", dropped_path));

                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                        let watcher_logger = logger.clone();

                        dioxus::prelude::spawn(async move {
                            while let Some(msg) = rx.recv().await {
                                watcher_logger.send(msg);
                            }
                        });

                        let path_buf = std::path::PathBuf::from(&dropped_path);

                        if let Err(e) = mor_rust_code_gui_manager::watcher::spawn_watcher(
    path_buf,
    tx,
    app_mode.read().clone(),
) {
                            logger.send(format!("[FATAL] Watcher fault: {}", e));
                        }
                    }
                }
            },

            div { id: "left-panel",
                h2 { "Architectural Controls" }

                ModeSwitch {}

                ControlPanel {}
            }

            div { id: "center-panel",
                h2 { "Rust Module Visualization & Etymology" }

                div {
                    class: "graph-toolbar",
                    style: "display: flex; align-items: center; gap: 12px; padding: 8px 16px; background: #1e1e24; border-bottom: 1px solid #3c2a5d; margin-bottom: 4px;",

                    div {
                        class: "view-modes",
                        style: "display: flex; background: #161619; padding: 3px; gap: 3px;",

                        button {
                            class: "mode-btn",
                            style: if *view_mode.read() == "Hierarchy" {
                                "background: #9d72ff; color: #ffffff;"
                            } else {
                                ""
                            },
                            onclick: move |_| {
                                view_mode.set("Hierarchy".to_string());
                                logger.send(String::from("[VIEW] Switched to Hierarchy view"));
                            },
                            "Hierarchy"
                        }

                        button {
                            class: "mode-btn",
                            style: if *view_mode.read() == "Dependency" {
                                "background: #9d72ff; color: #ffffff;"
                            } else {
                                ""
                            },
                            onclick: move |_| {
                                view_mode.set("Dependency".to_string());
                                logger.send(String::from("[VIEW] Switched to Dependency view"));
                            },
                            "Dependency"
                        }

                        button {
                            class: "mode-btn",
                            style: if *view_mode.read() == "Complexity" {
                                "background: #9d72ff; color: #ffffff;"
                            } else {
                                ""
                            },
                            onclick: move |_| {
                                view_mode.set("Complexity".to_string());
                                logger.send(String::from("[VIEW] Switched to Complexity heatmap view"));
                            },
                            "Heatmap"
                        }

                        button {
                            class: "mode-btn",
                            style: if *view_mode.read() == "Types" {
                                "background: #9d72ff; color: #ffffff;"
                            } else {
                                ""
                            },
                            onclick: move |_| {
                                view_mode.set("Types".to_string());
                                logger.send(String::from("[VIEW] Switched to Types view"));
                            },
                            "Types"
                        }

                        button {
                            class: "mode-btn",
                            style: if *view_mode.read() == "Calls" {
                                "background: #9d72ff; color: #ffffff;"
                            } else {
                                ""
                            },
                            onclick: move |_| {
                                view_mode.set("Calls".to_string());
                                logger.send(String::from("[VIEW] Switched to Calls view"));
                            },
                            "Calls"
                        }

                        button {
                            class: "mode-btn",
                            style: if *view_mode.read() == "node_file_manager" {
                                "background: #9d72ff; color: #ffffff;"
                            } else {
                                ""
                            },
                            onclick: move |_| {
                                view_mode.set("node_file_manager".to_string());
                                logger.send(String::from("[VIEW] Switched to Node File Manager"));
                            },
                            "Node File Manager"
                        }
                    }

                    if *view_mode.read() == "Hierarchy" {
                        div {
                            class: "depth-control",
                            style: "display: flex; align-items: center; gap: 8px; color: #a9a9b3; font-size: 13px;",

                            span { "Depth" }

                            input {
                                r#type: "range",
                                min: "2",
                                max: "10",
                                value: "{depth.read()}",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<i32>() {
                                        depth.set(val);
                                    }
                                },
                                style: "accent-color: #9d72ff; width: 140px;",
                            }

                            span {
                                if *depth.read() >= 999 {
                                    "All"
                                } else {
                                    "{depth.read()}"
                                }
                            }

                            button {
                                onclick: move |_| {
                                    depth.set(999);
                                    logger.send(String::from("[VIEW] Hierarchy depth set to All"));
                                },
                                style: "font-size: 11px; padding: 2px 8px;",
                                "All"
                            }
                        }
                    }

                    button {
                        onclick: move |_| {
                            logger.send(format!(
                                "[VIEW] Refresh requested: {} view, depth {}",
                                view_mode.read(),
                                if *depth.read() >= 999 {
                                    "All".to_string()
                                } else {
                                    depth.read().to_string()
                                }
                            ));
                        },
                        style: "margin-left: auto; background: #3c2a5d; color: #e0e0e0; border: none; padding: 4px 12px; cursor: pointer;",
                        "Refresh View"
                    }
                }

                ModuleTree {}
            }

            div { id: "right-panel",
                h2 { "FileSystem Sync Log" }

                div { class: "log-window",
                    for log in sync_logs.read().iter() {
                        div { class: "log-entry", "{log.0}" }
                    }
                }
            }
        }
    }
}
