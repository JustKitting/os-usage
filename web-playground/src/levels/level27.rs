use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, UINode, Visual, Rect, ToastState};
use super::{fresh_rng, random_canvas_bg};

#[derive(Clone, Copy, PartialEq)]
enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastKind {
    fn icon(&self) -> &'static str {
        match self {
            ToastKind::Success => "\u{2714}",
            ToastKind::Error => "\u{2716}",
            ToastKind::Warning => "\u{26A0}",
            ToastKind::Info => "\u{2139}",
        }
    }
    fn color(&self) -> &'static str {
        match self {
            ToastKind::Success => "#22c55e",
            ToastKind::Error => "#ef4444",
            ToastKind::Warning => "#f59e0b",
            ToastKind::Info => "#3b82f6",
        }
    }
    fn label(&self) -> &'static str {
        match self {
            ToastKind::Success => "success",
            ToastKind::Error => "error",
            ToastKind::Warning => "warning",
            ToastKind::Info => "info",
        }
    }
}

const ALL_KINDS: &[ToastKind] = &[ToastKind::Success, ToastKind::Error, ToastKind::Warning, ToastKind::Info];

const MESSAGES: &[(&str, ToastKind)] = &[
    ("File uploaded successfully", ToastKind::Success),
    ("Payment processed", ToastKind::Success),
    ("Profile updated", ToastKind::Success),
    ("Message sent", ToastKind::Success),
    ("Settings saved", ToastKind::Success),
    ("Export complete", ToastKind::Success),
    ("Connection failed", ToastKind::Error),
    ("Invalid credentials", ToastKind::Error),
    ("Upload failed — file too large", ToastKind::Error),
    ("Server error — try again later", ToastKind::Error),
    ("Permission denied", ToastKind::Error),
    ("Session expired", ToastKind::Error),
    ("Storage almost full (90%)", ToastKind::Warning),
    ("Password expires in 3 days", ToastKind::Warning),
    ("Unsaved changes detected", ToastKind::Warning),
    ("API rate limit approaching", ToastKind::Warning),
    ("Slow network detected", ToastKind::Warning),
    ("New version available", ToastKind::Info),
    ("Maintenance scheduled for tonight", ToastKind::Info),
    ("2 new comments on your post", ToastKind::Info),
    ("Team member joined the project", ToastKind::Info),
    ("Backup completed at 3:00 AM", ToastKind::Info),
    ("Your trial ends in 5 days", ToastKind::Info),
];

#[derive(Clone)]
struct ToastInfo {
    message: String,
    kind: ToastKind,
    y: f32,
}

struct Level27State {
    toasts: Vec<ToastInfo>,
    target_idx: usize,
    style: u8,
    stack_x: f32,
    stack_start_y: f32,
    toast_w: f32,
}

fn random_level27() -> Level27State {
    let mut rng = fresh_rng();

    // Pick 3-6 toasts
    let count = rng.random_range(3..=6usize);
    let mut msg_pool: Vec<usize> = (0..MESSAGES.len()).collect();
    let mut toasts = Vec::new();

    let toast_w = rng.random_range(300.0..=400.0f32);
    let toast_h = 60.0f32;
    let gap = rng.random_range(8.0..=16.0f32);
    let stack_h = count as f32 * (toast_h + gap);
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (stack_x, stack_start_y) = super::safe_position_in(&mut rng, toast_w, stack_h, 60.0, vp_w * 1.3, vp_h * 1.3);

    for i in 0..count {
        let mi = rng.random_range(0..msg_pool.len());
        let msg_idx = msg_pool.remove(mi);
        let (message, kind) = MESSAGES[msg_idx];
        // Y relative to the stack container, not the viewport
        let y = i as f32 * (toast_h + gap);
        toasts.push(ToastInfo { message: message.to_string(), kind, y });
    }

    let target_idx = rng.random_range(0..count);
    let style = rng.random_range(0..3u8);

    Level27State { toasts, target_idx, style, stack_x, stack_start_y, toast_w }
}

#[component]
pub fn Level27() -> Element {
    let mut state = use_signal(|| random_level27());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong = use_signal(|| false);
    let initial_visible: Vec<bool> = vec![true; state.read().toasts.len()];
    let mut visible = use_signal(move || initial_visible);

    let st = state.read();
    let toasts: Vec<ToastInfo> = st.toasts.clone();
    let target_idx = st.target_idx;
    let style = st.style;
    let stack_x = st.stack_x;
    let stack_start_y = st.stack_start_y;
    let toast_w = st.toast_w;
    drop(st);

    let toast_count = toasts.len();
    let is_wrong = wrong();
    let cur_visible: Vec<bool> = visible.read().clone();

    let target_toast = &toasts[target_idx];
    let target_msg = target_toast.message.clone();
    let _target_kind = target_toast.kind;
    // Target Y in viewport coords for ground truth
    let target_y = stack_start_y + target_toast.y;
    let instruction = format!("Dismiss the \"{}\" notification", target_msg);

    let border_radius = match style { 0 => "14px", 1 => "4px", _ => "8px" };

    // Ground truth — build UINode tree (viewport-absolute coords)
    let stack_h_est = toast_count as f32 * 72.0;
    let card_rect = Rect::new(stack_x, stack_start_y, toast_w, stack_h_est);
    let children: Vec<UINode> = toasts.iter().enumerate().map(|(i, t)| {
        let toast_rect = Rect::new(stack_x, stack_start_y + t.y, toast_w, 60.0);
        let kind_label = t.kind.label();
        if i == target_idx {
            ui_node::toast(&t.message, toast_rect, kind_label, &t.message)
        } else {
            UINode::Toast(
                Visual::new(&t.message, toast_rect),
                ToastState { kind: kind_label.to_string(), message: t.message.clone() },
            )
        }
    }).collect();
    let tree = ui_node::card(card_rect, children);
    let description = String::new();
    let viewport_style = super::viewport_style(&bg(), true);

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
                    "Level 27"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Toast Dismiss"
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
                        style: "display: inline-block; background: rgba(0,0,0,0.7); padding: 8px 16px; border-radius: 8px; color: white; font-size: 14px; font-weight: 500; max-width: 600px;",
                        "{instruction}"
                    }
                }

                // Toast stack — single container so fixup centers them as a group
                div {
                    style: "position: absolute; left: {stack_x}px; top: {stack_start_y}px; width: {toast_w}px; height: {stack_h_est}px;",

                    for ti in 0..toast_count {
                        {
                            let toast = toasts[ti].clone();
                            let is_vis = cur_visible.get(ti).copied().unwrap_or(false);

                            if !is_vis {
                                return rsx! {};
                            }

                            let kind_color = toast.kind.color();
                            let kind_icon = toast.kind.icon();
                            let wrong_flash = is_wrong && ti == target_idx;

                            let toast_bg = if wrong_flash { "#fef2f2" } else { "white" };
                            let left_border = if wrong_flash {
                                "4px solid #ef4444".to_string()
                            } else {
                                format!("4px solid {}", kind_color)
                            };

                            let shadow = match style {
                                0 => "0 4px 20px rgba(0,0,0,0.15)",
                                1 => "0 1px 6px rgba(0,0,0,0.12)",
                                _ => "0 2px 12px rgba(0,0,0,0.14)",
                            };

                            // Positions are relative to the stack container
                            let toast_style = format!(
                                "position: absolute; left: 0; top: {}px; width: 100%; \
                                 background: {}; border-radius: {}; border-left: {}; \
                                 box-shadow: {}; padding: 14px 16px; \
                                 display: flex; align-items: center; gap: 12px; \
                                 font-family: system-ui, sans-serif; box-sizing: border-box; \
                                 transition: opacity 0.2s;",
                                toast.y, toast_bg, border_radius,
                                left_border, shadow
                            );

                            let icon_bg = format!("{}1a", kind_color);

                            rsx! {
                                div {
                                    style: "{toast_style}",

                                    div {
                                        style: "width: 28px; height: 28px; border-radius: 50%; background: {icon_bg}; display: flex; align-items: center; justify-content: center; flex-shrink: 0; font-size: 13px; color: {kind_color};",
                                        "{kind_icon}"
                                    }

                                    div {
                                        style: "flex: 1; font-size: 13px; color: #374151; line-height: 1.4;",
                                        "{toast.message}"
                                    }

                                    button {
                                        class: if ti == target_idx { "target" } else { "" },
                                        "data-label": "dismiss: {toast.message}",
                                        style: "width: 24px; height: 24px; border: none; background: transparent; border-radius: 4px; font-size: 14px; color: #9ca3af; cursor: pointer; display: flex; align-items: center; justify-content: center; flex-shrink: 0; font-family: system-ui, sans-serif; transition: background 0.1s;",
                                        tabindex: "-1",
                                        onclick: move |_| {
                                            if ti == target_idx {
                                                let mut v = visible.write();
                                                if let Some(val) = v.get_mut(ti) {
                                                    *val = false;
                                                }
                                                drop(v);
                                                spawn(async move {
                                                    gloo_timers::future::TimeoutFuture::new(300).await;
                                                    score.set(score() + 1);
                                                    bg.set(random_canvas_bg());
                                                    let new_st = random_level27();
                                                    let new_vis = vec![true; new_st.toasts.len()];
                                                    state.set(new_st);
                                                    visible.set(new_vis);
                                                    wrong.set(false);
                                                });
                                            } else {
                                                wrong.set(true);
                                                spawn(async move {
                                                    gloo_timers::future::TimeoutFuture::new(600).await;
                                                    wrong.set(false);
                                                });
                                            }
                                        },
                                        "\u{2715}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            super::GroundTruth {
                description: description,
                target_x: stack_x,
                target_y: target_y,
                target_w: toast_w,
                target_h: 60.0,
                tree: Some(tree.clone()),
            }
        }
    }
}
