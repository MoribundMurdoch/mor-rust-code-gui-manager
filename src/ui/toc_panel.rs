#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app_state::{
    ActiveWorkspacePath,
    TocSyncTrigger,
};

#[derive(Clone, Debug, PartialEq)]
struct Task {
    description: String,
    is_completed: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct Chapter {
    title: String,
    tasks: Vec<Task>,
}

fn resolve_workspace_root(active_path: &str) -> PathBuf {
    let path = PathBuf::from(active_path);

    if path.is_file() {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        path
    }
}

fn strip_ordered_prefix(line: &str) -> &str {
    let Some((number_part, rest)) = line.split_once('.') else {
        return line;
    };

    if number_part.chars().all(|ch| ch.is_ascii_digit()) {
        rest.trim_start()
    } else {
        line
    }
}

fn parse_task_line(trimmed: &str) -> Option<Task> {
    let normalized = strip_ordered_prefix(trimmed);

    let candidates = [
        ("- [ ] ", false),
        ("- [x] ", true),
        ("- [X] ", true),
        ("[ ] ", false),
        ("[x] ", true),
        ("[X] ", true),
    ];

    for (prefix, is_completed) in candidates {
        if let Some(description) = normalized.strip_prefix(prefix) {
            return Some(Task {
                description: description.trim().to_string(),
                is_completed,
            });
        }
    }

    // Support plain ordered-list quest items, e.g.
    // 1. Finish Heatmap cleanup
    // These are treated as incomplete tasks.
    if normalized != trimmed && !normalized.trim().is_empty() {
        return Some(Task {
            description: normalized.trim().to_string(),
            is_completed: false,
        });
    }

    None
}

fn parse_mor_plan(path: &PathBuf) -> Vec<Chapter> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut chapters = Vec::new();
    let mut current_chapter: Option<Chapter> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|ch| *ch == '#').count();

            if level <= 3 {
                if let Some(chapter) = current_chapter.take() {
                    chapters.push(chapter);
                }

                current_chapter = Some(Chapter {
                    title: trimmed[level..].trim().to_string(),
                    tasks: Vec::new(),
                });
            }
        } else if let Some(task) = parse_task_line(trimmed) {
            if current_chapter.is_none() {
                current_chapter = Some(Chapter {
                    title: String::from("Tasks"),
                    tasks: Vec::new(),
                });
            }

            if let Some(ref mut chapter) = current_chapter {
                chapter.tasks.push(task);
            }
        }
    }

    if let Some(chapter) = current_chapter {
        chapters.push(chapter);
    }

    chapters
}

pub fn QuestLogPanel() -> Element {
    let active_path = use_context::<ActiveWorkspacePath>().0;
    let mut sync_trigger = use_context::<TocSyncTrigger>().0;

    let workspace_string = active_path.read().clone();
    let workspace_root = resolve_workspace_root(&workspace_string);
    let plan_path = workspace_root.join("MOR_PLAN.md");

    // This read is the tripwire.
    // Whenever controls.rs increments TocSyncTrigger, Dioxus re-runs this component.
    let _sync_tick = *sync_trigger.read();

    // Re-parse MOR_PLAN.md every time the component re-runs.
    let chapters = parse_mor_plan(&plan_path);

    rsx! {
        div {
            class: "quest-log-container",

            div {
                class: "quest-log-header",
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",

                h3 {
                    style: "margin: 0; color: #d1b8ff; font-family: 'Georgia', serif;",
                    "Quest Log"
                }

                button {
                    class: "btn-outline",
                    style: "padding: 4px 8px; font-size: 12px;",

                    onclick: move |_| {
                        sync_trigger += 1;
                    },

                    "Refresh"
                }
            }

            if chapters.is_empty() {
                div {
                    style: "color: #7a6b99; font-style: italic; margin-bottom: 8px;",
                    "No readable quest tasks found."
                }

                div {
                    style: "color: #4a3b69; font-family: monospace; font-size: 12px;",
                    "Looked for: {plan_path.display()}"
                }
            } else {
                div {
                    style: "color: #4a3b69; font-family: monospace; font-size: 12px; margin-bottom: 12px;",
                    "{plan_path.display()}"
                }

                for chapter in chapters.iter() {
                    div {
                        class: "quest-chapter",
                        style: "margin-bottom: 24px;",

                        div {
                            class: "chapter-title",
                            style: "color: #9d72ff; border-bottom: 1px solid #2a2040; padding-bottom: 4px; margin-bottom: 12px; font-weight: bold;",
                            "{chapter.title}"
                        }

                        div {
                            class: "chapter-tasks",
                            style: "display: flex; flex-direction: column; gap: 8px;",

                            for task in chapter.tasks.iter() {
                                {
                                    let is_done = task.is_completed;
                                    let text_color = if is_done { "#4a3b69" } else { "#a9a9b3" };
                                    let text_decoration = if is_done { "line-through" } else { "none" };
                                    let icon = if is_done { "☑" } else { "☐" };

                                    rsx! {
                                        div {
                                            class: "quest-task",
                                            style: "display: flex; gap: 8px; color: {text_color}; text-decoration: {text_decoration}; align-items: flex-start;",

                                            span {
                                                style: "font-family: monospace; font-size: 14px;",
                                                "{icon}"
                                            }

                                            span {
                                                style: "line-height: 1.4;",
                                                "{task.description}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
