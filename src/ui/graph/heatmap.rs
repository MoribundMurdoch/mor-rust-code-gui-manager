#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app_state::ActiveWorkspacePath;
use crate::ui::file_automation_dialog::{FileAutomationDialog, FileAutomationTarget};

use super::file_actions::{open_containing_folder, open_file};

#[derive(Clone, Debug, PartialEq)]
struct RustFileMetrics {
    label: String,
    path: PathBuf,
    relative_path: String,
    lines: usize,
    structs: usize,
    enums: usize,
    functions: usize,
    impl_blocks: usize,
    uses: usize,
    score: usize,
    x: f64,
    y: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct HeatmapContextMenu {
    label: String,
    path: PathBuf,
    x: f64,
    y: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct HeatmapTooltip {
    node: RustFileMetrics,
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PanDragState {
    start_mouse_x: f64,
    start_mouse_y: f64,
    start_pan_x: f64,
    start_pan_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ViewTransform {
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
}

impl ViewTransform {
    fn reset() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

fn is_placeholder_workspace(path: &str) -> bool {
    path.trim().is_empty() || path == "Awaiting filesystem initialization..."
}

fn should_ignore_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            matches!(
                name,
                "target"
                    | ".git"
                    | "node_modules"
                    | "dist"
                    | "build"
                    | ".next"
                    | ".dioxus"
                    | ".idea"
                    | ".vscode"
                    | "__pycache__"
            )
        })
        .unwrap_or(false)
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if should_ignore_dir(dir) {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut paths = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<Vec<_>>();

    paths.sort_by(|left, right| {
        let left_is_dir = left.is_dir();
        let right_is_dir = right.is_dir();

        right_is_dir.cmp(&left_is_dir).then_with(|| {
            left.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .cmp(
                    &right
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase(),
                )
        })
    });

    for path in paths {
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn scan_scope_root(root: &Path) -> PathBuf {
    let src_root = root.join("src");

    if src_root.exists() {
        src_root
    } else {
        root.to_path_buf()
    }
}

fn relative_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn label_for_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown.rs")
        .to_string()
}

fn count_metric(contents: &str, matcher: impl Fn(&str) -> bool) -> usize {
    contents
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.starts_with("//"))
        .filter(|line| matcher(line))
        .count()
}

fn analyze_rust_file(path: &Path, root: &Path, index: usize) -> RustFileMetrics {
    let contents = fs::read_to_string(path).unwrap_or_default();

    let lines = contents.lines().count();

    let structs = count_metric(&contents, |line| {
        line.starts_with("struct ") || line.starts_with("pub struct ")
    });

    let enums = count_metric(&contents, |line| {
        line.starts_with("enum ") || line.starts_with("pub enum ")
    });

    let functions = count_metric(&contents, |line| {
        line.starts_with("fn ")
            || line.starts_with("pub fn ")
            || line.starts_with("pub(crate) fn ")
            || line.starts_with("async fn ")
            || line.starts_with("pub async fn ")
            || line.starts_with("pub(crate) async fn ")
    });

    let impl_blocks = count_metric(&contents, |line| line.starts_with("impl "));

    let uses = count_metric(&contents, |line| {
        line.starts_with("use ") || line.starts_with("pub use ")
    });

    let score = (lines / 20)
        + (structs * 8)
        + (enums * 8)
        + (functions * 4)
        + (impl_blocks * 5)
        + uses;

    let columns = 5usize;
    let start_x = 145.0;
    let start_y = 155.0;
    let gap_x = 165.0;
    let gap_y = 140.0;
    let col = index % columns;
    let row = index / columns;

    RustFileMetrics {
        label: label_for_path(path),
        path: path.to_path_buf(),
        relative_path: relative_path(path, root),
        lines,
        structs,
        enums,
        functions,
        impl_blocks,
        uses,
        score,
        x: start_x + (col as f64 * gap_x),
        y: start_y + (row as f64 * gap_y),
    }
}

fn heatmap_nodes(root: &Path) -> Vec<RustFileMetrics> {
    let scan_root = scan_scope_root(root);

    let mut rust_files = Vec::new();
    collect_rust_files(&scan_root, &mut rust_files);

    rust_files
        .iter()
        .enumerate()
        .map(|(index, path)| analyze_rust_file(path, root, index))
        .collect()
}

fn radius_for_score(score: usize) -> f64 {
    let capped = score.min(120) as f64;
    20.0 + (capped / 120.0 * 28.0)
}

fn stroke_for_score(score: usize) -> &'static str {
    if score >= 70 {
        "#bc8d6b"
    } else if score >= 35 {
        "#9d72ff"
    } else {
        "#6f4bb3"
    }
}

fn stroke_width_for_score(score: usize) -> &'static str {
    if score >= 70 {
        "6"
    } else if score >= 35 {
        "4"
    } else {
        "2.5"
    }
}

fn pressure_label(score: usize) -> &'static str {
    if score >= 70 {
        "High pressure"
    } else if score >= 35 {
        "Medium pressure"
    } else {
        "Low pressure"
    }
}

fn clamp_tooltip_x(x: f64) -> f64 {
    x.clamp(12.0, 660.0)
}

fn clamp_tooltip_y(y: f64) -> f64 {
    y.clamp(76.0, 520.0)
}

pub fn HeatmapView() -> Element {
    let active_path = use_context::<ActiveWorkspacePath>().0;
    let logger = use_context::<Coroutine<String>>();

    let workspace_path = active_path.read().clone();
    let workspace_root = PathBuf::from(workspace_path.as_str());
    let has_workspace = !is_placeholder_workspace(&workspace_path) && workspace_root.exists();

    let scan_root = if has_workspace {
        scan_scope_root(&workspace_root)
    } else {
        PathBuf::new()
    };

    let nodes = if has_workspace {
        heatmap_nodes(&workspace_root)
    } else {
        Vec::new()
    };

    let max_score = nodes.iter().map(|node| node.score).max().unwrap_or(0);
    let total_score = nodes.iter().map(|node| node.score).sum::<usize>();

    let mut view = use_signal(ViewTransform::reset);
    let mut pan_drag = use_signal(|| None::<PanDragState>);
    let mut tooltip = use_signal(|| None::<HeatmapTooltip>);
    let mut context_menu = use_signal(|| None::<HeatmapContextMenu>);
    let mut automation_target = use_signal(|| None::<FileAutomationTarget>);

    let view_now = *view.read();
    let graph_transform = format!(
        "translate({} {}) scale({})",
        view_now.pan_x,
        view_now.pan_y,
        view_now.zoom
    );

    let zoom_label = format!("{:.0}%", view_now.zoom * 100.0);
    let heatmap_status = if has_workspace {
        format!(
            "Scope: {} · Files: {} · Total score: {} · Max score: {} · Zoom: {}",
            scan_root.display(),
            nodes.len(),
            total_score,
            max_score,
            zoom_label
        )
    } else {
        String::from("Select or drop a project root to scan src/**/*.rs")
    };

    let scope_note = String::from(
        "Scope selector: Project src/ for now · Current Folder / Whole Workspace coming later · Middle-drag to pan · Wheel to zoom"
    );

    rsx! {
        div {
            class: "graph-container heatmap-view",
            style: "position: relative; width: 100%; height: 100%; min-height: 760px; overflow: hidden;",
            prevent_default: "oncontextmenu onwheel",

            onclick: move |_| {
                context_menu.set(None);
            },

            onmouseup: move |_| {
                pan_drag.set(None);
            },

            onmouseleave: move |_| {
                pan_drag.set(None);
                tooltip.set(None);
            },

            onmousemove: move |evt| {
                let coords = evt.coordinates().element();

                let drag_state = {
                    *pan_drag.read()
                };

                if let Some(drag) = drag_state {
                    let dx = coords.x - drag.start_mouse_x;
                    let dy = coords.y - drag.start_mouse_y;
                    let current_zoom = {
                        view.read().zoom
                    };

                    view.set(ViewTransform {
                        pan_x: drag.start_pan_x + dx,
                        pan_y: drag.start_pan_y + dy,
                        zoom: current_zoom,
                    });
                }

                let current_tooltip = {
                    tooltip.read().clone()
                };

                if let Some(current_tooltip) = current_tooltip {
                    tooltip.set(Some(HeatmapTooltip {
                        node: current_tooltip.node,
                        x: clamp_tooltip_x(coords.x + 18.0),
                        y: clamp_tooltip_y(coords.y + 18.0),
                    }));
                }
            },

            onwheel: move |evt| {
                let delta_y = evt.delta().strip_units().y;
                let current = *view.read();

                let zoom_factor = if delta_y < 0.0 { 1.08 } else { 0.92 };
                let next_zoom = (current.zoom * zoom_factor).clamp(0.45, 2.50);

                view.set(ViewTransform {
                    pan_x: current.pan_x,
                    pan_y: current.pan_y,
                    zoom: next_zoom,
                });
            },

            div {
                style: "position: absolute; top: 10px; left: 10px; color: #6f4bb3; font-family: monospace; z-index: 2; pointer-events: none;",
                "Heatmap View · Project Rust file pressure map"
            }

            div {
                style: "position: absolute; top: 32px; left: 10px; color: #4a3b69; font-family: monospace; font-size: 12px; z-index: 2; pointer-events: none;",
                "{heatmap_status}"
            }

            div {
                style: "position: absolute; top: 58px; left: 10px; color: #4a3b69; font-family: monospace; font-size: 12px; z-index: 2; pointer-events: none;",
                "{scope_note}"
            }

            div {
                style: "position: absolute; top: 10px; right: 12px; display: flex; gap: 8px; z-index: 5;",

                button {
                    class: "toolbar-action-btn",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        view.set(ViewTransform::reset());
                        logger.send(String::from("[VIEW] Heatmap view reset"));
                    },
                    "Reset View"
                }

                button {
                    class: "toolbar-action-btn",
                    disabled: true,
                    title: "Coming soon: fit all heatmap nodes into the visible canvas.",
                    "Fit View Soon"
                }
            }

            svg {
                width: "100%",
                height: "760",
                prevent_default: "oncontextmenu onwheel",
                style: "background-color: #161619; z-index: 1; cursor: grab;",

                onmousedown: move |evt| {
                    evt.stop_propagation();

                    let is_middle_mouse = evt.data.trigger_button().map(|button| format!("{:?}", button) == "Auxiliary" || format!("{:?}", button) == "Middle" || format!("{:?}", button) == "1").unwrap_or(false);

                    if is_middle_mouse {
                        let coords = evt.coordinates().element();
                        let current = *view.read();

                        pan_drag.set(Some(PanDragState {
                            start_mouse_x: coords.x,
                            start_mouse_y: coords.y,
                            start_pan_x: current.pan_x,
                            start_pan_y: current.pan_y,
                        }));
                    }
                },

                if has_workspace && nodes.is_empty() {
                    text {
                        x: "40",
                        y: "100",
                        fill: "#bc8d6b",
                        font_family: "Georgia, serif",
                        font_size: "18px",
                        "No Rust files found."
                    }
                }

                g {
                    transform: "{graph_transform}",

                    for node in nodes.iter() {
                        {
                            let node_for_hover = node.clone();
                            let node_for_menu = node.clone();
                            let node_for_open = node.clone();
                            let radius = radius_for_score(node.score);
                            let stroke = stroke_for_score(node.score);
                            let stroke_width = stroke_width_for_score(node.score);

                            rsx! {
                                g {
                                    transform: "translate({node.x}, {node.y})",

                                    onmouseenter: move |evt| {
                                        let coords = evt.coordinates().element();

                                        tooltip.set(Some(HeatmapTooltip {
                                            node: node_for_hover.clone(),
                                            x: clamp_tooltip_x(coords.x + 18.0),
                                            y: clamp_tooltip_y(coords.y + 18.0),
                                        }));
                                    },

                                    onmouseleave: move |_| {
                                        tooltip.set(None);
                                    },

                                    oncontextmenu: move |evt| {
                                        evt.stop_propagation();
                                        let coords = evt.coordinates().element();

                                        context_menu.set(Some(HeatmapContextMenu {
                                            label: node_for_menu.label.clone(),
                                            path: node_for_menu.path.clone(),
                                            x: coords.x,
                                            y: coords.y,
                                        }));
                                    },

                                    onclick: move |evt| {
                                        evt.stop_propagation();

                                        match open_file(&node_for_open.path) {
                                            Ok(_) => logger.send(format!(
                                                "[OPEN] File opened: {}",
                                                node_for_open.path.display()
                                            )),
                                            Err(e) => logger.send(format!(
                                                "[ERROR] Failed to open file {}: {}",
                                                node_for_open.path.display(),
                                                e
                                            )),
                                        }
                                    },

                                    circle {
                                        r: "{radius}",
                                        fill: "#1e1e24",
                                        stroke: "{stroke}",
                                        stroke_width: "{stroke_width}",
                                        style: "cursor: pointer;",
                                    }

                                    text {
                                        y: "5",
                                        text_anchor: "middle",
                                        fill: "#f0e8ff",
                                        font_family: "monospace",
                                        font_size: "13px",
                                        style: "pointer-events: none;",
                                        "{node.score}"
                                    }

                                    text {
                                        y: "{radius + 24.0}",
                                        text_anchor: "middle",
                                        fill: "#f0e8ff",
                                        font_family: "Georgia, serif",
                                        font_size: "13px",
                                        style: "pointer-events: none;",
                                        "{node.label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(tip) = tooltip.read().clone() {
                div {
                    class: "node-context-menu",
                    style: "position: absolute; left: {tip.x}px; top: {tip.y}px; z-index: 40; min-width: 275px; max-width: 360px; pointer-events: none;",

                    div {
                        class: "node-context-title",
                        "{tip.node.label}"
                    }

                    div { class: "context-menu-item", "Pressure: {pressure_label(tip.node.score)}" }
                    div { class: "context-menu-item", "Score: {tip.node.score}" }
                    div { class: "context-menu-item", "Lines: {tip.node.lines}" }
                    div { class: "context-menu-item", "Structs: {tip.node.structs}" }
                    div { class: "context-menu-item", "Enums: {tip.node.enums}" }
                    div { class: "context-menu-item", "Functions: {tip.node.functions}" }
                    div { class: "context-menu-item", "Impl blocks: {tip.node.impl_blocks}" }
                    div { class: "context-menu-item", "Use statements: {tip.node.uses}" }
                    div { class: "context-menu-item", "{tip.node.relative_path}" }
                }
            }

            if let Some(menu) = context_menu.read().clone() {
                div {
                    class: "node-context-menu",
                    style: "position: absolute; left: {menu.x}px; top: {menu.y}px; z-index: 50;",
                    onclick: move |evt| {
                        evt.stop_propagation();
                    },

                    div {
                        class: "node-context-title",
                        "{menu.label}"
                    }

                    button {
                        class: "context-menu-item",
                        onclick: {
                            let file_path = menu.path.clone();

                            move |_| {
                                match open_file(&file_path) {
                                    Ok(_) => logger.send(format!(
                                        "[OPEN] File opened: {}",
                                        file_path.display()
                                    )),
                                    Err(e) => logger.send(format!(
                                        "[ERROR] Failed to open file {}: {}",
                                        file_path.display(),
                                        e
                                    )),
                                }

                                context_menu.set(None);
                            }
                        },
                        "Open File"
                    }

                    button {
                        class: "context-menu-item",
                        onclick: {
                            let file_path = menu.path.clone();

                            move |_| {
                                match open_containing_folder(&file_path) {
                                    Ok(_) => logger.send(format!(
                                        "[OPEN] Containing folder opened for {}",
                                        file_path.display()
                                    )),
                                    Err(e) => logger.send(format!(
                                        "[ERROR] Failed to open containing folder for {}: {}",
                                        file_path.display(),
                                        e
                                    )),
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
                                label: menu.label.clone(),
                                path: menu.path.clone(),
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
