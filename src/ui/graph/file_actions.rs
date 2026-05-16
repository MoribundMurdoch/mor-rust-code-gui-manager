use std::path::{Path, PathBuf};
use std::process::Command;

pub fn resolve_node_path(active_path: &str, node_path: &str) -> PathBuf {
    let node_path_buf = PathBuf::from(node_path);

    if node_path_buf.is_absolute() {
        return node_path_buf;
    }

    if active_path == "Awaiting filesystem initialization..." || active_path.trim().is_empty() {
        return node_path_buf;
    }

    Path::new(active_path).join(node_path)
}

pub fn open_file(path: &Path) -> std::io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

pub fn open_containing_folder(path: &Path) -> std::io::Result<()> {
    let folder = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };

    Command::new("xdg-open").arg(folder).spawn().map(|_| ())
}
