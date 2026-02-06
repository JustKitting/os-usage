use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg};

const FILE_POOL: &[(&str, &str, &str)] = &[
    ("report", "pdf", "#ef4444"),
    ("photo", "jpg", "#3b82f6"),
    ("data", "csv", "#22c55e"),
    ("notes", "txt", "#6b7280"),
    ("invoice", "pdf", "#ef4444"),
    ("backup", "zip", "#f59e0b"),
    ("image", "png", "#3b82f6"),
    ("document", "docx", "#3b82f6"),
    ("budget", "xlsx", "#22c55e"),
    ("slides", "pptx", "#f97316"),
    ("readme", "md", "#6b7280"),
    ("config", "json", "#f59e0b"),
    ("export", "sql", "#f97316"),
    ("archive", "tar", "#f59e0b"),
    ("clip", "mp4", "#8b5cf6"),
    ("track", "mp3", "#8b5cf6"),
    ("script", "py", "#14b8a6"),
    ("styles", "css", "#14b8a6"),
    ("page", "html", "#ec4899"),
    ("server", "log", "#6b7280"),
];

const FILE_W: f32 = 80.0;
const FILE_H: f32 = 96.0;

#[derive(Clone)]
struct FileIcon {
    name: String,
    ext: String,
    color: String,
    orig_x: f32,
    orig_y: f32,
}

struct Level15State {
    files: Vec<FileIcon>,
    target: usize,
    drop_x: f32,
    drop_y: f32,
    drop_w: f32,
    drop_h: f32,
}

fn random_level15() -> Level15State {
    let mut rng = fresh_rng();
    let file_count = rng.random_range(2..=5usize);

    let drop_w = rng.random_range(180.0..=240.0f32);
    let drop_h = rng.random_range(140.0..=180.0f32);

    let margin = 50.0;
    let gap = 30.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();

    // Sizes: drop zone first, then file icons
    let mut sizes: Vec<(f32, f32)> = vec![(drop_w, drop_h)];
    for _ in 0..file_count {
        sizes.push((FILE_W, FILE_H));
    }

    // Place items without overlap
    let mut rects: Vec<(f32, f32, f32, f32)> = Vec::new();
    let mut all_pos: Vec<(f32, f32)> = Vec::new();
    for &(w, h) in &sizes {
        let mut pos = (margin, margin);
        for _ in 0..300 {
            let (x, y) = super::safe_position_in(&mut rng, w, h, margin, vp_w * 1.3, vp_h * 1.3);
            let ok = rects.iter().all(|&(rx, ry, rw, rh)| {
                x >= rx + rw + gap || x + w + gap <= rx || y >= ry + rh + gap || y + h + gap <= ry
            });
            if ok {
                pos = (x, y);
                break;
            }
        }
        rects.push((pos.0, pos.1, w, h));
        all_pos.push(pos);
    }

    let (drop_x, drop_y) = all_pos[0];

    let mut pool: Vec<usize> = (0..FILE_POOL.len()).collect();
    let mut files = Vec::new();
    for i in 0..file_count {
        let pi = rng.random_range(0..pool.len());
        let (name, ext, color) = FILE_POOL[pool.remove(pi)];
        let (x, y) = all_pos[i + 1];
        files.push(FileIcon {
            name: name.to_string(),
            ext: ext.to_string(),
            color: color.to_string(),
            orig_x: x,
            orig_y: y,
        });
    }

    let target = rng.random_range(0..file_count);

    Level15State { files, target, drop_x, drop_y, drop_w, drop_h }
}

fn snap_back(state: &Signal<Level15State>, file_pos: &mut Signal<Vec<(f32, f32)>>, fi: usize) {
    let st = state.read();
    if let Some(f) = st.files.get(fi) {
        let orig = (f.orig_x, f.orig_y);
        drop(st);
        let mut p = file_pos.write();
        if let Some(pos) = p.get_mut(fi) {
            *pos = orig;
        }
    }
}

#[component]
pub fn Level15() -> Element {
    let mut state = use_signal(|| random_level15());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_pos = {
        let s = state.read();
        s.files.iter().map(|f| (f.orig_x, f.orig_y)).collect::<Vec<_>>()
    };
    let mut file_pos = use_signal(move || initial_pos);
    let mut drag_idx = use_signal(|| Option::<usize>::None);
    let mut drag_off = use_signal(|| (0.0f32, 0.0f32));
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let files: Vec<FileIcon> = st.files.clone();
    let target = st.target;
    let drop_x = st.drop_x;
    let drop_y = st.drop_y;
    let drop_w = st.drop_w;
    let drop_h = st.drop_h;
    drop(st);

    let file_count = files.len();
    let target_name = format!("{}.{}", files[target].name, files[target].ext);
    let is_wrong = wrong();
    let cur_drag = drag_idx();
    let positions: Vec<(f32, f32)> = file_pos.read().clone();

    // Check if dragged file is over drop zone
    let drag_over = if let Some(di) = cur_drag {
        let (fx, fy) = positions.get(di).copied().unwrap_or((0.0, 0.0));
        let cx = fx + FILE_W / 2.0;
        let cy = fy + FILE_H / 2.0;
        cx >= drop_x && cx <= drop_x + drop_w && cy >= drop_y && cy <= drop_y + drop_h
    } else {
        false
    };

    let dz_border = if is_wrong { "#ef4444" } else if drag_over { "#4f46e5" } else { "#d1d5db" };
    let dz_bg = if is_wrong { "#fef2f2" } else if drag_over { "#eef2ff" } else { "white" };
    let dz_arrow = if is_wrong { "#ef4444" } else if drag_over { "#4f46e5" } else { "#9ca3af" };
    let viewport_style = format!("{} user-select: none;", super::viewport_style(&bg(), true));

    // Ground truth via UINode tree
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let tree = ui_node::card(
        Rect::new(0.0, 0.0, vp_w, vp_h),
        vec![
            ui_node::drag_source(&target_name, Rect::new(files[target].orig_x, files[target].orig_y, FILE_W, FILE_H)),
            ui_node::drop_zone("Upload Zone", Rect::new(drop_x, drop_y, drop_w, drop_h)),
        ],
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
                    "Level 21"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Drag & Drop"
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

                // Instruction banner
                div {
                    style: "position: absolute; top: 16px; left: 50%; transform: translateX(-50%); background: rgba(0,0,0,0.8); border-radius: 8px; padding: 8px 16px; z-index: 50; pointer-events: none; white-space: nowrap;",
                    p {
                        style: "margin: 0; font-size: 14px; color: #e5e7eb; font-weight: 500;",
                        "Drag "
                        span { style: "font-weight: 700; color: white; font-family: monospace;", "\"{target_name}\"" }
                        " to the upload area"
                    }
                }

                // Drop zone
                div {
                    class: "target",
                    "data-label": "Upload Zone",
                    style: "position: absolute; left: {drop_x}px; top: {drop_y}px; width: {drop_w}px; height: {drop_h}px; background: {dz_bg}; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); display: flex; align-items: center; justify-content: center; box-sizing: border-box; transition: background 0.15s;",

                    div {
                        style: "border: 2px dashed {dz_border}; border-radius: 8px; padding: 20px 16px; text-align: center; width: 100%; box-sizing: border-box; transition: border-color 0.15s;",

                        div {
                            style: "font-size: 28px; color: {dz_arrow}; margin-bottom: 6px; transition: color 0.15s;",
                            "\u{2191}"
                        }
                        div {
                            style: "font-size: 14px; color: #374151; font-weight: 600;",
                            "Upload File"
                        }
                        div {
                            style: "font-size: 11px; color: #9ca3af; margin-top: 4px;",
                            if drag_over { "Release to upload" } else { "Drop file here" }
                        }
                    }
                }

                // File icons
                for fi in 0..file_count {
                    {
                        let f = files[fi].clone();
                        let (fx, fy) = positions.get(fi).copied().unwrap_or((0.0, 0.0));
                        let is_me = cur_drag == Some(fi);
                        let z = if is_me { "200" } else { "10" };
                        let pe = if is_me { "none" } else { "auto" };
                        let opacity = if is_me { "0.85" } else { "1" };
                        let shadow = if is_me { "0 8px 24px rgba(0,0,0,0.5)" } else { "0 2px 8px rgba(0,0,0,0.3)" };
                        let full_name = format!("{}.{}", f.name, f.ext);
                        let ext_upper = f.ext.to_uppercase();

                        rsx! {
                            div {
                                class: if fi == target { "target" } else { "" },
                                "data-label": "{full_name}",
                                style: "position: absolute; left: {fx}px; top: {fy}px; z-index: {z}; pointer-events: {pe}; cursor: grab; opacity: {opacity}; display: flex; flex-direction: column; align-items: center; user-select: none;",
                                onmousedown: move |e: Event<MouseData>| {
                                    e.prevent_default();
                                    wrong.set(false);
                                    drag_idx.set(Some(fi));
                                    let coords = e.element_coordinates();
                                    drag_off.set((coords.x as f32, coords.y as f32));
                                },

                                // Document icon
                                div {
                                    style: "width: 56px; height: 68px; background: white; border-radius: 4px; border: 1px solid #d1d5db; position: relative; box-shadow: {shadow}; overflow: hidden;",

                                    // Fold corner
                                    div {
                                        style: "position: absolute; top: 0; right: 0; width: 12px; height: 12px; background: #e5e7eb; border-bottom-left-radius: 2px;",
                                    }

                                    // Text lines
                                    div {
                                        style: "position: absolute; top: 14px; left: 7px; right: 7px; display: flex; flex-direction: column; gap: 3px;",
                                        div { style: "height: 2px; background: #e5e7eb; border-radius: 1px;" }
                                        div { style: "height: 2px; background: #e5e7eb; border-radius: 1px; width: 75%;" }
                                        div { style: "height: 2px; background: #e5e7eb; border-radius: 1px; width: 55%;" }
                                    }

                                    // Extension badge
                                    div {
                                        style: "position: absolute; bottom: 6px; left: 50%; transform: translateX(-50%); background: {f.color}; color: white; padding: 1px 5px; border-radius: 2px; font-size: 9px; font-weight: 700; font-family: monospace; white-space: nowrap;",
                                        "{ext_upper}"
                                    }
                                }

                                // Filename
                                div {
                                    style: "font-size: 11px; color: #e5e7eb; margin-top: 4px; max-width: 80px; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-shadow: 0 1px 3px rgba(0,0,0,0.8);",
                                    "{full_name}"
                                }
                            }
                        }
                    }
                }

                // Drag overlay — captures mouse during drag
                if cur_drag.is_some() {
                    div {
                        style: "position: absolute; inset: 0; z-index: 100; cursor: grabbing;",
                        onmousemove: move |e: Event<MouseData>| {
                            if let Some(fi) = drag_idx() {
                                let coords = e.element_coordinates();
                                let (ox, oy) = drag_off();
                                let (vp_w, vp_h) = crate::primitives::viewport_size();
                                let nx = (coords.x as f32 - ox).clamp(0.0, vp_w - FILE_W);
                                let ny = (coords.y as f32 - oy).clamp(0.0, vp_h - FILE_H);
                                let mut p = file_pos.write();
                                if let Some(pos) = p.get_mut(fi) {
                                    *pos = (nx, ny);
                                }
                            }
                        },
                        onmouseup: move |_| {
                            if let Some(fi) = drag_idx() {
                                let cur = file_pos.read().get(fi).copied().unwrap_or((0.0, 0.0));
                                let cx = cur.0 + FILE_W / 2.0;
                                let cy = cur.1 + FILE_H / 2.0;
                                let in_zone = cx >= drop_x && cx <= drop_x + drop_w
                                    && cy >= drop_y && cy <= drop_y + drop_h;

                                if in_zone && fi == target {
                                    score.set(score() + 1);
                                    bg.set(random_canvas_bg());
                                    let new_st = random_level15();
                                    let new_pos: Vec<(f32, f32)> = new_st.files.iter().map(|f| (f.orig_x, f.orig_y)).collect();
                                    state.set(new_st);
                                    file_pos.set(new_pos);
                                    wrong.set(false);
                                } else {
                                    if in_zone {
                                        wrong.set(true);
                                        spawn(async move {
                                            gloo_timers::future::TimeoutFuture::new(600).await;
                                            wrong.set(false);
                                        });
                                    }
                                    snap_back(&state, &mut file_pos, fi);
                                }
                            }
                            drag_idx.set(None);
                        },
                        onmouseleave: move |_| {
                            if let Some(fi) = drag_idx() {
                                snap_back(&state, &mut file_pos, fi);
                            }
                            drag_idx.set(None);
                        },
                    }
                }
            }

            super::GroundTruth {
                description: String::new(),
                target_x: drop_x,
                target_y: drop_y,
                target_w: drop_w,
                target_h: drop_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
