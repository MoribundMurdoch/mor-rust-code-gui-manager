use dioxus::prelude::*;
use std::collections::HashMap;

use crate::ui::file_automation_dialog::{FileAutomationDialog, FileAutomationTarget};

use super::data::{Edge, Node, NodeContextMenu};
use super::file_actions::{open_containing_folder, open_file, resolve_node_path};

pub fn HierarchyView() -> Element {
    use crate::app_state::ActiveWorkspacePath;

    let active_path = use_context::<ActiveWorkspacePath>().0;
    let logger = use_context::<Coroutine<String>>();

    let mut nodes = use_signal(|| {
        let mut map = HashMap::new();

        map.insert(
            "lib".to_string(),
            Node {
                id: "lib".to_string(),
                label: "lib.rs".to_string(),
                path: "src/lib.rs".to_string(),
                x: 400.0,
                y: 100.0,
                structs: 0,
                enums: 0,
            },
        );

        map.insert(
            "watcher".to_string(),
            Node {
                id: "watcher".to_string(),
                label: "watcher.rs".to_string(),
                path: "src/watcher.rs".to_string(),
                x: 250.0,
                y: 250.0,
                structs: 1,
                enums: 0,
            },
        );

        map.insert(
            "parser".to_string(),
            Node {
                id: "parser".to_string(),
                label: "parser.rs".to_string(),
                path: "src/parser.rs".to_string(),
                x: 550.0,
                y: 250.0,
                structs: 0,
                enums: 0,
            },
        );

        map.insert(
            "ui".to_string(),
            Node {
                id: "ui".to_string(),
                label: "ui/mod.rs".to_string(),
                path: "src/ui/mod.rs".to_string(),
                x: 400.0,
                y: 350.0,
                structs: 0,
                enums: 0,
            },
        );

        map.insert(
            "controls".to_string(),
            Node {
                id: "controls".to_string(),
                label: "controls.rs".to_string(),
                path: "src/ui/controls.rs".to_string(),
                x: 250.0,
                y: 500.0,
                structs: 0,
                enums: 0,
            },
        );

        map.insert(
            "graph".to_string(),
            Node {
                id: "graph".to_string(),
                label: "graph.rs".to_string(),
                path: "src/ui/graph.rs".to_string(),
                x: 550.0,
                y: 500.0,
                structs: 2,
                enums: 0,
            },
        );

        map
    });

    let edges = use_signal(|| {
        vec![
            Edge {
                source: "lib".to_string(),
                target: "watcher".to_string(),
            },
            Edge {
                source: "lib".to_string(),
                target: "parser".to_string(),
            },
            Edge {
                source: "lib".to_string(),
                target: "ui".to_string(),
            },
            Edge {
                source: "ui".to_string(),
                target: "controls".to_string(),
            },
            Edge {
                source: "ui".to_string(),
                target: "graph".to_string(),
            },
        ]
    });

    let mut dragging_node = use_signal(|| None::<String>);
    let mut hovered_node = use_signal(|| None::<String>);
    let mut context_menu = use_signal(|| None::<NodeContextMenu>);
    let mut automation_target = use_signal(|| None::<FileAutomationTarget>);

    rsx! {
        div {
            class: "graph-container",
            style: "position: relative; width: 100%; height: 100%; min-height: 600px;",
            prevent_default: "oncontextmenu",

            onclick: move |_| {
                context_menu.set(None);
            },

            div {
                style: "position: absolute; top: 10px; left: 10px; color: #3c2a5d; font-family: monospace; z-index: 0; pointer-events: none;",
                "{active_path.read()}"
            }

            div {
                style: "position: absolute; top: 32px; left: 10px; color: #4a3b69; font-family: monospace; font-size: 12px; z-index: 0; pointer-events: none;",
                "Left-click label: open file · Right-click node: show file actions"
            }

            svg {
                width: "100%",
                height: "100%",
                prevent_default: "oncontextmenu",
                style: "background-color: #161619; z-index: 1;",

                onmousemove: move |evt| {
                    if let Some(node_id) = dragging_node.read().clone() {
                        let mut nodes_map = nodes.write();

                        if let Some(node) = nodes_map.get_mut(&node_id) {
                            let coords = evt.coordinates();
                            node.x = coords.element().x;
                            node.y = coords.element().y;
                        }
                    }
                },

                onmouseup: move |_| {
                    dragging_node.set(None);
                },

                onmouseleave: move |_| {
                    dragging_node.set(None);
                    hovered_node.set(None);
                },

                for edge in edges.read().iter() {
                    if let (Some(source_node), Some(target_node)) =
                        (nodes.read().get(&edge.source), nodes.read().get(&edge.target))
                    {
                        line {
                            x1: "{source_node.x}",
                            y1: "{source_node.y}",
                            x2: "{target_node.x}",
                            y2: "{target_node.y}",
                            stroke: "#4a3b69",
                            stroke_width: "2",
                        }
                    }
                }

                for (id, node) in nodes.read().iter() {
                    g {
                        oncontextmenu: {
                            let node_path = node.path.clone();
                            let node_label = node.label.clone();

                            move |evt| {
                                evt.stop_propagation();

                                let workspace_path = active_path.read().clone();
                                let resolved_path = resolve_node_path(&workspace_path, &node_path);
                                let coords = evt.coordinates();

                                context_menu.set(Some(NodeContextMenu {
                                    node_label: node_label.clone(),
                                    file_path: resolved_path,
                                    x: coords.element().x,
                                    y: coords.element().y,
                                }));
                            }
                        },

                        onmouseenter: {
                            let node_id_for_hover = id.clone();

                            move |_| {
                                hovered_node.set(Some(node_id_for_hover.clone()));
                            }
                        },

                        onmouseleave: move |_| {
                            hovered_node.set(None);
                        },

                        circle {
                            cx: "{node.x}",
                            cy: "{node.y}",
                            r: "25",
                            fill: "#1e1e24",
                            stroke: "#9d72ff",
                            stroke_width: "3",
                            style: "cursor: grab;",

                            onmousedown: {
                                let node_id_for_drag = id.clone();

                                move |evt| {
                                    evt.stop_propagation();
                                    dragging_node.set(Some(node_id_for_drag.clone()));
                                }
                            },
                        }

                        text {
                            x: "{node.x}",
                            y: "{node.y + 45.0}",
                            text_anchor: "middle",
                            fill: "#e0e0e0",
                            font_family: "Georgia, serif",
                            font_size: "14px",
                            style: "cursor: pointer; user-select: none;",

                            onclick: {
                                let node_path = node.path.clone();

                                move |evt| {
                                    evt.stop_propagation();

                                    let workspace_path = active_path.read().clone();
                                    let resolved_path = resolve_node_path(&workspace_path, &node_path);

                                    match open_file(&resolved_path) {
                                        Ok(_) => {
                                            logger.send(format!(
                                                "[OPEN] File opened: {}",
                                                resolved_path.display()
                                            ));
                                        }
                                        Err(e) => {
                                            logger.send(format!(
                                                "[ERROR] Failed to open file {}: {}",
                                                resolved_path.display(),
                                                e
                                            ));
                                        }
                                    }
                                }
                            },

                            "{node.label}"
                        }

                        if hovered_node.read().as_ref() == Some(id) {
                            g {
                                rect {
                                    x: "{node.x + 30.0}",
                                    y: "{node.y - 40.0}",
                                    width: "120",
                                    height: "55",
                                    fill: "#0d0d0f",
                                    stroke: "#4a3b69",
                                    stroke_width: "1",
                                }

                                text {
                                    x: "{node.x + 40.0}",
                                    y: "{node.y - 20.0}",
                                    fill: "#a9a9b3",
                                    font_family: "monospace",
                                    font_size: "12px",
                                    "Structs: {node.structs}"
                                }

                                text {
                                    x: "{node.x + 40.0}",
                                    y: "{node.y - 5.0}",
                                    fill: "#a9a9b3",
                                    font_family: "monospace",
                                    font_size: "12px",
                                    "Enums:   {node.enums}"
                                }
                            }
                        }
                    }
                }
            }

            if let Some(menu) = context_menu.read().as_ref() {
                div {
                    class: "node-context-menu",
                    style: "position: absolute; left: {menu.x}px; top: {menu.y}px; z-index: 20;",
                    onclick: move |evt| {
                        evt.stop_propagation();
                    },

                    div {
                        class: "node-context-title",
                        "{menu.node_label}"
                    }

                    button {
                        class: "context-menu-item",

                        onclick: {
                            let file_path = menu.file_path.clone();

                            move |_| {
                                match open_file(&file_path) {
                                    Ok(_) => {
                                        logger.send(format!(
                                            "[OPEN] File opened: {}",
                                            file_path.display()
                                        ));
                                    }
                                    Err(e) => {
                                        logger.send(format!(
                                            "[ERROR] Failed to open file {}: {}",
                                            file_path.display(),
                                            e
                                        ));
                                    }
                                }

                                context_menu.set(None);
                            }
                        },

                        "Open File"
                    }

                    button {
                        class: "context-menu-item",

                        onclick: {
                            let file_path = menu.file_path.clone();

                            move |_| {
                                match open_containing_folder(&file_path) {
                                    Ok(_) => {
                                        logger.send(format!(
                                            "[OPEN] Containing folder opened for {}",
                                            file_path.display()
                                        ));
                                    }
                                    Err(e) => {
                                        logger.send(format!(
                                            "[ERROR] Failed to open containing folder for {}: {}",
                                            file_path.display(),
                                            e
                                        ));
                                    }
                                }

                                context_menu.set(None);
                            }
                        },

                        "Open Containing Folder"
                    }

                    button {
                        class: "context-menu-item",

                        onclick: {
                            let target = FileAutomationTarget {
                                label: menu.node_label.clone(),
                                path: menu.file_path.clone(),
                            };

                            move |evt| {
                                evt.stop_propagation();
                                automation_target.set(Some(target.clone()));
                                context_menu.set(None);
                            }
                        },

                        "Automations..."
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
