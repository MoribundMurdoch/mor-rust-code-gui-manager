use dioxus::prelude::*;

use crate::ui::node_file_manager::NodeFileManager;

use super::calls::CallsView;
use super::dependency::DependencyView;
use super::heatmap::HeatmapView;
use super::hierarchy::HierarchyView;
use super::types::TypesView;

pub fn ModuleTree() -> Element {
    use crate::app_state::GraphViewMode;

    let graph_view_mode = use_context::<GraphViewMode>().0;
    let mode = graph_view_mode.read().clone();

    match mode.as_str() {
        "node_file_manager" => rsx! { NodeFileManager {} },
        "Complexity" => rsx! { HeatmapView {} },
        "Dependency" => rsx! { DependencyView {} },
        "Types" => rsx! { TypesView {} },
        "Calls" => rsx! { CallsView {} },
        _ => rsx! { HierarchyView {} },
    }
}
