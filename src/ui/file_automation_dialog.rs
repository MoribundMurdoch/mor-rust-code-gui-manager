#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::path::PathBuf;

use crate::app_state::AppMode;
use crate::automation::register_module::{
    apply_register_module_preview,
    build_register_module_preview,
    guess_target_file_for_module,
    module_name_from_file_path,
    ModuleVisibility,
    RegisterModulePreview,
    RegisterModuleRequest,
};

#[derive(Clone, Debug, PartialEq)]
pub struct FileAutomationTarget {
    pub label: String,
    pub path: PathBuf,
}

#[component]
pub fn FileAutomationDialog(
    target: FileAutomationTarget,
    onclose: EventHandler<()>,
) -> Element {
    let app_mode = use_context::<Signal<AppMode>>();
    let logger = use_context::<Coroutine<String>>();

    let mut module_name = use_signal(|| {
        module_name_from_file_path(&target.path).unwrap_or_else(|_| String::new())
    });

    let mut target_file = use_signal(|| {
        guess_target_file_for_module(&target.path)
            .display()
            .to_string()
    });

    let mut visibility = use_signal(|| ModuleVisibility::Public);
    let mut register_preview = use_signal(|| None::<RegisterModulePreview>);

    let is_automate = app_mode.read().is_automate();

    rsx! {
        div {
            class: "automation-dialog-backdrop",

            div {
                class: "automation-dialog",

                div {
                    class: "automation-dialog-header",

                    div {
                        h3 { "File Automations" }
                        div {
                            class: "automation-dialog-subtitle",
                            "{target.label}"
                        }
                    }

                    button {
                        class: "automation-dialog-close",
                        onclick: move |_| onclose.call(()),
                        "Close"
                    }
                }

                div {
                    class: "automation-dialog-path",
                    "{target.path.display()}"
                }

                div {
                    class: "automation-dialog-mode",
                    strong { "{app_mode.read().label()}" }
                    span { " — {app_mode.read().description()}" }
                }

                div {
                    class: "automation-section",

                    h4 { "Module Registration" }

                    div {
                        class: "automation-form-row",

                        label { "Module name" }

                        input {
                            value: "{module_name.read()}",
                            oninput: move |evt| {
                                module_name.set(evt.value());
                                register_preview.set(None);
                            },
                        }
                    }

                    div {
                        class: "automation-form-row",

                        label { "Target file" }

                        input {
                            value: "{target_file.read()}",
                            oninput: move |evt| {
                                target_file.set(evt.value());
                                register_preview.set(None);
                            },
                        }
                    }

                    div {
                        class: "automation-form-row",

                        label { "Visibility" }

                        select {
                            value: "{visibility.read().label()}",
                            onchange: move |evt| {
                                let next = match evt.value().as_str() {
                                    "mod" => ModuleVisibility::Private,
                                    "pub(crate) mod" => ModuleVisibility::PublicCrate,
                                    _ => ModuleVisibility::Public,
                                };

                                visibility.set(next);
                                register_preview.set(None);
                            },

                            option { value: "pub mod", "pub mod" }
                            option { value: "mod", "mod" }
                            option { value: "pub(crate) mod", "pub(crate) mod" }
                        }
                    }

                    div {
                        class: "automation-action-buttons",

                        button {
                            onclick: move |_| {
                                let request = RegisterModuleRequest {
                                    target_file: PathBuf::from(target_file.read().as_str()),
                                    module_name: module_name.read().to_string(),
                                    visibility: visibility.read().clone(),
                                };

                                match build_register_module_preview(request) {
                                    Ok(preview) => {
                                        logger.send(preview.summary());
                                        register_preview.set(Some(preview));
                                    }
                                    Err(e) => {
                                        logger.send(format!("[ERROR] Register Module preview failed: {}", e));
                                        register_preview.set(None);
                                    }
                                }
                            },

                            "Preview"
                        }

                        button {
                            disabled: !is_automate || register_preview.read().is_none(),
                            title: if is_automate {
                                "Apply this module registration"
                            } else {
                                "Switch to Automate Mode to apply changes"
                            },

                            onclick: move |_| {
                                if !app_mode.read().is_automate() {
                                    logger.send(String::from(
                                        "[BLOCKED] Register Module requires Automate Mode."
                                    ));
                                    return;
                                }

                                let Some(preview) = register_preview.read().clone() else {
                                    logger.send(String::from(
                                        "[BLOCKED] Generate a preview before applying Register Module."
                                    ));
                                    return;
                                };

                                match apply_register_module_preview(&preview) {
                                    Ok(msg) => {
                                        logger.send(msg);
                                        register_preview.set(None);
                                    }
                                    Err(e) => {
                                        logger.send(format!("[ERROR] Register Module apply failed: {}", e));
                                    }
                                }
                            },

                            "Apply"
                        }
                    }

                    if let Some(preview) = register_preview.read().as_ref() {
                        pre {
                            class: "automation-preview",
                            "{preview.preview_diff}"
                        }
                    }
                }

                div {
                    class: "automation-section",

                    h4 { "Coming Soon" }

                    ul {
                        li { "Generate test module" }
                        li { "Generate documentation comments" }
                        li { "Show AST summary" }
                        li { "Detect missing module declaration" }
                        li { "Open Python-assisted automation hooks" }
                    }
                }
            }
        }
    }
}
