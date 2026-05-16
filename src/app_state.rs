use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppMode {
    Inspect,
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
            Self::Inspect => {
                "Read-only: scan, visualize, and inspect without modifying files."
            }
            Self::Automate => {
                "Automation enabled: approved actions may modify project files."
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveSidebarTab {
    None,
    ControlPanel,
    QuestLog,
    SyncLog,
    Automations,
    PluginHub,
}

#[derive(Clone, Copy)]
pub struct ActiveWorkspacePath(pub Signal<String>);

#[derive(Clone, Copy)]
pub struct GraphViewMode(pub Signal<String>);

#[derive(Clone, Copy)]
pub struct GraphDepth(pub Signal<i32>);

#[derive(Clone, Copy)]
pub struct TocSyncTrigger(pub Signal<usize>);

#[derive(Clone, Copy)]
pub struct ActiveSidebar(pub Signal<ActiveSidebarTab>);

#[derive(Clone, Copy)]
pub struct SyncLogEntries(pub Signal<Vec<String>>);

#[derive(Clone, Copy)]
pub struct ActiveEditorFile(pub Signal<Option<std::path::PathBuf>>);

// State to track if the Find/Replace plugin widget is toggled ON.
#[derive(Clone, Copy)]
pub struct PluginFindReplaceOpen(pub Signal<bool>);

// State to track if the standalone Wiki Search plugin widget is toggled ON.
#[derive(Clone, Copy)]
pub struct PluginWikiSearchOpen(pub Signal<bool>);

// State to track if the standalone Node Editor plugin widget is toggled ON.
#[derive(Clone, Copy)]
pub struct PluginNodeEditorOpen(pub Signal<bool>);

// Increment this signal to request that the active embedded editor save its current buffer.
#[derive(Clone, Copy)]
pub struct EditorSaveRequest(pub Signal<usize>);
