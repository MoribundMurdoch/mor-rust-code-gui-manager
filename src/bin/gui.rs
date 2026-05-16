#![allow(non_snake_case)]

use dioxus::html::HasFileData;
use dioxus::prelude::*;
use dioxus_desktop::muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use dioxus_desktop::{Config, WindowBuilder};
use futures_util::StreamExt;
use std::time::Duration;

use mor_rust_code_gui_manager::app_state::{
    ActiveEditorFile, ActiveSidebarTab, ActiveWorkspacePath, AppMode, EditorSaveRequest, GraphDepth, GraphViewMode,
    PluginFindReplaceOpen, PluginNodeEditorOpen, PluginWikiSearchOpen, SyncLogEntries,
    TocSyncTrigger,
};
use mor_rust_code_gui_manager::ui::editor::CodeEditor;
use mor_rust_code_gui_manager::ui::graph::ModuleTree;
use mor_rust_code_gui_manager::ui::plugins::find_replace::FindReplacePlugin;
use mor_rust_code_gui_manager::ui::plugins::wiki_search::WikiSearchPlugin;
use mor_rust_code_gui_manager::ui::plugins::node_editor::NodeEditorPlugin; // <-- Expose the Node Editor plugin
use mor_rust_code_gui_manager::ui::sidebar::RuneLiteSidebar;

const PURPLE_INK_CSS: &str = include_str!("../../assets/purple_ink.css");

fn main() {
    // Native top-level menu bar.
    let menu_bar = Menu::new();

    // Standard File menu.
    let file_menu = Submenu::new("File", true);

    let save_item =
        MenuItem::with_id("menu_save_file", "Save File", true, None);
    let close_editor_item =
        MenuItem::with_id("menu_close_editor", "Close Editor", true, None);

    let _ = file_menu.append(&save_item);
    let _ = file_menu.append(&close_editor_item);
    let _ = file_menu.append(&PredefinedMenuItem::separator());
    let _ = file_menu.append(&PredefinedMenuItem::quit(None));

    // Standard Edit menu.
    // This helps native copy/paste/select-all behavior continue working.
    let edit_menu = Submenu::new("Edit", true);
    let _ = edit_menu.append(&PredefinedMenuItem::undo(None));
    let _ = edit_menu.append(&PredefinedMenuItem::redo(None));
    let _ = edit_menu.append(&PredefinedMenuItem::separator());
    let _ = edit_menu.append(&PredefinedMenuItem::cut(None));
    let _ = edit_menu.append(&PredefinedMenuItem::copy(None));
    let _ = edit_menu.append(&PredefinedMenuItem::paste(None));
    let _ = edit_menu.append(&PredefinedMenuItem::select_all(None));

    // Custom Moribund Tools menu.
    let tools_menu = Submenu::new("Moribund Tools", true);

    let find_replace_item =
        MenuItem::with_id("menu_toggle_find", "Toggle Find/Replace Panel", true, None);

    let wiki_search_item =
        MenuItem::with_id("menu_toggle_wiki", "Open Wiki Search", true, None);

    let _ = tools_menu.append(&find_replace_item);
    let _ = tools_menu.append(&wiki_search_item);

    // Assemble the menu bar.
    let _ = menu_bar.append(&file_menu);
    let _ = menu_bar.append(&edit_menu);
    let _ = menu_bar.append(&tools_menu);

    let window = WindowBuilder::new()
        .with_title("Mor Rust GUI Manager")
        .with_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(
            1200.0,
            800.0,
        ));

    let config = Config::default()
        .with_window(window)
        .with_menu(menu_bar);

    LaunchBuilder::new()
        .with_cfg(config)
        .launch(App);
}

fn App() -> Element {
    // Disable the native Chromium/WebView right-click menu app-wide.
    //
    // This lets the app's own custom context menus own the right-click surface
    // instead of fighting Back / Forward / Reload / Inspect Element.
    //
    // The window flag prevents duplicate listeners from stacking across rerenders.
    let _disable_native_context_menu = eval(r#"
        if (!window.__mor_context_menu_disabled) {
            window.__mor_context_menu_disabled = true;

            window.addEventListener('contextmenu', (event) => {
                event.preventDefault();
            }, { capture: true });
        }
    "#);

    let mut sync_logs = use_signal(|| {
        vec![
            String::from("[SYSTEM] Initializing Mor Rust Code GUI Manager..."),
            String::from("[SYSTEM] Applying Purple Ink aesthetics..."),
            String::from("[SYSTEM] Standing by for filesystem watcher."),
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

    // RuneLite-style sidebar tab state.
    // Use ControlPanel while building so the sidebar opens visibly.
    let active_sidebar_tab = use_signal(|| ActiveSidebarTab::ControlPanel);

    // Quest Log sync tripwire.
    // Watcher messages can increment this to make QuestLogPanel re-read MOR_PLAN.md.
    let toc_sync_trigger = use_signal(|| 0usize);

    // Internal editor active file state.
    // None means the graph owns the main viewport.
    // Some(path) means the internal editor owns the main viewport.
    let mut active_editor_file = use_signal(|| None::<std::path::PathBuf>);

    // Native File > Save and Ctrl+S tripwire for the embedded editor.
    let mut editor_save_request = use_signal(|| 0usize);

    // Plugin Hub state: controls whether floating plugin widgets are visible.
    let mut find_replace_open = use_signal(|| false);
    let mut wiki_search_open = use_signal(|| false);
    let node_editor_open = use_signal(|| false);

    // Bind the coroutine to `logger` so controls and drop events can send UI log messages.
    let logger = use_coroutine(|mut rx: UnboundedReceiver<String>| async move {
        while let Some(event_msg) = rx.next().await {
            sync_logs.write().push(event_msg);
        }
    });

    use_context_provider(|| ActiveWorkspacePath(active_path));
    use_context_provider(|| logger);
    use_context_provider(|| app_mode);
    use_context_provider(|| GraphViewMode(view_mode));
    use_context_provider(|| GraphDepth(depth));
    use_context_provider(|| active_sidebar_tab);
    use_context_provider(|| SyncLogEntries(sync_logs));
    use_context_provider(|| TocSyncTrigger(toc_sync_trigger));
    use_context_provider(|| ActiveEditorFile(active_editor_file));
    use_context_provider(|| EditorSaveRequest(editor_save_request));
    use_context_provider(|| PluginFindReplaceOpen(find_replace_open));
    use_context_provider(|| PluginWikiSearchOpen(wiki_search_open));
    use_context_provider(|| PluginNodeEditorOpen(node_editor_open));

    // Native menu click bridge.
    //
    // muda's recv() is blocking, so do not use recv().await and do not run
    // recv() directly in use_future.
    //
    // Also do not move Dioxus signals/coroutines into std::thread::spawn,
    // because Dioxus 0.5 signal storage is not Send.
    //
    // Instead, poll with try_recv(), then yield with a short async sleep.
    use_future(move || async move {
        let menu_channel = dioxus_desktop::muda::MenuEvent::receiver();

        loop {
            while let Ok(event) = menu_channel.try_recv() {
                match event.id.0.as_str() {
                    "menu_toggle_find" => {
                        let current = *find_replace_open.read();
                        find_replace_open.set(!current);
                    }
                    "menu_toggle_wiki" => {
                        wiki_search_open.set(true);
                    }
                    "menu_save_file" => {
                        editor_save_request += 1;
                    }
                    "menu_close_editor" => {
                        active_editor_file.set(None);
                    }
                    _ => {}
                }
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    rsx! {
        style { "{PURPLE_INK_CSS}" }

        div {
            class: "main-app-layout",
            prevent_default: "ondragover",
            ondragover: move |_| {},

            // Active drag-and-drop project target.
            ondrop: move |evt| {
                if let Some(file_engine) = evt.files() {
                    if let Some(dropped_path) = file_engine.files().first() {
                        active_path.set(dropped_path.clone());
                        logger.send(format!("[SYSTEM] Target dropped: {}", dropped_path));

                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
                        let watcher_logger = logger.clone();
                        let mut watcher_toc_trigger = toc_sync_trigger;

                        dioxus::prelude::spawn(async move {
                            while let Some(msg) = rx.recv().await {
                                if msg == "[SYNC_TOC]" {
                                    watcher_toc_trigger += 1;
                                } else {
                                    watcher_logger.send(msg);
                                }
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

            div {
                class: "graph-view-area",
                style: "position: relative;",

                // Conditional layout splitter stack
                if *node_editor_open.read() {
                    // 1. If the node editor is active, swap out everything inside the workspace area
                    NodeEditorPlugin {}
                } else if active_editor_file.read().is_some() {
                    // 2. Fall back to the traditional full-viewport code editor if a file is loaded
                    CodeEditor {}
                } else {
                    // 3. Fall back to standard graph view canvas layout
                    div {
                        class: "graph-toolbar",

                        div {
                            class: "view-modes",

                            button {
                                class: if *view_mode.read() == "Hierarchy" {
                                    "mode-btn active"
                                } else {
                                    "mode-btn"
                                },
                                "data-tooltip": "Visualize the structural parent-child module tree architecture.",
                                onclick: move |_| {
                                    view_mode.set("Hierarchy".to_string());
                                    logger.send(String::from("[VIEW] Switched to Hierarchy view"));
                                },
                                "Hierarchy"
                            }

                            button {
                                class: if *view_mode.read() == "Dependency" {
                                    "mode-btn active"
                                } else {
                                    "mode-btn"
                                },
                                "data-tooltip": "Track module dependencies, imports, and coupling between files.",
                                onclick: move |_| {
                                    view_mode.set("Dependency".to_string());
                                    logger.send(String::from("[VIEW] Switched to Dependency view"));
                                },
                                "Dependency"
                            }

                            button {
                                class: if *view_mode.read() == "Complexity" {
                                    "mode-btn active"
                                } else {
                                    "mode-btn"
                                },
                                "data-tooltip": "Locate high-complexity files and refactor hot spots.",
                                onclick: move |_| {
                                    view_mode.set("Complexity".to_string());
                                    logger.send(String::from("[VIEW] Switched to Complexity heatmap view"));
                                },
                                "Heatmap"
                            }

                            button {
                                class: if *view_mode.read() == "Types" {
                                    "mode-btn active"
                                } else {
                                    "mode-btn"
                                },
                                "data-tooltip": "Inspect declared structs, enums, traits, and type layouts.",
                                onclick: move |_| {
                                    view_mode.set("Types".to_string());
                                    logger.send(String::from("[VIEW] Switched to Types view"));
                                },
                                "Types"
                            }

                            button {
                                class: if *view_mode.read() == "Calls" {
                                    "mode-btn active"
                                } else {
                                    "mode-btn"
                                },
                                "data-tooltip": "Trace static function call graphs and cross-module execution routes.",
                                onclick: move |_| {
                                    view_mode.set("Calls".to_string());
                                    logger.send(String::from("[VIEW] Switched to Calls view"));
                                },
                                "Calls"
                            }

                            button {
                                class: if *view_mode.read() == "node_file_manager" {
                                    "mode-btn active"
                                } else {
                                    "mode-btn"
                                },
                                "data-tooltip": "Browse the project as interactive file nodes for inspection and safe file actions.",
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
                                    "All"
                                }
                            }
                        }

                        button {
                            class: "toolbar-action-btn",
                            "data-tooltip": "Re-run the current graph view using the latest project state.",
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
                            "Refresh View"
                        }
                    }

                    div {
                        class: "graph-canvas-wrap",
                        ModuleTree {}
                    }
                }

                // Floating OSRS-chatbox-style plugin widgets (staying layered on top).
                FindReplacePlugin {}
                WikiSearchPlugin {}
            }

            RuneLiteSidebar {}
        }
    }
}