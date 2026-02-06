use dioxus::prelude::*;
use crate::ui_node::{UINode, ViewportTransform};

/// Strip HTML tags to get plain text
pub fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Bounding box: [x, y, width, height]
fn get_window_bbox() -> [i32; 4] {
    if let Some(window) = web_sys::window() {
        let w = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0) as i32;
        let h = window.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0) as i32;
        [0, 0, w, h]
    } else {
        [0, 0, 0, 0]
    }
}

/// Bounding box of the #viewport element: [x, y, width, height] (visual/screen coords)
fn get_viewport_bbox() -> [f64; 4] {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Some(element) = document.get_element_by_id("viewport") {
            let rect = element.get_bounding_client_rect();
            return [rect.x(), rect.y(), rect.width(), rect.height()];
        }
    }
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    [0.0, 0.0, vp_w as f64, vp_h as f64]
}

/// Extract a label from a target element using tag-specific logic.
/// `data-label` attribute always takes priority.
fn extract_label(el: &web_sys::Element) -> String {
    if let Some(label) = el.get_attribute("data-label") {
        return label;
    }
    match el.tag_name().as_str() {
        "SELECT" => "dropdown".to_string(),
        "INPUT" | "TEXTAREA" => {
            el.get_attribute("placeholder").unwrap_or_else(|| "input".to_string())
        }
        _ => strip_tags(&el.inner_html()).trim().to_string(),
    }
}

/// Query all elements with class "target" inside #viewport, return label + screen bbox.
fn get_target_bboxes() -> Vec<(String, [i32; 4])> {
    let mut targets = Vec::new();
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Some(viewport) = document.get_element_by_id("viewport") {
            let elems = viewport.get_elements_by_class_name("target");
            for i in 0..elems.length() {
                if let Some(el) = elems.item(i) {
                    let rect = el.get_bounding_client_rect();
                    let bbox = [
                        rect.x() as i32,
                        rect.y() as i32,
                        rect.width() as i32,
                        rect.height() as i32,
                    ];
                    let label = extract_label(&el);
                    targets.push((label, bbox));
                }
            }
        }
    }
    targets
}

/// Classify a target's visibility relative to the viewport's visible rect.
/// `bbox` is the target's screen-space [x, y, w, h] from getBoundingClientRect.
/// `vp` is the viewport's screen-space [x, y, w, h].
fn target_visibility(bbox: &[i32; 4], vp: &[f64; 4]) -> &'static str {
    let (tx, ty, tw, th) = (bbox[0] as f64, bbox[1] as f64, bbox[2] as f64, bbox[3] as f64);
    let (vx, vy, vw, vh) = (vp[0], vp[1], vp[2], vp[3]);

    // Overlap on each axis
    let overlap_x = (tx + tw).min(vx + vw) - tx.max(vx);
    let overlap_y = (ty + th).min(vy + vh) - ty.max(vy);

    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        "offscreen"
    } else {
        let visible_area = overlap_x * overlap_y;
        let total_area = (tw * th).max(1.0);
        if visible_area >= total_area * 0.99 {
            "visible"
        } else {
            "partial"
        }
    }
}

/// Get the viewport's current scroll position [scrollLeft, scrollTop].
fn get_viewport_scroll() -> [i32; 2] {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Some(el) = document.get_element_by_id("viewport") {
            return [el.scroll_left(), el.scroll_top()];
        }
    }
    [0, 0]
}

#[component]
pub fn GroundTruth(
    description: String,
    target_x: f32,
    target_y: f32,
    target_w: f32,
    target_h: f32,
    #[props(default)] steps: String,
    #[props(default)] tree: Option<UINode>,
) -> Element {
    let (vp_init_w, vp_init_h) = crate::primitives::viewport_size();
    let mut vp_signal = use_signal(move || [0.0f64, 0.0, vp_init_w as f64, vp_init_h as f64]);
    let mut win_signal = use_signal(|| [0i32, 0, 0, 0]);
    let mut scroll_signal = use_signal(|| [0i32, 0]);
    let mut targets_signal = use_signal(Vec::<(String, [i32; 4])>::new);

    // Tick counter — polls DOM periodically to catch interactive changes
    // (e.g. dropdown open/close moving class="target" between elements)
    let mut tick = use_signal(|| 0u32);
    // use_future is cancelled automatically when the component unmounts,
    // preventing leaked infinite loops that grow WASM memory.
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(200).await;
            let t = *tick.peek();
            tick.set(t.wrapping_add(1));
        }
    });

    // Track description changes for immediate re-query on level state changes
    let mut prev_desc = use_signal(|| String::new());
    if *prev_desc.peek() != description {
        prev_desc.set(description.clone());
    }

    // Re-runs on tick (200ms poll) or description change (level state)
    use_effect(move || {
        let _ = tick.read();
        let _ = prev_desc.read();
        let win = get_window_bbox();
        let vp = get_viewport_bbox();
        let scroll = get_viewport_scroll();
        let targets = get_target_bboxes();
        if *win_signal.peek() != win {
            win_signal.set(win);
        }
        if *vp_signal.peek() != vp {
            vp_signal.set(vp);
        }
        if *scroll_signal.peek() != scroll {
            scroll_signal.set(scroll);
        }
        if *targets_signal.peek() != targets {
            targets_signal.set(targets);
        }
    });

    let win = *win_signal.read();
    let vp = *vp_signal.read();
    let scroll = *scroll_signal.read();
    let dom_targets = targets_signal.read().clone();

    // Resolve UINode tree with viewport transform → window-space coordinates
    let vt = ViewportTransform::from_viewport(&vp);
    let resolved = tree.as_ref().map(|t| t.resolve_with(&vt));
    let description = resolved.as_ref().map_or(description, |r| r.description.clone());
    let steps = resolved.as_ref().map_or(steps, |r| r.steps_json());
    let thinking = resolved.as_ref().map(|r| r.thinking.clone()).unwrap_or_default();

    // Build targets string: prefer DOM-queried targets, fall back to props.
    // Each target gets a "visibility" field: "visible", "partial", or "offscreen".
    let targets_str = if !dom_targets.is_empty() {
        let parts: Vec<String> = dom_targets.iter()
            .map(|(label, t)| {
                let vis = target_visibility(t, &vp);
                if label.is_empty() {
                    format!("{{\"bbox\": [{}, {}, {}, {}], \"visibility\": \"{}\"}}", t[0], t[1], t[2], t[3], vis)
                } else {
                    format!("{{\"label\": \"{}\", \"bbox\": [{}, {}, {}, {}], \"visibility\": \"{}\"}}", label, t[0], t[1], t[2], t[3], vis)
                }
            })
            .collect();
        format!("[{}]", parts.join(", "))
    } else {
        // Fallback: compute from props + viewport offset
        let (vp_w, _vp_h) = crate::primitives::viewport_size();
        let scale = if vp_w > 0.0 { vp[2] / vp_w as f64 } else { 1.0 };
        let target = [
            (vp[0] + target_x as f64 * scale) as i32,
            (vp[1] + target_y as f64 * scale) as i32,
            (target_w as f64 * scale) as i32,
            (target_h as f64 * scale) as i32,
        ];
        format!("[{{\"bbox\": [{}, {}, {}, {}], \"visibility\": \"visible\"}}]", target[0], target[1], target[2], target[3])
    };

    let window_str = format!("[{}, {}, {}, {}]", win[0], win[1], win[2], win[3]);
    let viewport_str = format!("[{}, {}, {}, {}]", vp[0] as i32, vp[1] as i32, vp[2] as i32, vp[3] as i32);
    let scroll_str = format!("[{}, {}]", scroll[0], scroll[1]);

    // Post-render visibility thinking: augment the UINode thinking with
    // visibility annotations based on actual DOM measurements.
    let vis_thinking = if !dom_targets.is_empty() {
        let mut parts = Vec::new();
        for (label, bbox) in &dom_targets {
            let vis = target_visibility(bbox, &vp);
            match vis {
                "offscreen" => parts.push(format!(
                    "I cannot see \"{}\" — it is off-screen. I need to scroll to find it.", label
                )),
                "partial" => parts.push(format!(
                    "\"{}\" is partially cut off by the viewport edge. I may need to scroll to see it fully.", label
                )),
                _ => {} // visible — no extra thinking needed
            }
        }
        parts.join(" ")
    } else {
        String::new()
    };

    // Combine UINode thinking with post-render visibility thinking
    let full_thinking = if vis_thinking.is_empty() {
        thinking
    } else if thinking.is_empty() {
        vis_thinking
    } else {
        format!("{} {}", thinking, vis_thinking)
    };

    rsx! {
        div {
            id: "ground-truth",
            style: "max-width: 1024px; width: 100%; max-height: 180px; overflow-y: auto; background: #111827; border-radius: 8px; padding: 16px; margin-top: 12px; font-family: monospace; font-size: 12px; color: #9ca3af; box-sizing: border-box; word-break: break-all;",
            h3 {
                style: "margin: 0 0 8px 0; color: #e5e7eb; font-size: 13px;",
                "Ground Truth"
            }
            div { style: "padding: 4px 0;", "{description}" }
            div { style: "padding: 4px 0; color: #6b7280;", "window: {window_str}" }
            div { style: "padding: 4px 0; color: #6b7280;", "viewport: {viewport_str}" }
            if scroll[0] != 0 || scroll[1] != 0 {
                div { style: "padding: 4px 0; color: #6b7280;", "scroll: {scroll_str}" }
            }
            div { style: "padding: 4px 0; color: #6b7280;", "targets: {targets_str}" }
            if !steps.is_empty() {
                div { style: "padding: 4px 0; color: #6b7280;", "steps: {steps}" }
            }
            if !full_thinking.is_empty() {
                div { style: "padding: 4px 0; color: #8b5cf6;", "thinking: {full_thinking}" }
            }
        }
    }
}
