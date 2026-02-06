use dioxus::prelude::*;

use crate::levels::GroundTruth;
use crate::ui_node::{self, Rect};

const FILE_W: f32 = 80.0;
const FILE_H: f32 = 96.0;
const DROP_X: f32 = 500.0;
const DROP_Y: f32 = 200.0;
const DROP_W: f32 = 200.0;
const DROP_H: f32 = 160.0;
const FILE_X: f32 = 100.0;
const FILE_Y: f32 = 250.0;

#[component]
pub fn TestDrag() -> Element {
    let mut file_pos = use_signal(|| (FILE_X, FILE_Y));
    let mut drag_active = use_signal(|| false);
    let mut drag_off = use_signal(|| (0.0f32, 0.0f32));
    let mut result = use_signal(|| "idle".to_string());

    let (fx, fy) = file_pos();
    let dragging = drag_active();
    let z = if dragging { "200" } else { "10" };
    let pe = if dragging { "none" } else { "auto" };
    let opacity = if dragging { "0.85" } else { "1" };
    let shadow = if dragging {
        "0 8px 24px rgba(0,0,0,0.5)"
    } else {
        "0 2px 8px rgba(0,0,0,0.3)"
    };

    // Check if file center is over drop zone
    let cx = fx + FILE_W / 2.0;
    let cy = fy + FILE_H / 2.0;
    let drag_over =
        dragging && cx >= DROP_X && cx <= DROP_X + DROP_W && cy >= DROP_Y && cy <= DROP_Y + DROP_H;

    let dz_border = if drag_over { "#4f46e5" } else { "#d1d5db" };
    let dz_bg = if drag_over { "#eef2ff" } else { "white" };

    let result_text = result.read().clone();

    let tree = ui_node::card(
        Rect::new(0.0, 0.0, 800.0, 600.0),
        vec![
            ui_node::drag_source("test.txt", Rect::new(FILE_X, FILE_Y, FILE_W, FILE_H)),
            ui_node::drop_zone("Drop Zone", Rect::new(DROP_X, DROP_Y, DROP_W, DROP_H)),
        ],
    );

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: system-ui, sans-serif;",

            div {
                id: "viewport",
                "data-fixed": "true",
                style: "width: 800px; height: 600px; background: #1a1a2e; position: relative; overflow: hidden; user-select: none;",

                // Drop zone
                div {
                    class: "target",
                    "data-label": "Drop Zone",
                    style: "position: absolute; left: {DROP_X}px; top: {DROP_Y}px; width: {DROP_W}px; height: {DROP_H}px; background: {dz_bg}; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); display: flex; align-items: center; justify-content: center; box-sizing: border-box; transition: background 0.15s;",

                    div {
                        style: "border: 2px dashed {dz_border}; border-radius: 8px; padding: 20px 16px; text-align: center; width: 100%; box-sizing: border-box;",
                        div { style: "font-size: 28px; color: #9ca3af; margin-bottom: 6px;", "\u{2191}" }
                        div { style: "font-size: 14px; color: #374151; font-weight: 600;", "Drop Here" }
                    }
                }

                // File icon — uses onmousedown, same pattern as level15
                div {
                    class: "target",
                    "data-label": "test.txt",
                    style: "position: absolute; left: {fx}px; top: {fy}px; z-index: {z}; pointer-events: {pe}; cursor: grab; opacity: {opacity}; display: flex; flex-direction: column; align-items: center; user-select: none;",
                    onmousedown: move |e: Event<MouseData>| {
                        e.prevent_default();
                        drag_active.set(true);
                        result.set("dragging".to_string());
                        let coords = e.element_coordinates();
                        drag_off.set((coords.x as f32, coords.y as f32));
                    },

                    div {
                        style: "width: 56px; height: 68px; background: white; border-radius: 4px; border: 1px solid #d1d5db; position: relative; box-shadow: {shadow}; overflow: hidden;",
                        div { style: "position: absolute; top: 0; right: 0; width: 12px; height: 12px; background: #e5e7eb; border-bottom-left-radius: 2px;" }
                        div {
                            style: "position: absolute; bottom: 6px; left: 50%; transform: translateX(-50%); background: #6b7280; color: white; padding: 1px 5px; border-radius: 2px; font-size: 9px; font-weight: 700; font-family: monospace;",
                            "TXT"
                        }
                    }
                    div {
                        style: "font-size: 11px; color: #e5e7eb; margin-top: 4px; text-align: center; text-shadow: 0 1px 3px rgba(0,0,0,0.8);",
                        "test.txt"
                    }
                }

                // Drag overlay — captures mouse during drag (same pattern as level15)
                if dragging {
                    div {
                        style: "position: absolute; inset: 0; z-index: 100; cursor: grabbing;",
                        onmousemove: move |e: Event<MouseData>| {
                            let coords = e.element_coordinates();
                            let (ox, oy) = drag_off();
                            let nx = (coords.x as f32 - ox).clamp(0.0, 800.0 - FILE_W);
                            let ny = (coords.y as f32 - oy).clamp(0.0, 600.0 - FILE_H);
                            file_pos.set((nx, ny));
                        },
                        onmouseup: move |_| {
                            let (fx, fy) = file_pos();
                            let cx = fx + FILE_W / 2.0;
                            let cy = fy + FILE_H / 2.0;
                            let in_zone = cx >= DROP_X && cx <= DROP_X + DROP_W
                                && cy >= DROP_Y && cy <= DROP_Y + DROP_H;

                            if in_zone {
                                result.set("dropped".to_string());
                            } else {
                                result.set("cancelled".to_string());
                                file_pos.set((FILE_X, FILE_Y));
                            }
                            drag_active.set(false);
                        },
                        onmouseleave: move |_| {
                            result.set("cancelled".to_string());
                            file_pos.set((FILE_X, FILE_Y));
                            drag_active.set(false);
                        },
                    }
                }

                div {
                    id: "result",
                    style: "display: none;",
                    "{result_text}"
                }
            }

            GroundTruth {
                description: String::new(),
                target_x: DROP_X,
                target_y: DROP_Y,
                target_w: DROP_W,
                target_h: DROP_H,
                tree: Some(tree),
            }
        }
    }
}
