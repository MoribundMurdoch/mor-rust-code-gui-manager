use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ManagerNode {
    pub label: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ManagerContextMenu {
    pub label: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NodeContextPeek {
    pub title: String,
    pub path: String,
    pub metadata: Vec<String>,
    pub symbols: Vec<String>,
    pub related_files: Vec<String>,
    pub preview: Vec<String>,
}
