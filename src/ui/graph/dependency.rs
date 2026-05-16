use dioxus::prelude::*;

use super::hierarchy::HierarchyView;

pub fn DependencyView() -> Element {
    // Phase 1 placeholder:
    // Keep behavior unchanged while graph.rs is split into view modules.
    // This will become a real dedicated view in the next phase.
    rsx! {
        HierarchyView {}
    }
}
