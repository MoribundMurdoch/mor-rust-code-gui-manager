use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub path: String,
    pub x: f64,
    pub y: f64,
    pub structs: usize,
    pub enums: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeContextMenu {
    pub node_label: String,
    pub file_path: PathBuf,
    pub x: f64,
    pub y: f64,
}
