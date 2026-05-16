#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::app_state::PluginNodeEditorOpen;

pub fn NodeEditorPlugin() -> Element {
    let mut node_editor_open = use_context::<PluginNodeEditorOpen>().0;

    // --- Placeholder Node 1: Enum (Mode) ---
    let mut pos1_x = use_signal(|| 80.0f64);
    let mut pos1_y = use_signal(|| 120.0f64);
    let mut drag1 = use_signal(|| false);

    // --- Placeholder Node 2: Struct (Config) ---
    let mut pos2_x = use_signal(|| 450.0f64);
    let mut pos2_y = use_signal(|| 80.0f64);
    let mut drag2 = use_signal(|| false);

    // --- Placeholder Node 3: Function (Init) ---
    let mut pos3_x = use_signal(|| 850.0f64);
    let mut pos3_y = use_signal(|| 180.0f64);
    let mut drag3 = use_signal(|| false);

    let mut drag_offset = use_signal(|| (0.0, 0.0));

    if !*node_editor_open.read() {
        return rsx! { div {} };
    }

    // Spline 1: Connect Enum Output to Struct Input
    let s1_start_x = *pos1_x.read() + 220.0;
    let s1_start_y = *pos1_y.read() + 65.0; 
    let s1_end_x = *pos2_x.read();
    let s1_end_y = *pos2_y.read() + 65.0; 
    let s1_ctrl_x1 = s1_start_x + 100.0;
    let s1_ctrl_x2 = s1_end_x - 100.0;
    let path_1 = format!("M {},{} C {},{} {},{} {},{}", s1_start_x, s1_start_y, s1_ctrl_x1, s1_start_y, s1_ctrl_x2, s1_end_y, s1_end_x, s1_end_y);

    // Spline 2: Connect Struct Output to Function Input
    let s2_start_x = *pos2_x.read() + 220.0;
    let s2_start_y = *pos2_y.read() + 40.0; 
    let s2_end_x = *pos3_x.read();
    let s2_end_y = *pos3_y.read() + 40.0; 
    let s2_ctrl_x1 = s2_start_x + 100.0;
    let s2_ctrl_x2 = s2_end_x - 100.0;
    let path_2 = format!("M {},{} C {},{} {},{} {},{}", s2_start_x, s2_start_y, s2_ctrl_x1, s2_start_y, s2_ctrl_x2, s2_end_y, s2_end_x, s2_end_y);

    rsx! {
        div {
            class: "blueprint-workspace",
            
            // SVG Canvas for Wires
            svg {
                class: "bp-svg-layer",
                path { class: "bp-spline", d: "{path_1}" }
                path { class: "bp-spline", style: "stroke: #f472b6;", d: "{path_2}" }
            }

            // Top-right Controls
            div {
                style: "position: absolute; top: 16px; right: 16px; z-index: 100; display: flex; gap: 8px;",
                div {
                    style: "background: rgba(0,0,0,0.5); padding: 8px 16px; color: var(--mor-text-dim); font-family: var(--mor-font-code); font-size: 12px; border: 1px solid var(--mor-border-dark);",
                    "EXPERIMENTAL V0.1"
                }
                button {
                    class: "safety-mode-btn danger",
                    onclick: move |_| node_editor_open.set(false),
                    "Exit Node Editor"
                }
            }

            // --- NODE 1: Enum ---
            div {
                class: "bp-node",
                style: "left: {pos1_x}px; top: {pos1_y}px;",
                onmousemove: move |evt| { if *drag1.read() { let o = *drag_offset.read(); let c = evt.client_coordinates(); pos1_x.set(c.x - o.0); pos1_y.set(c.y - o.1); } },
                onmouseup: move |_| drag1.set(false), onmouseleave: move |_| drag1.set(false),
                div {
                    class: "bp-header enum",
                    onmousedown: move |evt| { drag1.set(true); drag_offset.set((evt.client_coordinates().x - *pos1_x.read(), evt.client_coordinates().y - *pos1_y.read())); },
                    div { class: "bp-pin execution" }
                    span { "Enum: AppMode" }
                }
                div {
                    class: "bp-body",
                    div { class: "bp-row", span { "Inspect" }, div { class: "bp-pin", style: "background: #60a5fa;" } }
                    div { class: "bp-row", span { "Automate" }, div { class: "bp-pin", style: "background: #60a5fa;" } }
                }
            }

            // --- NODE 2: Struct ---
            div {
                class: "bp-node",
                style: "left: {pos2_x}px; top: {pos2_y}px;",
                onmousemove: move |evt| { if *drag2.read() { let o = *drag_offset.read(); let c = evt.client_coordinates(); pos2_x.set(c.x - o.0); pos2_y.set(c.y - o.1); } },
                onmouseup: move |_| drag2.set(false), onmouseleave: move |_| drag2.set(false),
                div {
                    class: "bp-header struct",
                    onmousedown: move |evt| { drag2.set(true); drag_offset.set((evt.client_coordinates().x - *pos2_x.read(), evt.client_coordinates().y - *pos2_y.read())); },
                    span { "Struct: Config" }
                    div { class: "bp-pin execution" }
                }
                div {
                    class: "bp-body",
                    div { class: "bp-row", div { class: "bp-pin", style: "background: #60a5fa;" }, span { "mode: AppMode" } }
                    div { class: "bp-row", div { class: "bp-pin", style: "background: #10b981;" }, span { "version: u32" } }
                    div { class: "bp-row", div { class: "bp-pin", style: "background: #f472b6;" }, span { "path: String" } }
                }
            }

            // --- NODE 3: Function ---
            div {
                class: "bp-node",
                style: "left: {pos3_x}px; top: {pos3_y}px; width: 260px;",
                onmousemove: move |evt| { if *drag3.read() { let o = *drag_offset.read(); let c = evt.client_coordinates(); pos3_x.set(c.x - o.0); pos3_y.set(c.y - o.1); } },
                onmouseup: move |_| drag3.set(false), onmouseleave: move |_| drag3.set(false),
                div {
                    class: "bp-header", style: "background: linear-gradient(180deg, #7c2d12 0%, #431407 100%); border-bottom: 2px solid #2a0a02;",
                    onmousedown: move |evt| { drag3.set(true); drag_offset.set((evt.client_coordinates().x - *pos3_x.read(), evt.client_coordinates().y - *pos3_y.read())); },
                    div { class: "bp-pin execution" }
                    span { "fn: init_workspace()" }
                    div { class: "bp-pin execution" }
                }
                div {
                    class: "bp-body",
                    div { class: "bp-row", div { class: "bp-pin execution" }, span { "config: Config" }, div { class: "bp-pin", style: "background: #a78bfa;" } }
                }
            }
        }
    }
}
