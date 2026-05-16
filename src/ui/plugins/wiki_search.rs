// src/ui/plugins/wiki_search.rs
#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app_state::PluginWikiSearchOpen;

pub fn WikiSearchPlugin() -> Element {
    let mut wiki_search_open = use_context::<PluginWikiSearchOpen>().0;
    let logger = use_context::<Coroutine<String>>();

    let mut query = use_signal(|| String::new());

    let mut pos_x = use_signal(|| 60.0f64);
    let mut pos_y = use_signal(|| 60.0f64);
    let mut is_dragging = use_signal(|| false);
    let mut drag_offset = use_signal(|| (0.0f64, 0.0f64));

    if !*wiki_search_open.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "osrs-plugin-window",
            style: "left: {pos_x}px; top: {pos_y}px; width: 400px; height: 180px; z-index: 9999;",

            onmousemove: move |evt| {
                if *is_dragging.read() {
                    let coords = evt.client_coordinates();
                    let offset = *drag_offset.read();

                    pos_x.set(coords.x - offset.0);
                    pos_y.set(coords.y - offset.1);
                }
            },

            onmouseup: move |_| {
                is_dragging.set(false);
            },

            onmouseleave: move |_| {
                is_dragging.set(false);
            },

            div {
                style: "background-color: #121014; border-bottom: 2px solid var(--mor-border-mid); padding: 8px 12px; cursor: grab; display: flex; justify-content: space-between; align-items: center; user-select: none;",

                onmousedown: move |evt| {
                    is_dragging.set(true);

                    let coords = evt.client_coordinates();

                    drag_offset.set((
                        coords.x - *pos_x.read(),
                        coords.y - *pos_y.read(),
                    ));
                },

                div {
                    style: "color: var(--mor-accent-bright); font-size: 14px; font-weight: bold;",
                    "Rust Wiki Link"
                }

                div {
                    style: "color: var(--mor-text-dim); font-size: 11px; font-style: italic;",
                    "(Drag)"
                }
            }

            div {
                class: "osrs-plugin-content",
                style: "justify-content: center; align-items: center;",

                input {
                    style: "width: 100%; background: #050506; color: var(--mor-text); border: 1px solid var(--mor-accent-purple); padding: 8px; font-family: var(--mor-font-code); outline: none;",
                    placeholder: "e.g., PathBuf, Arc, serde...",
                    autofocus: true,
                    value: "{query.read()}",

                    oninput: move |evt| {
                        query.set(evt.value());
                    },

                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter {
                            crate::wiki_lookup::search_rust_docs(&query.read(), &logger);
                        }
                    }
                }
            }

            div {
                class: "osrs-plugin-tabs",

                button {
                    class: "osrs-plugin-tab",
                    onclick: move |_| {
                        wiki_search_open.set(false);
                    },
                    "Close"
                }

                button {
                    class: "osrs-plugin-tab active",
                    onclick: move |_| {
                        crate::wiki_lookup::search_rust_docs(&query.read(), &logger);
                    },
                    "Search std"
                }

                button {
                    class: "osrs-plugin-tab active",
                    onclick: move |_| {
                        crate::wiki_lookup::search_crates_io(&query.read(), &logger);
                    },
                    "Search docs.rs"
                }
            }
        }
    }
}