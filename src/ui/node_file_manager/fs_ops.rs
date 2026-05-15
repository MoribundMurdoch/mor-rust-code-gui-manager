use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) const IGNORED_NAMES: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    ".next",
    ".dioxus",
    ".idea",
    ".vscode",
    "__pycache__",
    ".DS_Store",
];

pub(super) fn is_placeholder_workspace(path: &str) -> bool {
    path.trim().is_empty() || path == "Awaiting filesystem initialization..."
}

pub(super) fn should_ignore(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| IGNORED_NAMES.contains(&name))
        .unwrap_or(false)
}

pub(super) fn label_for_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project root")
        .to_string()
}

pub(super) fn relative_display_path(path: &Path, workspace_root: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub(super) fn sorted_children(path: &Path, show_hidden: bool) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    let mut children = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            if should_ignore(path) {
                return false;
            }

            if show_hidden {
                return true;
            }

            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| !name.starts_with('.'))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    children.sort_by(|left, right| {
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

    children
}

pub(super) fn breadcrumbs_for(root: &Path, current_dir: &Path) -> Vec<PathBuf> {
    let Ok(relative) = current_dir.strip_prefix(root) else {
        return vec![current_dir.to_path_buf()];
    };

    let mut crumbs = vec![root.to_path_buf()];
    let mut cursor = root.to_path_buf();

    for component in relative.components() {
        cursor.push(component.as_os_str());
        crumbs.push(cursor.clone());
    }

    crumbs
}

pub(super) fn open_path(path: &Path) -> std::io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

pub(super) fn open_containing_folder(path: &Path) -> std::io::Result<()> {
    let folder = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };

    open_path(folder)
}

pub(super) fn open_terminal_at(path: &Path) -> std::io::Result<()> {
    let folder = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };

    Command::new("xdg-terminal-exec")
        .current_dir(folder)
        .spawn()
        .map(|_| ())
}
