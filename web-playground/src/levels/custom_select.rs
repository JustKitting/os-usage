use dioxus::prelude::*;

const CHEVRON_SVG: &str = "url('data:image/svg+xml;utf8,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%2212%22 height=%2212%22 viewBox=%220 0 24 24%22 fill=%22none%22 stroke=%22%236b7280%22 stroke-width=%222%22><polyline points=%226 9 12 15 18 9%22/></svg>')";

const PANEL_MAX_H: f64 = 184.0;

/// Read CSS zoom from #main element's inline style (set by autoFit JS).
fn get_zoom() -> f64 {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("main"))
        .and_then(|el| el.get_attribute("style"))
        .and_then(|style| {
            for part in style.split(';') {
                if let Some(val) = part.trim().strip_prefix("zoom:") {
                    return val.trim().parse::<f64>().ok();
                }
            }
            None
        })
        .unwrap_or(1.0)
}

static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[component]
pub fn CustomSelect(
    options: Vec<String>,
    is_target: bool,
    target_option: String,
    border_color: String,
    on_select: EventHandler<String>,
) -> Element {
    let mut is_open = use_signal(|| false);
    let mut selected_text = use_signal(|| String::new());
    let mut panel_pos = use_signal(|| (0.0f64, 0.0f64, 0.0f64));

    let trigger_id = use_hook(|| {
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("cs-{n}")
    });

    let open = is_open();
    let display = if selected_text.read().is_empty() {
        "Choose...".to_string()
    } else {
        selected_text.read().clone()
    };
    let display_color = if selected_text.read().is_empty() { "#9ca3af" } else { "#111" };

    let trigger_is_target = is_target && !open;

    let trigger_style = format!(
        "padding: 10px 32px 10px 14px; border: 1px solid {}; border-radius: 6px; \
         font-size: 14px; font-family: system-ui, sans-serif; background: white; \
         color: {}; cursor: pointer; user-select: none; width: 100%; \
         box-sizing: border-box; text-align: left; \
         background-image: {}; background-repeat: no-repeat; \
         background-position: right 10px center; transition: border-color 0.15s;",
        border_color, display_color, CHEVRON_SVG
    );

    let (panel_left, panel_top, panel_width) = *panel_pos.read();
    let panel_style = format!(
        "position: fixed; left: {panel_left}px; top: {panel_top}px; width: {panel_width}px; \
         background: white; border: 1px solid #d1d5db; \
         border-radius: 6px; \
         box-shadow: 0 4px 12px rgba(0,0,0,0.15); \
         z-index: 1000; max-height: 180px; overflow-y: auto;"
    );

    let tid = trigger_id.clone();

    rsx! {
        div {
            style: "position: relative; width: 100%;",

            // Trigger
            div {
                id: "{trigger_id}",
                class: if trigger_is_target { "target" } else { "" },
                "data-label": "{display}",
                style: "{trigger_style}",
                tabindex: "-1",
                onclick: move |_| {
                    if open {
                        is_open.set(false);
                        return;
                    }
                    // Query trigger rect synchronously via web_sys
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        if let Some(el) = doc.get_element_by_id(&tid) {
                            let rect = el.get_bounding_client_rect();
                            let zoom = get_zoom();
                            let bottom = rect.y() + rect.height();
                            // Panel max-height in screen pixels
                            let panel_screen_h = PANEL_MAX_H * zoom;
                            let window_h = web_sys::window()
                                .and_then(|w| w.inner_height().ok())
                                .and_then(|v| v.as_f64())
                                .unwrap_or(crate::primitives::viewport_size().1 as f64);
                            let top_screen = if (bottom + panel_screen_h) > window_h {
                                rect.y() - panel_screen_h
                            } else {
                                bottom + 2.0
                            };
                            // position:fixed inside zoomed container — divide screen coords by zoom
                            panel_pos.set((rect.x() / zoom, top_screen / zoom, rect.width() / zoom));
                        }
                    }
                    is_open.set(true);
                },
                "{display}"
            }

            // Dropdown panel + backdrop (fixed position to escape overflow:hidden)
            if open {
                // Backdrop
                div {
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 999;",
                    onclick: move |e| {
                        e.stop_propagation();
                        is_open.set(false);
                    },
                }

                // Options panel
                div {
                    style: "{panel_style}",

                    for opt in options.iter() {
                        {
                            let opt_val = opt.clone();
                            let opt_display = opt.clone();
                            let is_target_opt = is_target && *opt == target_option;

                            rsx! {
                                div {
                                    class: if is_target_opt { "target" } else { "" },
                                    "data-label": "{opt_display}",
                                    tabindex: "-1",
                                    style: "padding: 8px 14px; cursor: pointer; font-size: 14px; \
                                            color: #111; font-family: system-ui, sans-serif;",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        selected_text.set(opt_val.clone());
                                        is_open.set(false);
                                        on_select.call(opt_val.clone());
                                    },
                                    "{opt_display}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
