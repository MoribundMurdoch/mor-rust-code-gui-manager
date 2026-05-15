use std::fs;
use std::path::Path;

use super::data::NodeContextPeek;
use super::fs_ops::{label_for_path, relative_display_path, sorted_children};

fn is_probably_text_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).unwrap_or(""),
        "rs"
            | "toml"
            | "md"
            | "txt"
            | "css"
            | "html"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "json"
            | "yaml"
            | "yml"
            | "xml"
            | "ron"
    )
}

fn compact_line(line: &str) -> String {
    line.trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn detect_rust_symbols(contents: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    for line in contents.lines() {
        let line = compact_line(line);

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        let interesting = line.starts_with("pub struct ")
            || line.starts_with("struct ")
            || line.starts_with("pub enum ")
            || line.starts_with("enum ")
            || line.starts_with("pub fn ")
            || line.starts_with("fn ")
            || line.starts_with("pub mod ")
            || line.starts_with("mod ")
            || line.starts_with("#[component]")
            || line.starts_with("pub const ")
            || line.starts_with("const ");

        if interesting {
            symbols.push(line);
        }

        if symbols.len() >= 8 {
            break;
        }
    }

    symbols
}

fn declaration_preview(contents: &str, extension: &str) -> Vec<String> {
    let mut preview = Vec::new();

    for line in contents.lines() {
        let line = compact_line(line);

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        let useful = match extension {
            "rs" => {
                line.starts_with("use ")
                    || line.starts_with("pub ")
                    || line.starts_with("fn ")
                    || line.starts_with("struct ")
                    || line.starts_with("enum ")
                    || line.starts_with("impl ")
                    || line.starts_with("#[component]")
            }
            "toml" => line.starts_with('[') || line.contains('='),
            "css" => line.ends_with('{') || line.starts_with("--") || line.contains(':'),
            _ => true,
        };

        if useful {
            preview.push(line);
        }

        if preview.len() >= 6 {
            break;
        }
    }

    preview
}

pub(super) fn folder_peek(path: &Path, workspace_root: &Path, show_hidden: bool) -> NodeContextPeek {
    let children = sorted_children(path, show_hidden);
    let folders = children.iter().filter(|child| child.is_dir()).count();
    let files = children.iter().filter(|child| child.is_file()).count();

    let related_files = children
        .iter()
        .take(8)
        .map(|child| label_for_path(child))
        .collect::<Vec<_>>();

    NodeContextPeek {
        title: label_for_path(path),
        path: relative_display_path(path, workspace_root),
        metadata: vec![
            String::from("Kind: folder"),
            format!("Children: {}", children.len()),
            format!("Folders: {}", folders),
            format!("Files: {}", files),
        ],
        symbols: Vec::new(),
        related_files,
        preview: Vec::new(),
    }
}

pub(super) fn file_peek(path: &Path, workspace_root: &Path) -> NodeContextPeek {
    let metadata = fs::metadata(path).ok();
    let size_bytes = metadata.as_ref().map(|meta| meta.len()).unwrap_or(0);
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    let mut peek = NodeContextPeek {
        title: label_for_path(path),
        path: relative_display_path(path, workspace_root),
        metadata: vec![
            format!("Kind: {}", if extension.is_empty() { "file" } else { extension }),
            format!("Size: {} bytes", size_bytes),
        ],
        symbols: Vec::new(),
        related_files: Vec::new(),
        preview: Vec::new(),
    };

    if !is_probably_text_file(path) {
        peek.metadata.push(String::from("Preview: skipped binary/unknown type"));
        return peek;
    }

    if size_bytes > 128 * 1024 {
        peek.metadata.push(String::from("Preview: skipped file over 128 KB"));
        return peek;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        peek.metadata.push(String::from("Preview: unreadable as text"));
        return peek;
    };

    peek.metadata.push(format!("Lines: {}", contents.lines().count()));

    if extension == "rs" {
        peek.symbols = detect_rust_symbols(&contents);
    }

    peek.preview = declaration_preview(&contents, extension);

    peek
}
