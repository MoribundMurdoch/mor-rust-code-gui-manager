use dioxus::prelude::*;
use std::path::PathBuf;

use crate::app_state::ActiveWorkspacePath;
use crate::ui::file_automation_dialog::{FileAutomationDialog, FileAutomationTarget};
use crate::wiki_lookup::{common_file_wiki_lookup, log_and_open_wiki};

use super::data::{ManagerContextMenu, ManagerNode};
use super::fs_ops::{
    breadcrumbs_for, is_placeholder_workspace, label_for_path, open_containing_folder, open_path,
    open_terminal_at,
};
use super::layout::build_nodes;
use super::peek::{file_peek, folder_peek};

fn navigate_to(
    next_dir: PathBuf,
    current_dir: &mut Signal<PathBuf>,
    back_stack: &mut Signal<Vec<PathBuf>>,
    forward_stack: &mut Signal<Vec<PathBuf>>,
) {
    if next_dir.is_dir() && next_dir.exists() {
        let previous = current_dir.read().clone();

        if previous != next_dir && previous.exists() {
            back_stack.write().push(previous);
            forward_stack.write().clear();
        }

        current_dir.set(next_dir);
    }
}

#[component]
pub fn NodeFileManager() -> Element {
    let active_path = use_context::<ActiveWorkspacePath>().0;
    let logger = use_context::<Coroutine<String>>();

    let workspace_string = active_path.read().clone();
    let workspace_root = PathBuf::from(workspace_string.as_str());
    let has_workspace = !is_placeholder_workspace(&workspace_string) && workspace_root.exists();

    let mut current_dir = use_signal(PathBuf::new);
    let mut back_stack = use_signal(Vec::<PathBuf>::new);
    let mut forward_stack = use_signal(Vec::<PathBuf>::new);
    let mut show_hidden = use_signal(|| false);
    let mut context_menu = use_signal(|| None::<ManagerContextMenu>);
    let mut hovered_node = use_signal(|| None::<ManagerNode>);
    let mut automation_target = use_signal(|| None::<FileAutomationTarget>);

    // --- CAMERA PANNING & ZOOM STATE ---
    let mut pan_offset = use_signal(|| (0.0_f64, 0.0_f64));
    let mut is_panning = use_signal(|| false);
    let mut last_mouse_pos = use_signal(|| (0.0_f64, 0.0_f64));
    let mut zoom_level = use_signal(|| 1.0_f64);

    if has_workspace && (!current_dir.read().exists() || current_dir.read().as_os_str().is_empty()) {
        current_dir.set(workspace_root.clone());
    }

    let current_dir_value = if current_dir.read().exists() {
        current_dir.read().clone()
    } else {
        workspace_root.clone()
    };

    let nodes = if has_workspace {
        build_nodes(&current_dir_value, *show_hidden.read())
    } else {
        Vec::new()
    };

    let can_go_up = has_workspace
        && current_dir_value != workspace_root
        && current_dir_value.parent().is_some();

    let crumbs = if has_workspace {
        breadcrumbs_for(&workspace_root, &current_dir_value)
    } else {
        Vec::new()
    };

    rsx! {
        div {
            class: "node-file-manager-graph",
            prevent_default: "oncontextmenu",
            onclick: move |_| {
                context_menu.set(None);
            },

            div {
                class: "node-file-manager-topbar",
                onclick: move |evt| evt.stop_propagation(),

                div {
                    class: "node-file-manager-title",
                    "Node File Manager"
                }

                button {
                    class: "nfm-nav-button",
                    disabled: back_stack.read().is_empty(),
                    onclick: move |_| {
                        let previous = back_stack.write().pop();

                        if let Some(previous) = previous {
                            forward_stack.write().push(current_dir.read().clone());
                            current_dir.set(previous);
                        }
                    },
                    "←"
                }

                button {
                    class: "nfm-nav-button",
                    disabled: forward_stack.read().is_empty(),
                    onclick: move |_| {
                        let next = forward_stack.write().pop();

                        if let Some(next) = next {
                            back_stack.write().push(current_dir.read().clone());
                            current_dir.set(next);
                        }
                    },
                    "→"
                }

                button {
                    class: "nfm-nav-button",
                    disabled: !can_go_up,
                    onclick: move |_| {
                        let parent = {
                            let current = current_dir.read();
                            current.parent().map(|parent| parent.to_path_buf())
                        };

                        if let Some(parent) = parent {
                            let previous = current_dir.read().clone();

                            if parent.exists() {
                                back_stack.write().push(previous);
                                forward_stack.write().clear();
                                current_dir.set(parent);
                            }
                        }
                    },
                    "↑"
                }

                div {
                    class: "nfm-breadcrumbs",

                    for crumb in crumbs.iter() {
                        {
                            let crumb_path = crumb.clone();
                            let crumb_label = if crumb_path == workspace_root {
                                label_for_path(&workspace_root)
                            } else {
                                label_for_path(&crumb_path)
                            };

                            rsx! {
                                button {
                                    class: if crumb_path == current_dir_value {
                                        "nfm-crumb active"
                                    } else {
                                        "nfm-crumb"
                                    },
                                    onclick: move |_| {
                                        navigate_to(
                                            crumb_path.clone(),
                                            &mut current_dir,
                                            &mut back_stack,
                                            &mut forward_stack,
                                        );
                                    },
                                    "{crumb_label}"
                                }
                            }
                        }
                    }
                }

                button {
                    class: "nfm-nav-button",
                    onclick: move |_| {
                        let next = {
                            let current = *show_hidden.read();
                            !current
                        };

                        show_hidden.set(next);
                    },
                    if *show_hidden.read() {
                        "Hide dotfiles"
                    } else {
                        "Show dotfiles"
                    }
                }

                button {
                    class: "nfm-nav-button",
                    disabled: !has_workspace,
                    onclick: move |_| {
                        current_dir.set(workspace_root.clone());
                        back_stack.write().clear();
                        forward_stack.write().clear();
                        pan_offset.set((0.0, 0.0));
                        zoom_level.set(1.0);
                    },
                    "Root"
                }
            }

            if !has_workspace {
                div {
                    class: "nfm-empty",
                    "Select or drop a project root before using Node File Manager."
                }
            } else {
                div {
                    class: "nfm-pathbar",
                    "{current_dir_value.display()}"
                }

                svg {
                    class: "nfm-svg",
                    width: "100%",
                    height: "760",
                    prevent_default: "oncontextmenu",

                    // --- CAMERA PAN + POINTER TRACKING EVENTS ---
                    onmousedown: move |evt| {
                        // The nodes themselves stop propagation, so an onmousedown here is guaranteed
                        // to be on the empty canvas void.
                        is_panning.set(true);

                        let coords = evt.coordinates().client();
                        last_mouse_pos.set((coords.x, coords.y));
                    },

                    onmousemove: move |evt| {
                        let coords = evt.coordinates().client();
                        let current_x = coords.x;
                        let current_y = coords.y;

                        if *is_panning.read() {
                            let (last_x, last_y) = *last_mouse_pos.read();

                            let delta_x = current_x - last_x;
                            let delta_y = current_y - last_y;

                            let (offset_x, offset_y) = *pan_offset.read();
                            pan_offset.set((offset_x + delta_x, offset_y + delta_y));
                        }

                        // Always remember the latest mouse position.
                        // WheelData does not expose coordinates in this Dioxus version,
                        // so onwheel uses this saved pointer position.
                        last_mouse_pos.set((current_x, current_y));
                    },

                    onmouseup: move |_| {
                        is_panning.set(false);
                    },

                    onmouseleave: move |_| {
                        is_panning.set(false);
                    },

                    onwheel: move |evt| {
                        // WheelData gives scroll delta, but not mouse coordinates.
                        // Use the latest coordinates saved by onmousemove.
                        let (mouse_x, mouse_y) = *last_mouse_pos.read();

                        let delta_y = evt.data().delta().strip_units().y;

                        let zoom_sensitivity = 0.1;
                        let min_zoom = 0.2;
                        let max_zoom = 3.0;

                        let current_zoom = *zoom_level.read();
                        let mut new_zoom = current_zoom;

                        if delta_y < 0.0 {
                            new_zoom += zoom_sensitivity;
                        } else if delta_y > 0.0 {
                            new_zoom -= zoom_sensitivity;
                        }

                        new_zoom = new_zoom.clamp(min_zoom, max_zoom);

                        if new_zoom != current_zoom {
                            let (pan_x, pan_y) = *pan_offset.read();

                            // Convert physical mouse position into logical SVG/canvas space.
                            let logical_x = (mouse_x - pan_x) / current_zoom;
                            let logical_y = (mouse_y - pan_y) / current_zoom;

                            // Recalculate pan so the same logical point stays under the cursor.
                            let new_pan_x = mouse_x - (logical_x * new_zoom);
                            let new_pan_y = mouse_y - (logical_y * new_zoom);

                            zoom_level.set(new_zoom);
                            pan_offset.set((new_pan_x, new_pan_y));
                        }
                    },

                    // --- INFINITE CANVAS GROUP ---
                    g {
                        transform: "translate({pan_offset.read().0}, {pan_offset.read().1}) scale({zoom_level.read()})",

                        for node in nodes.iter() {
                            if node.path != current_dir_value {
                                line {
                                    x1: "460",
                                    y1: "115",
                                    x2: "{node.x}",
                                    y2: "{node.y}",
                                    stroke: "#4a3b69",
                                    stroke_width: "1.5",
                                    opacity: "0.75",
                                }
                            }
                        }

                        for node in nodes.iter() {
                            {
                                let node_path = node.path.clone();
                                let node_label = node.label.clone();
                                let is_dir = node.is_dir;
                                let is_current = node.path == current_dir_value;
                                let radius = if is_current { 36 } else { 28 };

                                rsx! {
                                    g {
                                        class: "nfm-node",
                                        transform: "translate({node.x}, {node.y})",

                                        oncontextmenu: {
                                            let node_path = node_path.clone();
                                            let node_label = node_label.clone();

                                            move |evt| {
                                                evt.stop_propagation();
                                                let coords = evt.coordinates();

                                                context_menu.set(Some(ManagerContextMenu {
                                                    label: node_label.clone(),
                                                    path: node_path.clone(),
                                                    is_dir,
                                                    x: coords.element().x,
                                                    y: coords.element().y,
                                                }));
                                            }
                                        },

                                        onclick: {
                                            let node_path = node_path.clone();

                                            move |evt| {
                                                evt.stop_propagation();

                                                if is_dir && !is_current {
                                                    navigate_to(
                                                        node_path.clone(),
                                                        &mut current_dir,
                                                        &mut back_stack,
                                                        &mut forward_stack,
                                                    );
                                                }
                                            }
                                        },

                                        onmouseenter: {
                                            let node_for_hover = node.clone();

                                            move |_| {
                                                hovered_node.set(Some(node_for_hover.clone()));
                                            }
                                        },

                                        onmouseleave: move |_| {
                                            hovered_node.set(None);
                                        },

                                        circle {
                                            r: "{radius}",
                                            class: if is_current {
                                                "nfm-circle current"
                                            } else if is_dir {
                                                "nfm-circle folder"
                                            } else {
                                                "nfm-circle file"
                                            },
                                        }

                                        text {
                                            class: "nfm-node-icon",
                                            y: "5",
                                            text_anchor: "middle",
                                            if is_current {
                                                "⌂"
                                            } else if is_dir {
                                                "▣"
                                            } else {
                                                "◦"
                                            }
                                        }

                                        text {
                                            class: "nfm-node-label",
                                            y: "{radius + 24}",
                                            text_anchor: "middle",
                                            "{node.label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(peek_node) = hovered_node.read().clone() {
                {
                    // Apply camera pan + zoom so the HTML div tracks the SVG layout.
                    let offset_x = pan_offset.read().0;
                    let offset_y = pan_offset.read().1;
                    let zoom = *zoom_level.read();

                    let peek_left = peek_node.x * zoom + offset_x + 38.0;
                    let peek_top = peek_node.y * zoom + offset_y - 18.0;

                    let peek = if peek_node.is_dir {
                        folder_peek(&peek_node.path, &workspace_root, *show_hidden.read())
                    } else {
                        file_peek(&peek_node.path, &workspace_root)
                    };

                    rsx! {
                        div {
                            class: "nfm-context-peek",
                            style: "left: {peek_left}px; top: {peek_top}px;",

                            div {
                                class: "nfm-peek-title",
                                "{peek.title}"
                            }

                            div {
                                class: "nfm-peek-path",
                                "{peek.path}"
                            }

                            for item in peek.metadata.iter() {
                                div {
                                    class: "nfm-peek-meta",
                                    "{item}"
                                }
                            }

                            if !peek.symbols.is_empty() {
                                div {
                                    class: "nfm-peek-section",
                                    "LLM anchors"
                                }

                                for symbol in peek.symbols.iter() {
                                    div {
                                        class: "nfm-peek-line",
                                        "{symbol}"
                                    }
                                }
                            }

                            if !peek.related_files.is_empty() {
                                div {
                                    class: "nfm-peek-section",
                                    "Nearby nodes"
                                }

                                for related in peek.related_files.iter() {
                                    div {
                                        class: "nfm-peek-line",
                                        "{related}"
                                    }
                                }
                            }

                            if !peek.preview.is_empty() {
                                div {
                                    class: "nfm-peek-section",
                                    "Useful preview"
                                }

                                for line in peek.preview.iter() {
                                    div {
                                        class: "nfm-peek-code",
                                        "{line}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(menu) = context_menu.read().clone() {
                div {
                    class: "nfm-context-menu",
                    style: "left: {menu.x}px; top: {menu.y}px;",
                    onclick: move |evt| evt.stop_propagation(),

                    div {
                        class: "nfm-context-title",
                        "{menu.label}"
                    }

                    if menu.is_dir {
                        button {
                            class: "context-menu-item",
                            onclick: {
                                let path = menu.path.clone();

                                move |_| {
                                    navigate_to(
                                        path.clone(),
                                        &mut current_dir,
                                        &mut back_stack,
                                        &mut forward_stack,
                                    );
                                    context_menu.set(None);
                                    pan_offset.set((0.0, 0.0));
                                    zoom_level.set(1.0);
                                }
                            },
                            "Open Folder Node"
                        }

                        button {
                            class: "context-menu-item",
                            onclick: {
                                let path = menu.path.clone();

                                move |_| {
                                    match open_path(&path) {
                                        Ok(_) => logger.send(format!("[OPEN] Folder opened: {}", path.display())),
                                        Err(e) => logger.send(format!("[ERROR] Failed to open folder {}: {}", path.display(), e)),
                                    }

                                    context_menu.set(None);
                                }
                            },
                            "Open in System File Manager"
                        }

                        button {
                            class: "context-menu-item",
                            onclick: {
                                let path = menu.path.clone();

                                move |_| {
                                    match open_terminal_at(&path) {
                                        Ok(_) => logger.send(format!("[OPEN] Terminal opened at: {}", path.display())),
                                        Err(e) => logger.send(format!("[ERROR] Failed to open terminal at {}: {}", path.display(), e)),
                                    }

                                    context_menu.set(None);
                                }
                            },
                            "Open Terminal Here"
                        }
                    } else {
                        button {
                            class: "context-menu-item",
                            onclick: {
                                let path = menu.path.clone();

                                move |_| {
                                    match open_path(&path) {
                                        Ok(_) => logger.send(format!("[OPEN] File opened: {}", path.display())),
                                        Err(e) => logger.send(format!("[ERROR] Failed to open file {}: {}", path.display(), e)),
                                    }

                                    context_menu.set(None);
                                }
                            },
                            "Open File"
                        }

                        button {
                            class: "context-menu-item",
                            onclick: {
                                let path = menu.path.clone();

                                move |_| {
                                    match open_containing_folder(&path) {
                                        Ok(_) => logger.send(format!("[OPEN] Containing folder opened for {}", path.display())),
                                        Err(e) => logger.send(format!("[ERROR] Failed to open containing folder for {}: {}", path.display(), e)),
                                    }

                                    context_menu.set(None);
                                }
                            },
                            "Open Containing Folder"
                        }

                        button {
                            class: "context-menu-item",
                            onclick: {
                                let path = menu.path.clone();
                                let label = menu.label.clone();

                                move |_| {
                                    automation_target.set(Some(FileAutomationTarget {
                                        label: label.clone(),
                                        path: path.clone(),
                                    }));

                                    context_menu.set(None);
                                }
                            },
                            "Automations..."
                        }

                        if let Some(lookup) = common_file_wiki_lookup(&menu.path) {
                            button {
                                class: "context-menu-item",
                                onclick: {
                                    let lookup = lookup.clone();

                                    move |_| {
                                        log_and_open_wiki(&logger, lookup.clone());
                                        context_menu.set(None);
                                    }
                                },
                                "{lookup.label}"
                            }
                        }
                    }

                    button {
                        class: "context-menu-item",
                        onclick: move |_| {
                            context_menu.set(None);
                        },
                        "Cancel"
                    }
                }
            }

            if let Some(target) = automation_target.read().clone() {
                FileAutomationDialog {
                    target,
                    onclose: move |_| automation_target.set(None),
                }
            }
        }
    }
}