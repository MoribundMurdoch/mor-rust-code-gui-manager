#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::fs;
use std::path::PathBuf;

use crate::app_state::{PluginFindReplaceOpen, AppMode};

pub fn FindReplacePlugin() -> Element {
    let mut find_replace_open = use_context::<PluginFindReplaceOpen>().0;
    let app_mode = use_context::<Signal<AppMode>>();
    let logger = use_context::<Coroutine<String>>();

    let mut find_text = use_signal(|| String::new());
    let mut replace_text = use_signal(|| String::new());
    let mut target_file = use_signal(|| None::<PathBuf>);

    // --- Drag State ---
    let mut pos_x = use_signal(|| 20.0f64);
    let mut pos_y = use_signal(|| 20.0f64);
    let mut is_dragging = use_signal(|| false);
    let mut drag_offset = use_signal(|| (0.0f64, 0.0f64));

    if !*find_replace_open.read() {
        return rsx! { div {} };
    }

    let is_automate = app_mode.read().is_automate();

    rsx! {
        div {
            class: "osrs-plugin-window",
            style: "left: {pos_x}px; top: {pos_y}px;",
            
            // The wrapper tracks mouse movement, but ONLY moves the window if is_dragging is true
            onmousemove: move |evt| {
                if *is_dragging.read() {
                    let coords = evt.client_coordinates();
                    let offset = *drag_offset.read();
                    pos_x.set(coords.x - offset.0);
                    pos_y.set(coords.y - offset.1);
                }
            },
            onmouseup: move |_| is_dragging.set(false),
            onmouseleave: move |_| is_dragging.set(false),

            // --- THE NEW DRAG HANDLE (TITLE BAR) ---
            div {
                style: "background-color: #121014; border-bottom: 2px solid var(--mor-border-mid); padding: 8px 12px; cursor: grab; display: flex; justify-content: space-between; align-items: center; user-select: none;",
                
                // Clicking here initiates the drag
                onmousedown: move |evt| {
                    is_dragging.set(true);
                    let coords = evt.client_coordinates();
                    drag_offset.set((coords.x - *pos_x.read(), coords.y - *pos_y.read()));
                },
                
                div { 
                    style: "color: var(--mor-accent-bright); font-size: 14px; font-weight: bold;", 
                    "Block Find/Replace" 
                }
                div { 
                    style: "color: var(--mor-text-dim); font-size: 11px; font-style: italic;", 
                    "(Drag this bar to move)" 
                }
            }

            // Main Content Area
            div {
                class: "osrs-plugin-content",
                
                div { 
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    div { style: "color: var(--mor-accent-bright); font-size: 13px;", "Find Chunk:" }
                    if let Some(path) = target_file.read().as_ref() {
                        div { style: "color: var(--mor-text-muted); font-size: 12px; font-family: monospace;", "Target: {path.file_name().unwrap_or_default().to_string_lossy()}" }
                    } else {
                        div { style: "color: #7a3344; font-size: 12px; font-style: italic;", "No target file locked" }
                    }
                }
                
                textarea {
                    class: "chunk-textarea",
                    placeholder: "Paste exact multi-line block here...",
                    value: "{find_text.read()}",
                    // Notice: No stop_propagation hacks here! Copy/Paste works perfectly now.
                    oninput: move |evt| find_text.set(evt.value()),
                }

                div { style: "color: var(--mor-accent-bright); font-size: 13px;", "Replace With:" }
                
                textarea {
                    class: "chunk-textarea",
                    placeholder: "Paste new structure here...",
                    value: "{replace_text.read()}",
                    oninput: move |evt| replace_text.set(evt.value()),
                }
            }

            // Bottom OSRS-style Tab Row
            div {
                class: "osrs-plugin-tabs",
                
                button {
                    class: "osrs-plugin-tab",
                    onclick: move |_| find_replace_open.set(false),
                    "Close"
                }
                
                button {
                    class: "osrs-plugin-tab",
                    onclick: move |_| {
                        if let Some(file) = rfd::FileDialog::new().pick_file() {
                            target_file.set(Some(file.clone()));
                            logger.send(format!("[PLUGIN] Target locked: {}", file.display()));
                        }
                    },
                    "Target File"
                }
                
                button {
                    class: "osrs-plugin-tab",
                    disabled: true,
                    style: "opacity: 0.5; cursor: not-allowed;",
                    "Target Workspace"
                }
                
                button {
                    class: if is_automate { "osrs-plugin-tab active" } else { "osrs-plugin-tab" },
                    disabled: !is_automate || target_file.read().is_none() || find_text.read().is_empty(),
                    
                    onclick: move |_| {
                        if !is_automate {
                            logger.send(String::from("[BLOCKED] Block replace requires Automate Mode."));
                            return;
                        }

                        let Some(path) = target_file.read().clone() else { return };
                        let find_str = find_text.read().replace("\r\n", "\n");
                        let replace_str = replace_text.read().replace("\r\n", "\n");

                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                let normalized_content = content.replace("\r\n", "\n");
                                if !normalized_content.contains(&find_str) {
                                    logger.send(String::from("[WARNING] Find chunk not found in target file. Exact whitespace match required."));
                                    return;
                                }

                                let new_content = normalized_content.replace(&find_str, &replace_str);
                                match fs::write(&path, new_content) {
                                    Ok(_) => {
                                        logger.send(format!("[SYSTEM] Block replace executed on {}", path.file_name().unwrap_or_default().to_string_lossy()));
                                        find_text.set(String::new());
                                        replace_text.set(String::new());
                                    },
                                    Err(e) => logger.send(format!("[ERROR] Write failed: {}", e)),
                                }
                            },
                            Err(e) => logger.send(format!("[ERROR] Read failed: {}", e)),
                        }
                    },
                    if is_automate { "Execute Swap" } else { "Requires Automate" }
                }
            }
        }
    }
}