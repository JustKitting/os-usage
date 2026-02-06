use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg, ordinal};

const GROUP_NAMES: &[&str] = &[
    "Size", "Color", "Plan", "Priority", "Shipping",
    "Format", "Language", "Theme", "Region", "Category",
    "Role", "Status", "Frequency", "Rating", "Type",
];

const OPTION_POOLS: &[&[&str]] = &[
    &["Small", "Medium", "Large", "Extra Large"],
    &["Red", "Blue", "Green", "Yellow", "Purple", "Orange"],
    &["Free", "Basic", "Pro", "Enterprise"],
    &["Low", "Medium", "High", "Critical"],
    &["Standard", "Express", "Overnight", "Economy"],
    &["PDF", "CSV", "JSON", "XML", "HTML"],
    &["English", "Spanish", "French", "German", "Japanese"],
    &["Light", "Dark", "System", "Custom"],
    &["North", "South", "East", "West", "Central"],
    &["General", "Science", "Sports", "Tech", "Art"],
    &["Admin", "Editor", "Viewer", "Guest"],
    &["Active", "Inactive", "Pending", "Archived"],
    &["Daily", "Weekly", "Monthly", "Yearly"],
    &["Poor", "Fair", "Good", "Excellent"],
    &["Personal", "Business", "Education", "Government"],
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

#[derive(Clone)]
struct RadioGroup {
    name: String,
    options: Vec<String>,
    accent: String,
}

struct Level17State {
    groups: Vec<RadioGroup>,
    target_group: usize,
    target_option: usize,
    mode: u8, // 0=by group+option name, 1=by ordinal group + option name, 2=by group + ordinal option
    x: f32,
    y: f32,
    card_w: f32,
}

fn random_level17() -> Level17State {
    let mut rng = fresh_rng();
    let group_count = rng.random_range(1..=4usize);

    let mut group_pool: Vec<usize> = (0..GROUP_NAMES.len()).collect();
    let mut color_pool: Vec<usize> = (0..ACCENT_COLORS.len()).collect();
    let mut groups = Vec::new();

    for _ in 0..group_count {
        let gi = rng.random_range(0..group_pool.len());
        let idx = group_pool.remove(gi);
        let name = GROUP_NAMES[idx].to_string();

        let all_opts = OPTION_POOLS[idx];
        let opt_count = rng.random_range(3..=all_opts.len().min(5));
        let mut opt_pool: Vec<usize> = (0..all_opts.len()).collect();
        let mut options = Vec::new();
        for _ in 0..opt_count {
            let oi = rng.random_range(0..opt_pool.len());
            options.push(all_opts[opt_pool.remove(oi)].to_string());
        }

        let ci = rng.random_range(0..color_pool.len());
        let accent = ACCENT_COLORS[color_pool.remove(ci)].to_string();

        groups.push(RadioGroup { name, options, accent });
    }

    let target_group = rng.random_range(0..group_count);
    let target_option = rng.random_range(0..groups[target_group].options.len());

    let mode = if group_count == 1 {
        // Single group: just name the option
        if rng.random_bool(0.5) { 0 } else { 2 }
    } else {
        rng.random_range(0..3u8)
    };

    let card_w = rng.random_range(280.0..=420.0f32);
    let group_h = 40.0; // label + spacing
    let opt_h = 32.0;
    let total_opts: usize = groups.iter().map(|g| g.options.len()).sum();
    let card_h = group_count as f32 * group_h + total_opts as f32 * opt_h + 100.0;

    let margin = 50.0;
    let (x, y) = super::safe_position(&mut rng, card_w, card_h, margin);

    Level17State { groups, target_group, target_option, mode, x, y, card_w }
}

#[component]
pub fn Level17() -> Element {
    let mut state = use_signal(|| random_level17());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_sel: Vec<Option<usize>> = {
        let s = state.read();
        vec![None; s.groups.len()]
    };
    let mut selections = use_signal(move || initial_sel);
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let groups: Vec<RadioGroup> = st.groups.clone();
    let target_group = st.target_group;
    let target_option = st.target_option;
    let mode = st.mode;
    let card_x = st.x;
    let card_y = st.y;
    let card_w = st.card_w;
    drop(st);

    let group_count = groups.len();
    let is_wrong = wrong();
    let viewport_style = super::viewport_style(&bg(), false);
    let sels: Vec<Option<usize>> = selections.read().clone();

    let target_group_name = groups[target_group].name.clone();
    let target_option_name = groups[target_group].options[target_option].clone();

    let instruction = match mode {
        1 => {
            let g_ord = ordinal(target_group + 1);
            format!("In the {} group, select \"{}\"", g_ord, target_option_name)
        }
        2 => {
            let o_ord = ordinal(target_option + 1);
            format!("In \"{}\", select the {} option", target_group_name, o_ord)
        }
        _ => {
            if group_count == 1 {
                format!("Select \"{}\"", target_option_name)
            } else {
                format!("In \"{}\", select \"{}\"", target_group_name, target_option_name)
            }
        }
    };

    let group_h = 40.0;
    let opt_h = 32.0;
    let total_opts: usize = groups.iter().map(|g| g.options.len()).sum();
    let card_h = group_count as f32 * group_h + total_opts as f32 * opt_h + 100.0;
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px; box-sizing: border-box;",
        card_x, card_y, card_w
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    // Ground truth via UINode tree
    let radio_nodes: Vec<_> = groups.iter().enumerate().map(|(gi, g)| {
        let target_opt_idx = if gi == target_group { target_option } else { 0 };
        let mut node = ui_node::radio_group(
            &g.name,
            Rect::new(card_x + 16.0, card_y + 40.0 + gi as f32 * (group_h + g.options.len() as f32 * opt_h), card_w - 32.0, group_h + g.options.len() as f32 * opt_h),
            g.options.clone(),
            target_opt_idx,
        );
        if gi != target_group {
            node.visual_mut().is_target = false;
        }
        node
    }).collect();
    let tree = ui_node::form(
        Rect::new(card_x, card_y, card_w, card_h),
        "Submit",
        radio_nodes,
    );
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
                    "Level 5"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Radio"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            // Canvas
            div {
                id: "viewport",
                style: "{viewport_style}",

                div {
                    style: "{card_style}",

                    p {
                        style: "margin: 0 0 16px 0; font-size: 14px; color: #374151; font-weight: 500;",
                        "{instruction}"
                    }

                    for gi in 0..group_count {
                        {
                            let g = groups[gi].clone();
                            let opt_count = g.options.len();
                            let selected = sels.get(gi).copied().flatten();
                            let is_last = gi == group_count - 1;
                            let mb = if is_last { "0" } else { "16px" };

                            rsx! {
                                div {
                                    style: "margin-bottom: {mb};",

                                    // Group label
                                    div {
                                        style: "font-size: 13px; font-weight: 600; color: #374151; margin-bottom: 8px;",
                                        "{g.name}"
                                    }

                                    // Options
                                    for oi in 0..opt_count {
                                        {
                                            let opt_name = g.options[oi].clone();
                                            let is_sel = selected == Some(oi);
                                            let outer_border = if is_sel { g.accent.clone() } else { "#d1d5db".to_string() };
                                            let inner_bg = if is_sel { g.accent.clone() } else { "transparent".to_string() };
                                            let text_color = if is_sel { "#111827" } else { "#4b5563" };
                                            let is_target = gi == target_group && oi == target_option;

                                            rsx! {
                                                div {
                                                    class: if is_target { "target" } else { "" },
                                                    "data-label": "{opt_name}",
                                                    style: "display: flex; align-items: center; gap: 8px; padding: 6px 8px; cursor: pointer; border-radius: 4px; transition: background 0.1s;",
                                                    tabindex: "-1",
                                                    onclick: move |_| {
                                                        let mut s = selections.write();
                                                        if let Some(v) = s.get_mut(gi) {
                                                            *v = Some(oi);
                                                        }
                                                    },

                                                    // Radio circle
                                                    div {
                                                        style: "width: 18px; height: 18px; border-radius: 50%; border: 2px solid {outer_border}; display: flex; align-items: center; justify-content: center; flex-shrink: 0; transition: border-color 0.15s;",
                                                        div {
                                                            style: "width: 10px; height: 10px; border-radius: 50%; background: {inner_bg}; transition: background 0.15s;",
                                                        }
                                                    }

                                                    // Label
                                                    span {
                                                        style: "font-size: 13px; color: {text_color}; user-select: none;",
                                                        "{opt_name}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Submit
                    button {
                        class: "target",
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s; margin-top: 16px;",
                        tabindex: "-1",
                        onclick: move |_| {
                            let sel = selections.read().get(target_group).copied().flatten();
                            if sel == Some(target_option) {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level17();
                                let count = new_st.groups.len();
                                state.set(new_st);
                                selections.set(vec![None; count]);
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
            }

            super::GroundTruth {
                description: String::new(),
                target_x: card_x,
                target_y: card_y,
                target_w: card_w,
                target_h: card_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
