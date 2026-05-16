#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::fs;

use crate::app_state::{ActiveEditorFile, AppMode, EditorSaveRequest};

pub fn CodeEditor() -> Element {
    let mut active_file = use_context::<ActiveEditorFile>().0;
    let app_mode = use_context::<Signal<AppMode>>();
    let logger = use_context::<Coroutine<String>>();
    let save_request = use_context::<EditorSaveRequest>().0;

    let path_opt = active_file.read().clone();

    if let Some(path) = path_opt {
        let is_automate = app_mode.read().is_automate();
        let initial_content = fs::read_to_string(&path).unwrap_or_default();
        let mut content = use_signal(|| initial_content.clone());

        let save_current_buffer = {
            let path = path.clone();
            let logger = logger.clone();

            move || {
                if !app_mode.read().is_automate() {
                    logger.send(String::from(
                        "[BLOCKED] Save requires Automate Mode."
                    ));
                    return;
                }

                match fs::write(&path, content.read().as_str()) {
                    Ok(_) => logger.send(format!("[SYSTEM] Saved: {}", path.display())),
                    Err(e) => logger.send(format!("[ERROR] Save failed: {}", e)),
                }
            }
        };

        // File > Save and Ctrl+S increment this signal from the native menu bridge.
        // Reading the signal makes this component re-run when the request changes.
        let requested_save_count = *save_request.read();
        if requested_save_count > 0 {
            save_current_buffer();
        }

        let mut editor_eval = eval(r###"
            async function loadCss(href) {
                if (document.querySelector(`link[href="${href}"]`)) return;
                let l = document.createElement('link');
                l.rel = 'stylesheet';
                l.href = href;
                document.head.appendChild(l);
            }

            async function loadScript(src) {
                return new Promise((resolve, reject) => {
                    if (document.querySelector(`script[src="${src}"]`)) {
                        resolve();
                        return;
                    }

                    let s = document.createElement('script');
                    s.src = src;
                    s.onload = resolve;
                    s.onerror = reject;
                    document.head.appendChild(s);
                });
            }

            (async function init() {
                try {
                    let mount;

                    while (!(mount = document.getElementById("cm-mount"))) {
                        await new Promise(r => setTimeout(r, 50));
                    }

                    mount.innerHTML = '';

                    // 1. Load CodeMirror core + Rust syntax mode
                    await loadCss("https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.13/codemirror.min.css");
                    await loadScript("https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.13/codemirror.min.js");
                    await loadScript("https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.13/mode/rust/rust.min.js");

                    // 2. Load autocomplete hint addons
                    await loadCss("https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.13/addon/hint/show-hint.min.css");
                    await loadScript("https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.13/addon/hint/show-hint.min.js");
                    await loadScript("https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.13/addon/hint/anyword-hint.min.js");

                    let initText = await dioxus.recv();
                    let isAutomate = await dioxus.recv();

                    let editor = CodeMirror(mount, {
                        value: initText,
                        mode: "rust",
                        theme: "purple-ink",
                        lineNumbers: true,
                        readOnly: isAutomate ? false : true,

                        extraKeys: {
                            "Ctrl-Space": "autocomplete",

                            // Alt-D: search Rust docs for selected text or word under cursor
                            "Alt-D": function(cm) {
                                let text = cm.getSelection();

                                if (!text) {
                                    let word = cm.findWordAt(cm.getCursor());
                                    if (word) {
                                        text = cm.getRange(word.anchor, word.head);
                                    }
                                }

                                if (text && text.trim().length > 0) {
                                    dioxus.send("DOC_SEARCH:" + text.trim());
                                }
                            }
                        }
                    });

                    editor.setSize("100%", "100%");

                    // Custom editor context menu.
                    //
                    // The app disables the native WebView context menu globally so graph nodes
                    // can own right-click behavior. CodeMirror still needs copy/cut/paste, so
                    // give the editor its own local menu.
                    let existingMenu = document.getElementById("mor-editor-context-menu");
                    if (existingMenu) {
                        existingMenu.remove();
                    }

                    let editorMenu = document.createElement("div");
                    editorMenu.id = "mor-editor-context-menu";
                    editorMenu.style.position = "fixed";
                    editorMenu.style.zIndex = "99999";
                    editorMenu.style.display = "none";
                    editorMenu.style.minWidth = "150px";
                    editorMenu.style.padding = "4px";
                    editorMenu.style.background = "#0d0d0f";
                    editorMenu.style.border = "1px solid #8a5cf6";
                    editorMenu.style.boxShadow = "0 8px 24px rgba(0, 0, 0, 0.45)";
                    editorMenu.style.color = "#f0e8ff";
                    editorMenu.style.fontFamily = "Georgia, serif";

                    function makeMenuButton(label, handler) {
                        let button = document.createElement("button");
                        button.textContent = label;
                        button.style.display = "block";
                        button.style.width = "100%";
                        button.style.padding = "6px 10px";
                        button.style.border = "none";
                        button.style.background = "transparent";
                        button.style.color = "#f0e8ff";
                        button.style.textAlign = "left";
                        button.style.cursor = "pointer";
                        button.onmouseenter = () => button.style.background = "#3c2a5d";
                        button.onmouseleave = () => button.style.background = "transparent";
                        button.onclick = async (event) => {
                            event.preventDefault();
                            event.stopPropagation();
                            editorMenu.style.display = "none";
                            editor.focus();
                            await handler();
                        };
                        return button;
                    }

                    editorMenu.appendChild(makeMenuButton("Copy", async () => {
                        let text = editor.getSelection();
                        if (text && navigator.clipboard) {
                            await navigator.clipboard.writeText(text);
                        } else {
                            document.execCommand("copy");
                        }
                    }));

                    editorMenu.appendChild(makeMenuButton("Cut", async () => {
                        if (editor.getOption("readOnly")) return;

                        let text = editor.getSelection();
                        if (text && navigator.clipboard) {
                            await navigator.clipboard.writeText(text);
                            editor.replaceSelection("");
                        } else {
                            document.execCommand("cut");
                        }
                    }));

                    editorMenu.appendChild(makeMenuButton("Paste", async () => {
                        if (editor.getOption("readOnly")) return;

                        // Programmatic browser paste is often blocked even inside desktop WebViews.
                        // Ask Rust to read the native clipboard, then send the pasted text back
                        // through the Dioxus eval channel.
                        dioxus.send("PASTE_REQUEST");
                    }));

                    editorMenu.appendChild(makeMenuButton("Select All", async () => {
                        editor.execCommand("selectAll");
                    }));

                    document.body.appendChild(editorMenu);

                    function hideEditorMenu() {
                        editorMenu.style.display = "none";
                    }

                    editor.getWrapperElement().addEventListener("contextmenu", (event) => {
                        event.preventDefault();
                        event.stopPropagation();

                        editor.focus();

                        let menuWidth = 160;
                        let menuHeight = 140;
                        let x = Math.min(event.clientX, window.innerWidth - menuWidth - 8);
                        let y = Math.min(event.clientY, window.innerHeight - menuHeight - 8);

                        editorMenu.style.left = Math.max(8, x) + "px";
                        editorMenu.style.top = Math.max(8, y) + "px";
                        editorMenu.style.display = "block";
                    }, true);

                    document.addEventListener("mousedown", (event) => {
                        if (!editorMenu.contains(event.target)) {
                            hideEditorMenu();
                        }
                    }, true);

                    document.addEventListener("keydown", (event) => {
                        if (event.key === "Escape") {
                            hideEditorMenu();
                        }
                    }, true);

                    (async function receiveEditorCommands() {
                        while (true) {
                            let command = await dioxus.recv();

                            if (typeof command !== "string") {
                                continue;
                            }

                            if (command.startsWith("PASTE_TEXT:")) {
                                editor.focus();
                                editor.replaceSelection(command.slice("PASTE_TEXT:".length));
                            }
                        }
                    })();

                    // Prefix standard text updates so Rust can distinguish them
                    editor.on("change", (cm) => {
                        dioxus.send("TXT:" + cm.getValue());
                    });

                    // Automatically trigger autocomplete while typing letters
                    editor.on("keyup", function (cm, event) {
                        if (
                            !cm.state.completionActive &&
                            event.keyCode >= 65 &&
                            event.keyCode <= 90
                        ) {
                            CodeMirror.commands.autocomplete(cm, null, {
                                completeSingle: false
                            });
                        }
                    });

                } catch (err) {
                    console.error("[CodeMirror Init Error]:", err);
                }
            })();
        "###);

        let _ = editor_eval.send(initial_content.into());
        let _ = editor_eval.send(is_automate.into());

        use_future(move || async move {
            while let Ok(js_content) = editor_eval.recv().await {
                if let Some(text) = js_content.as_str() {
                    if text.starts_with("TXT:") {
                        content.set(text["TXT:".len()..].to_string());
                    } else if text == "PASTE_REQUEST" {
                        match arboard::Clipboard::new()
                            .and_then(|mut clipboard| clipboard.get_text())
                        {
                            Ok(clipboard_text) => {
                                let _ = editor_eval.send(
                                    format!("PASTE_TEXT:{}", clipboard_text).into()
                                );
                            }
                            Err(e) => {
                                logger.send(format!("[ERROR] Clipboard paste failed: {}", e));
                            }
                        }
                    } else if text.starts_with("DOC_SEARCH:") {
                        let query = &text["DOC_SEARCH:".len()..];
                        crate::wiki_lookup::search_rust_docs(query, &logger);
                    }
                }
            }
        });

        rsx! {
            div {
                class: "embedded-editor-container",

                div {
                    class: "editor-header",

                    div {
                        class: "editor-title",
                        "{path.display()}"
                    }

                    div {
                        class: "editor-actions",

                        button {
                            class: "safety-mode-btn active",
                            disabled: !is_automate,
                            onclick: move |_| {
                                save_current_buffer();
                            },
                            if is_automate { "Save File" } else { "Read-Only (Requires Automate)" }
                        }

                        button {
                            class: "safety-mode-btn danger",
                            onclick: move |_| active_file.set(None),
                            "Close Editor"
                        }
                    }
                }

                div {
                    id: "cm-mount",
                    style: "flex-grow: 1; overflow: hidden; width: 100%; height: 100%; position: relative;"
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}