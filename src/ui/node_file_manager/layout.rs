use std::path::Path;

use super::data::ManagerNode;
use super::fs_ops::{label_for_path, sorted_children};

pub(super) fn build_nodes(current_dir: &Path, show_hidden: bool) -> Vec<ManagerNode> {
    let children = sorted_children(current_dir, show_hidden);
    let mut nodes = Vec::new();

    nodes.push(ManagerNode {
        label: label_for_path(current_dir),
        path: current_dir.to_path_buf(),
        is_dir: true,
        x: 460.0,
        y: 115.0,
    });

    let columns = 5usize;
    let start_x = 150.0;
    let start_y = 275.0;
    let gap_x = 165.0;
    let gap_y = 145.0;

    for (index, child) in children.into_iter().take(40).enumerate() {
        let col = index % columns;
        let row = index / columns;

        nodes.push(ManagerNode {
            label: label_for_path(&child),
            is_dir: child.is_dir(),
            path: child,
            x: start_x + (col as f64 * gap_x),
            y: start_y + (row as f64 * gap_y),
        });
    }

    nodes
}
