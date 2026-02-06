use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg, ordinal};

struct ListScenario {
    title: &'static str,
    items: &'static [&'static str],
}

const SCENARIOS: &[ListScenario] = &[
    ListScenario { title: "Priority Tasks", items: &[
        "Fix login bug", "Deploy to staging", "Write unit tests", "Update docs",
        "Review PR #42", "Refactor auth", "Add logging", "Setup CI",
    ]},
    ListScenario { title: "Playlist", items: &[
        "Bohemian Rhapsody", "Hotel California", "Stairway to Heaven",
        "Imagine", "Yesterday", "Hey Jude", "Wonderwall", "Creep",
    ]},
    ListScenario { title: "Shopping List", items: &[
        "Milk", "Eggs", "Bread", "Butter", "Cheese",
        "Apples", "Rice", "Chicken",
    ]},
    ListScenario { title: "Travel Itinerary", items: &[
        "Book flights", "Reserve hotel", "Rent car", "Pack bags",
        "Get passport", "Buy insurance", "Plan route", "Exchange currency",
    ]},
    ListScenario { title: "Recipe Steps", items: &[
        "Preheat oven", "Mix dry ingredients", "Beat eggs", "Combine wet and dry",
        "Pour into pan", "Bake 25 min", "Cool on rack", "Add frosting",
    ]},
    ListScenario { title: "Sprint Backlog", items: &[
        "FEAT-101", "BUG-203", "FEAT-105", "CHORE-44",
        "BUG-210", "FEAT-112", "CHORE-51", "BUG-215",
    ]},
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

const ITEM_H: f32 = 44.0;
const ITEM_GAP: f32 = 4.0;
const LIST_TOP: f32 = 60.0; // Space for title + hint within card

fn item_y(i: usize) -> f32 {
    i as f32 * (ITEM_H + ITEM_GAP)
}

struct Level25State {
    scenario_idx: usize,
    order: Vec<usize>,
    target_item: usize,
    target_pos: usize,
    style: u8,
    accent: String,
    card_x: f32,
    card_y: f32,
    card_w: f32,
}

fn random_level25() -> Level25State {
    let mut rng = fresh_rng();
    let scenario_idx = rng.random_range(0..SCENARIOS.len());
    let scenario = &SCENARIOS[scenario_idx];

    let count = rng.random_range(5..=7usize).min(scenario.items.len());
    let mut pool: Vec<usize> = (0..scenario.items.len()).collect();
    let mut order = Vec::new();
    for _ in 0..count {
        let idx = rng.random_range(0..pool.len());
        order.push(pool.remove(idx));
    }

    let item_idx_in_order = rng.random_range(0..count);
    let target_item = order[item_idx_in_order];

    let mut target_pos = rng.random_range(0..count);
    while target_pos == item_idx_in_order {
        target_pos = rng.random_range(0..count);
    }

    let style = rng.random_range(0..3u8);
    let accent = ACCENT_COLORS[rng.random_range(0..ACCENT_COLORS.len())].to_string();

    let card_w = rng.random_range(280.0..=380.0f32);
    let list_h = count as f32 * (ITEM_H + ITEM_GAP) - ITEM_GAP;
    let card_h = LIST_TOP + list_h + 16.0 + 56.0; // list + bottom padding + submit button
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (card_x, card_y) = super::safe_position_in(&mut rng, card_w, card_h, 60.0, vp_w * 1.3, vp_h * 1.3);

    Level25State { scenario_idx, order, target_item, target_pos, style, accent, card_x, card_y, card_w }
}

#[component]
pub fn Level25() -> Element {
    let mut state = use_signal(|| random_level25());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong = use_signal(|| false);
    let initial_order = state.read().order.clone();
    let mut order = use_signal(move || initial_order);

    // Drag state
    let mut drag_idx = use_signal(|| None::<usize>);
    let mut drag_start_page_y = use_signal(|| 0.0f32);
    let mut drag_start_item_y = use_signal(|| 0.0f32);
    let mut drag_y = use_signal(|| 0.0f32);

    let st = state.read();
    let scenario = &SCENARIOS[st.scenario_idx];
    let title = scenario.title;
    let target_item = st.target_item;
    let target_pos = st.target_pos;
    let style = st.style;
    let accent = st.accent.clone();
    let card_x = st.card_x;
    let card_y = st.card_y;
    let card_w = st.card_w;
    drop(st);

    let cur_order: Vec<usize> = order.read().clone();
    let item_count = cur_order.len();
    let is_wrong = wrong();
    let cur_drag = drag_idx();

    let target_label = scenario.items[target_item];
    let target_ord = ordinal(target_pos + 1);
    let instruction = format!("Move \"{}\" to {} position", target_label, target_ord);

    let is_correct = cur_order.get(target_pos) == Some(&target_item);

    let border_radius = match style { 0 => "16px", 1 => "6px", _ => "10px" };
    let item_radius = match style { 0 => "10px", 1 => "4px", _ => "6px" };
    let list_h = item_count as f32 * (ITEM_H + ITEM_GAP) - ITEM_GAP;
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; \
         background: white; border-radius: {}; \
         box-shadow: 0 4px 24px rgba(0,0,0,0.3); \
         font-family: system-ui, sans-serif; box-sizing: border-box; padding: 16px;",
        card_x, card_y, card_w, border_radius
    );

    let submit_bg = if is_wrong { "#ef4444" } else { &accent };

    // Ground truth
    let card_h_est = LIST_TOP + list_h + 16.0 + 56.0;
    let card_rect = Rect::new(card_x, card_y, card_w, card_h_est);
    let children: Vec<_> = cur_order.iter().map(|&si| {
        let label = scenario.items[si];
        let item_rect = Rect::new(card_x, card_y, card_w, card_h_est);
        if si == target_item {
            ui_node::target_button(label, item_rect)
        } else {
            ui_node::button(label, item_rect)
        }
    }).collect();
    let tree = ui_node::form(card_rect, "Submit", children);
    let description = String::new();
    let viewport_style = format!("{} user-select: none;", super::viewport_style(&bg(), true));

    rsx! {
        div {
            style: "min-height: 100vh; background: #0f0f1a; display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: system-ui, sans-serif;",

            div {
                style: "display: flex; gap: 16px; align-items: center; margin-bottom: 16px;",
                Link {
                    to: Route::LevelSelect {},
                    style: "color: #6b7280; text-decoration: none; font-size: 14px;",
                    "\u{2190} Levels"
                }
                h2 {
                    style: "color: #e5e7eb; margin: 0; font-size: 20px;",
                    "Level 25"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Sortable List"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            div {
                id: "viewport",
                style: "{viewport_style}",

                // Instruction
                div {
                    style: "position: absolute; left: 0; right: 0; top: 16px; text-align: center; z-index: 30;",
                    div {
                        style: "display: inline-block; background: rgba(0,0,0,0.7); padding: 8px 16px; border-radius: 8px; color: white; font-size: 14px; font-weight: 500;",
                        "{instruction}"
                    }
                }

                div {
                    style: "{card_style}",

                    // Title
                    h3 {
                        style: "margin: 0 0 12px 0; font-size: 16px; color: #111827; font-weight: 600;",
                        "{title}"
                    }

                    // Hint
                    p {
                        style: "margin: 0 0 12px 0; font-size: 12px; color: #9ca3af;",
                        "Drag items to reorder"
                    }

                    // List items — relatively positioned container with absolute items
                    div {
                        style: "position: relative; height: {list_h}px;",

                        for di in 0..item_count {
                            {
                                let si = cur_order[di];
                                let label = scenario.items[si];
                                let is_dragged = cur_drag == Some(di);
                                let is_target_item = si == target_item;
                                let is_target_pos = di == target_pos;
                                let accent_c = accent.clone();

                                let top = if is_dragged { drag_y() } else { item_y(di) };
                                let z = if is_dragged { "200" } else { "1" };
                                let pe = if is_dragged { "none" } else { "auto" };
                                let opacity = if is_dragged { "0.85" } else { "1" };
                                let shadow = if is_dragged {
                                    "0 8px 24px rgba(0,0,0,0.3)".to_string()
                                } else {
                                    "none".to_string()
                                };

                                let item_bg = if is_dragged {
                                    format!("{}22", accent_c)
                                } else {
                                    "#f9fafb".to_string()
                                };
                                let item_border = if is_dragged {
                                    format!("2px solid {}", accent_c)
                                } else if is_target_pos {
                                    "2px dashed #d1d5db".to_string()
                                } else {
                                    "2px solid transparent".to_string()
                                };
                                let font_weight = if is_target_item { "600" } else { "400" };
                                let transition = if is_dragged { "none" } else { "top 0.15s ease" };

                                let item_style = format!(
                                    "position: absolute; top: {top}px; left: 0; width: 100%; \
                                     height: {ITEM_H}px; z-index: {z}; pointer-events: {pe}; \
                                     opacity: {opacity}; box-shadow: {shadow}; \
                                     display: flex; align-items: center; gap: 10px; \
                                     padding: 10px 12px; background: {item_bg}; \
                                     border: {item_border}; border-radius: {item_radius}; font-size: 14px; \
                                     color: #374151; cursor: grab; text-align: left; \
                                     font-family: system-ui, sans-serif; box-sizing: border-box; \
                                     transition: {transition}; font-weight: {font_weight};"
                                );

                                rsx! {
                                    button {
                                        class: if is_target_item { "target" } else { "" },
                                        "data-label": "{label}",
                                        style: "{item_style}",
                                        tabindex: "-1",
                                        onmousedown: move |e: Event<MouseData>| {
                                            e.prevent_default();
                                            wrong.set(false);
                                            drag_idx.set(Some(di));
                                            drag_start_page_y.set(e.page_coordinates().y as f32);
                                            drag_start_item_y.set(item_y(di));
                                            drag_y.set(item_y(di));
                                        },
                                        // Grip handle
                                        span {
                                            style: "color: #d1d5db; font-size: 14px; flex-shrink: 0;",
                                            "\u{2261}"
                                        }
                                        // Position number
                                        span {
                                            style: "color: #9ca3af; font-size: 12px; width: 18px; flex-shrink: 0; font-family: monospace;",
                                            "{di + 1}."
                                        }
                                        span { "{label}" }
                                    }
                                }
                            }
                        }
                    }

                    // Submit
                    button {
                        "data-label": "Submit",
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: {item_radius}; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s; margin-top: 12px;",
                        tabindex: "-1",
                        onclick: move |_| {
                            if is_correct {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level25();
                                let new_order = new_st.order.clone();
                                state.set(new_st);
                                order.set(new_order);
                                drag_idx.set(None);
                                wrong.set(false);
                            } else {
                                wrong.set(true);
                                spawn(async move {
                                    gloo_timers::future::TimeoutFuture::new(600).await;
                                    wrong.set(false);
                                });
                            }
                        },
                        "Submit"
                    }
                }

                // Drag overlay — at viewport level to capture all mouse movement
                if cur_drag.is_some() {
                    div {
                        style: "position: absolute; inset: 0; z-index: 100; cursor: grabbing;",
                        onmousemove: move |e: Event<MouseData>| {
                            if let Some(mut di) = drag_idx() {
                                let delta = e.page_coordinates().y as f32 - drag_start_page_y();
                                let max_y = item_y(item_count - 1);
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
                                    }
                                }
                                // Check swap with item below
                                if di < item_count - 1 {
                                    let below_center = item_y(di + 1) + ITEM_H / 2.0;
                                    if dragged_center > below_center {
                                        order.write().swap(di, di + 1);
                                        drag_idx.set(Some(di + 1));
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
            }

            super::GroundTruth {
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: card_w,
                target_h: card_h_est,
                tree: Some(tree.clone()),
            }
        }
    }
}
