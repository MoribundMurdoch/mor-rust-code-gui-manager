// src/app_state.rs

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppMode {
    /// Read-only mode.
    ///
    /// The app may scan, visualize, inspect, and log project structure,
    /// but it should not write files or mutate the selected Rust project.
    Inspect,

    /// Mutation-enabled mode.
    ///
    /// The app may run approved automations such as module injection,
    /// file generation, or future refactor helpers.
    Automate,
}

impl AppMode {
    pub fn is_inspect(&self) -> bool {
        matches!(self, Self::Inspect)
    }

    pub fn is_automate(&self) -> bool {
        matches!(self, Self::Automate)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Inspect => "Inspect Mode",
            Self::Automate => "Automate Mode",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Inspect => "Read-only: scan, visualize, and inspect without modifying files.",
            Self::Automate => "Automation enabled: approved actions may modify project files.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveSidebarTab {
    None,
    ControlPanel,
    QuestLog,
    Automations,
}

#[derive(Clone, Copy)]
pub struct ActiveWorkspacePath(pub Signal<String>);

#[derive(Clone, Copy)]
pub struct GraphViewMode(pub Signal<String>);

#[derive(Clone, Copy)]
pub struct GraphDepth(pub Signal<i32>);