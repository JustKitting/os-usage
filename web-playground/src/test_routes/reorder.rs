use dioxus::prelude::*;

use crate::levels::GroundTruth;
use crate::ui_node::{self, Rect};

const ITEMS: &[&str] = &["Alpha", "Beta", "Gamma", "Delta"];
const CARD_X: f32 = 200.0;
const CARD_Y: f32 = 100.0;
const CARD_W: f32 = 340.0;
const ITEM_H: f32 = 44.0;
const ITEM_GAP: f32 = 4.0;
const LIST_TOP: f32 = 60.0;
const ACCENT: &str = "#4f46e5";

fn item_y(i: usize) -> f32 {
    i as f32 * (ITEM_H + ITEM_GAP)
}

#[component]
pub fn TestReorder() -> Element {
    let mut order = use_signal(|| vec![0usize, 1, 2, 3]);
    let mut drag_idx = use_signal(|| None::<usize>);
    let mut drag_start_page_y = use_signal(|| 0.0f32);
    let mut drag_start_item_y = use_signal(|| 0.0f32);
    let mut drag_y = use_signal(|| 0.0f32);
    let mut swap_count = use_signal(|| 0u32);

    let cur_order: Vec<usize> = order.read().clone();
    let cur_drag = drag_idx();
    let count = cur_order.len();

    let result = if swap_count() > 0 && cur_drag.is_none() {
        let labels: Vec<&str> = cur_order.iter().map(|&i| ITEMS[i]).collect();
        format!("reordered:{}", labels.join(","))
    } else if let Some(di) = cur_drag {
        format!("dragging:{}", ITEMS[cur_order[di]])
    } else {
        "idle".to_string()
    };

    let list_h = count as f32 * (ITEM_H + ITEM_GAP) - ITEM_GAP;
    let card_h = LIST_TOP + list_h + 16.0;
    let card_rect = Rect::new(CARD_X, CARD_Y, CARD_W, card_h);
    let children: Vec<_> = cur_order
        .iter()
        .map(|&si| ui_node::button(ITEMS[si], card_rect))
        .collect();
    let tree = ui_node::card(card_rect, children);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: system-ui, sans-serif;",

            div {
                id: "viewport",
                "data-fixed": "true",
                style: "width: 800px; height: 600px; background: #1a1a2e; position: relative; overflow: hidden; user-select: none;",

                div {
                    style: "position: absolute; left: {CARD_X}px; top: {CARD_Y}px; width: {CARD_W}px; \
                            background: white; border-radius: 12px; \
                            box-shadow: 0 4px 24px rgba(0,0,0,0.3); \
                            font-family: system-ui, sans-serif; box-sizing: border-box; padding: 16px;",

                    h3 {
                        style: "margin: 0 0 12px 0; font-size: 16px; color: #111827; font-weight: 600;",
                        "Test List"
                    }

                    p {
                        style: "margin: 0 0 12px 0; font-size: 12px; color: #9ca3af;",
                        "Drag items to reorder"
                    }

                    div {
                        id: "list-container",
                        style: "position: relative; height: {list_h}px;",

                        for di in 0..count {
                            {
                                let si = cur_order[di];
                                let label = ITEMS[si];
                                let is_dragged = cur_drag == Some(di);

                                let top = if is_dragged { drag_y() } else { item_y(di) };
                                let z = if is_dragged { "200" } else { "1" };
                                let pe = if is_dragged { "none" } else { "auto" };
                                let opacity = if is_dragged { "0.85" } else { "1" };
                                let shadow = if is_dragged {
                                    "0 8px 24px rgba(0,0,0,0.3)"
                                } else {
                                    "none"
                                };
                                let bg = if is_dragged {
                                    format!("{}22", ACCENT)
                                } else {
                                    "#f9fafb".to_string()
                                };
                                let border = if is_dragged {
                                    format!("2px solid {}", ACCENT)
                                } else {
                                    "2px solid transparent".to_string()
                                };
                                let transition = if is_dragged { "none" } else { "top 0.15s ease" };

                                rsx! {
                                    button {
                                        class: "target",
                                        "data-label": "{label}",
                                        "data-index": "{di}",
                                        style: "position: absolute; top: {top}px; left: 0; width: 100%; \
                                                height: {ITEM_H}px; z-index: {z}; pointer-events: {pe}; \
                                                opacity: {opacity}; box-shadow: {shadow}; \
                                                display: flex; align-items: center; gap: 10px; \
                                                padding: 10px 12px; background: {bg}; \
                                                border: {border}; border-radius: 8px; font-size: 14px; \
                                                color: #374151; cursor: grab; text-align: left; \
                                                font-family: system-ui, sans-serif; box-sizing: border-box; \
                                                transition: {transition};",
                                        tabindex: "-1",
                                        onmousedown: move |e: Event<MouseData>| {
                                            e.prevent_default();
                                            drag_idx.set(Some(di));
                                            drag_start_page_y.set(e.page_coordinates().y as f32);
                                            drag_start_item_y.set(item_y(di));
                                            drag_y.set(item_y(di));
                                        },
                                        span {
                                            style: "color: #d1d5db; font-size: 14px; flex-shrink: 0;",
                                            "\u{2261}"
                                        }
                                        span {
                                            class: "position-number",
                                            style: "color: #9ca3af; font-size: 12px; width: 18px; flex-shrink: 0; font-family: monospace;",
                                            "{di + 1}."
                                        }
                                        span {
                                            class: "item-label",
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Drag overlay — at viewport level to capture all mouse movement
                if cur_drag.is_some() {
                    div {
                        style: "position: absolute; inset: 0; z-index: 100; cursor: grabbing;",
                        onmousemove: move |e: Event<MouseData>| {
                            if let Some(mut di) = drag_idx() {
                                let delta = e.page_coordinates().y as f32 - drag_start_page_y();
                                let max_y = item_y(count - 1);
                                let new_y = (drag_start_item_y() + delta).clamp(0.0, max_y);
                                drag_y.set(new_y);

                                let dragged_center = new_y + ITEM_H / 2.0;

                                // Check swap with item above
                                if di > 0 {
                                    let above_center = item_y(di - 1) + ITEM_H / 2.0;
                                    if dragged_center < above_center {
                                        order.write().swap(di, di - 1);
                                        di -= 1;
                                        drag_idx.set(Some(di));
                                        swap_count.set(swap_count() + 1);
                                    }
                                }
                                // Check swap with item below
                                if di < count - 1 {
                                    let below_center = item_y(di + 1) + ITEM_H / 2.0;
                                    if dragged_center > below_center {
                                        order.write().swap(di, di + 1);
                                        drag_idx.set(Some(di + 1));
                                        swap_count.set(swap_count() + 1);
                                    }
                                }
                            }
                        },
                        onmouseup: move |_| {
                            drag_idx.set(None);
                        },
                        onmouseleave: move |_| {
                            drag_idx.set(None);
                        },
                    }
                }

                div {
                    id: "result",
                    style: "display: none;",
                    "{result}"
                }
            }

            GroundTruth {
                description: String::new(),
                target_x: CARD_X,
                target_y: CARD_Y,
                target_w: CARD_W,
                target_h: card_h,
                tree: Some(tree),
            }
        }
    }
}
