use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg, ordinal};

const RATING_LABELS: &[&str] = &[
    "Quality", "Service", "Value", "Cleanliness", "Comfort",
    "Location", "Food", "Staff", "Atmosphere", "Price",
    "Speed", "Design", "Usability", "Reliability", "Overall",
];

const STAR_COLORS: &[&str] = &[
    "#f59e0b", "#eab308", "#f97316", "#ef4444", "#ec4899",
    "#8b5cf6", "#3b82f6", "#06b6d4", "#10b981", "#84cc16",
];

#[derive(Clone)]
struct RatingInfo {
    label: String,
    max_stars: usize,
    target_val: usize,
    start_val: usize,
    color: String,
    star_size: f32,
}

struct Level19State {
    ratings: Vec<RatingInfo>,
    target_rating: usize,
    mode: u8,
    x: f32,
    y: f32,
    card_w: f32,
}

fn random_level19() -> Level19State {
    let mut rng = fresh_rng();
    let count = rng.random_range(1..=3usize);

    let mut label_pool: Vec<usize> = (0..RATING_LABELS.len()).collect();
    let mut color_pool: Vec<usize> = (0..STAR_COLORS.len()).collect();
    let mut ratings = Vec::new();

    for _ in 0..count {
        let li = rng.random_range(0..label_pool.len());
        let label = RATING_LABELS[label_pool.remove(li)].to_string();

        let ci = rng.random_range(0..color_pool.len());
        let color = STAR_COLORS[color_pool.remove(ci)].to_string();

        let max_stars = if rng.random_bool(0.7) { 5 } else { 10 };
        let target_val = rng.random_range(1..=max_stars);
        let start_val = if rng.random_bool(0.5) {
            0
        } else {
            let mut sv = target_val;
            while sv == target_val {
                sv = rng.random_range(0..=max_stars);
            }
            sv
        };

        let star_size = if max_stars == 10 {
            rng.random_range(18.0..=24.0f32)
        } else {
            rng.random_range(24.0..=36.0f32)
        };

        ratings.push(RatingInfo { label, max_stars, target_val, start_val, color, star_size });
    }

    let target_rating = rng.random_range(0..count);
    let mode = if count == 1 { 0 } else { rng.random_range(0..2u8) };

    let card_w = rng.random_range(280.0..=420.0f32);
    let row_h = 60.0;
    let card_h = count as f32 * row_h + 120.0;
    let margin = 50.0;
    let (x, y) = super::safe_position(&mut rng, card_w, card_h, margin);

    Level19State { ratings, target_rating, mode, x, y, card_w }
}

#[component]
pub fn Level19() -> Element {
    let mut state = use_signal(|| random_level19());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_vals: Vec<usize> = state.read().ratings.iter().map(|r| r.start_val).collect();
    let mut values = use_signal(move || initial_vals);
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let ratings: Vec<RatingInfo> = st.ratings.clone();
    let target_rating = st.target_rating;
    let mode = st.mode;
    let card_x = st.x;
    let card_y = st.y;
    let card_w = st.card_w;
    drop(st);

    let rating_count = ratings.len();
    let is_wrong = wrong();
    let cur_vals: Vec<usize> = values.read().clone();

    let target_label = ratings[target_rating].label.clone();
    let target_val = ratings[target_rating].target_val;
    let target_max = ratings[target_rating].max_stars;

    let instruction = match mode {
        1 => {
            let ord = ordinal(target_rating + 1);
            format!("Rate the {} one {} out of {}", ord, target_val, target_max)
        }
        _ => {
            if rating_count == 1 {
                format!("Rate {} out of {}", target_val, target_max)
            } else {
                format!("Rate \"{}\" {} out of {}", target_label, target_val, target_max)
            }
        }
    };

    let row_h = 60.0;
    let card_h = rating_count as f32 * row_h + 120.0;
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px; box-sizing: border-box;",
        card_x, card_y, card_w
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    // Ground truth via UINode tree
    let star_nodes: Vec<_> = ratings.iter().enumerate().map(|(i, r)| {
        let cv = cur_vals.get(i).copied().unwrap_or(r.start_val);
        let row_y = 40.0 + i as f32 * row_h;
        let mut node = ui_node::star_rating(
            &r.label,
            Rect::new(card_x + 16.0, card_y + row_y, card_w - 32.0, row_h),
            cv,
            r.target_val,
            r.max_stars,
        );
        if i != target_rating {
            node.visual_mut().is_target = false;
        }
        node
    }).collect();
    let tree = ui_node::form(
        Rect::new(card_x, card_y, card_w, card_h),
        "Submit",
        star_nodes,
    );
    let description = String::new();
    let viewport_style = super::viewport_style(&bg(), false);

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
                    "Level 8"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Star Rating"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            div {
                id: "viewport",
                style: "{viewport_style}",

                div {
                    style: "{card_style}",

                    p {
                        style: "margin: 0 0 16px 0; font-size: 14px; color: #374151; font-weight: 500;",
                        "{instruction}"
                    }

                    for ri in 0..rating_count {
                        {
                            let r = ratings[ri].clone();
                            let val = cur_vals.get(ri).copied().unwrap_or(r.start_val);
                            let is_last = ri == rating_count - 1;
                            let mb = if is_last { "0" } else { "16px" };

                            rsx! {
                                div {
                                    style: "margin-bottom: {mb};",

                                    div {
                                        style: "font-size: 13px; font-weight: 500; color: #374151; margin-bottom: 6px;",
                                        "{r.label}"
                                    }

                                    div {
                                        style: "display: flex; gap: 4px; align-items: center;",

                                        for si in 0..r.max_stars {
                                            {
                                                let filled = si < val;
                                                let color = if filled { r.color.clone() } else { "#d1d5db".to_string() };
                                                let star_char = if filled { "\u{2605}" } else { "\u{2606}" };
                                                let size = r.star_size;
                                                let si_plus_1 = si + 1;

                                                rsx! {
                                                    span {
                                                        class: if ri == target_rating && si + 1 == target_val { "target" } else { "" },
                                                        "data-label": "star {si_plus_1} of {r.label}",
                                                        style: "font-size: {size}px; color: {color}; cursor: pointer; line-height: 1; user-select: none; transition: color 0.1s;",
                                                        tabindex: "-1",
                                                        onclick: move |_| {
                                                            let mut v = values.write();
                                                            if let Some(val) = v.get_mut(ri) {
                                                                if *val == si + 1 {
                                                                    *val = 0;
                                                                } else {
                                                                    *val = si + 1;
                                                                }
                                                            }
                                                        },
                                                        "{star_char}"
                                                    }
                                                }
                                            }
                                        }

                                        span {
                                            style: "font-size: 12px; color: #9ca3af; margin-left: 8px; font-family: monospace;",
                                            "{val}/{r.max_stars}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    button {
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s; margin-top: 16px;",
                        tabindex: "-1",
                        onclick: move |_| {
                            let v = values.read().get(target_rating).copied().unwrap_or(0);
                            if v == target_val {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level19();
                                let new_vals: Vec<usize> = new_st.ratings.iter().map(|r| r.start_val).collect();
                                state.set(new_st);
                                values.set(new_vals);
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
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: card_w,
                target_h: card_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
