#![allow(non_snake_case)]

use dioxus::prelude::*;
use tokio::sync::mpsc;

use crate::app_state::{
    ActiveWorkspacePath,
    AppMode,
    TocSyncTrigger,
};
use crate::ui::mode_switch::ModeSwitch;
use crate::watcher::spawn_watcher;

fn spawn_log_bridge(
    logger: Coroutine<String>,
    toc_trigger: TocSyncTrigger,
) -> mpsc::UnboundedSender<String> {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let mut toc_trigger = toc_trigger.0;

    spawn(async move {
        while let Some(msg) = rx.recv().await {
            if msg == "[SYNC_TOC]" {
                toc_trigger += 1;
            } else {
                logger.send(msg);
            }
        }
    });

    tx
}

pub fn ControlPanel() -> Element {
    let mut active_path = use_context::<ActiveWorkspacePath>().0;
    let logger = use_context::<Coroutine<String>>();
    let app_mode = use_context::<Signal<AppMode>>();
    let toc_trigger = use_context::<TocSyncTrigger>();

    rsx! {
        div { class: "controls-container",
            ModeSwitch {}

            div { class: "control-group",
                h3 { "Workspace" }

                button {
                    class: "btn-outline",

                    onclick: move |_| {
                        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
                            let path_str = folder_path.display().to_string();

                            active_path.set(path_str.clone());
                            logger.send(format!("[SYSTEM] Target acquired: {}", path_str));

                            let tx = spawn_log_bridge(logger, toc_trigger);

                            if let Err(e) = spawn_watcher(&folder_path, tx, app_mode.read().clone()) {
                                logger.send(format!("[FATAL] Watcher fault: {}", e));
                            }
                        }
                    },

                    "Select Project Root"
                }
            }

            div { class: "control-group",
                h3 { "Visibility Toggles" }
                button { class: "btn-outline", "pub mod" }
                button { class: "btn-outline", "mod" }
                button { class: "btn-outline", "pub(crate) mod" }
            }

            div { class: "control-group",
                h3 { "Blueprint Templates" }
                button { class: "btn-outline", "Inject Struct" }
                button { class: "btn-outline", "Inject Enum" }
            }
        }
    }
}