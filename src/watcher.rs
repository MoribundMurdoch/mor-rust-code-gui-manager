use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc::UnboundedSender;

use crate::app_state::AppMode;
use crate::parser::inject_pub_mod;

pub fn spawn_watcher(
    watch_path: impl AsRef<Path>,
    tx: UnboundedSender<String>,
    app_mode: AppMode,
) -> Result<()> {
    let path = watch_path.as_ref().to_path_buf();

    std::thread::spawn(move || {
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();

        let mut watcher = match RecommendedWatcher::new(notify_tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                let _ = tx.send(format!("[FATAL] Failed to initialize watcher: {}", e));
                return;
            }
        };

        if let Err(e) = watcher.watch(&path, RecursiveMode::Recursive) {
            let _ = tx.send(format!("[FATAL] Failed to watch path {:?}: {}", path, e));
            return;
        }

        let _ = tx.send(format!("[SYSTEM] Watcher locked onto {:?}", path));

        match app_mode {
            AppMode::Inspect => {
                let _ = tx.send(String::from(
                    "[SAFETY] Inspect Mode active: watcher will log file changes only.",
                ));
            }
            AppMode::Automate => {
                let _ = tx.send(String::from(
                    "[SAFETY] Automate Mode active: approved watcher automations may modify files.",
                ));
            }
        }

        for res in notify_rx {
            match res {
                Ok(Event { kind, paths, .. }) => {
                    // --- PHASE 5 & CORE: File Creation Logic ---
                    if kind.is_create() {
                        for p in &paths {
                            if p.extension().map_or(false, |ext| ext == "rs") {
                                let _ = tx.send(format!("[DETECTED] New Rust file: {}", p.display()));

                                match app_mode {
                                    AppMode::Inspect => {
                                        let _ = tx.send(format!(
                                            "[INSPECT] Would consider module registration for {}, but writes are blocked.",
                                            p.display()
                                        ));
                                    }

                                    AppMode::Automate => {
                                        match inject_pub_mod(&p) {
                                            Ok(log_msg) => {
                                                let _ = tx.send(log_msg);
                                            }
                                            Err(e) => {
                                                let _ = tx.send(format!(
                                                    "[ERROR] Parser fault on {}: {}",
                                                    p.display(),
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } 
                    // --- PHASE 3: Quest Log Sync Logic ---
                    else if kind.is_modify() {
                        for p in paths {
                            if p.file_name().map_or(false, |name| name == "MOR_PLAN.md") {
                                // Send a specific trigger string to the main app loop
                                let _ = tx.send(String::from("[SYNC_TOC]"));
                            }
                        }
                    }
                }

                Err(e) => {
                    let _ = tx.send(format!("[ERROR] Watcher fault: {:?}", e));
                }
            }
        }
    });

    Ok(())
}