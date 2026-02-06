//! Playground - the main training canvas

use dioxus::prelude::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::pool::ElementPool;
use crate::primitives::Animation;
use crate::transform::Sampler;
use super::element::CanvasElement;

/// The 1024x1024 training playground
#[component]
pub fn Playground() -> Element {
    let pool = use_hook(|| ElementPool::with_builtins());
    let pool_total = pool.total();

    let mut seed_counter = use_signal(|| 42u64);
    let mut elements = use_signal(|| {
        let pool = ElementPool::with_builtins();
        let mut rng = SmallRng::seed_from_u64(42);
        Sampler::random_page(&mut rng, &pool, 5)
    });
    let mut clicked = use_signal(|| Option::<String>::None);
    let mut bg_speed = use_signal(|| 30u32);

    let regenerate = {
        let pool = pool.clone();
        move |_| {
            let new_seed = *seed_counter.read() + 1;
            seed_counter.set(new_seed);
            let mut rng = SmallRng::seed_from_u64(new_seed);
            let new_elements = Sampler::random_page(&mut rng, &pool, 5);
            elements.set(new_elements);
            clicked.set(None);
        }
    };

    let keyframes = Animation::keyframes_css();

    rsx! {
        // Inject keyframe definitions once
        style { "{keyframes}" }

        // Expose element state to debugger clients
        script {
            r#"
            window.getElements = function() {{
                return Array.from(document.querySelectorAll('[data-element-id]')).map(function(el) {{
                    return {{
                        id: el.dataset.elementId,
                        kind: el.dataset.kind,
                        label: el.dataset.label,
                        x: parseFloat(el.dataset.x),
                        y: parseFloat(el.dataset.y),
                        width: parseFloat(el.dataset.width),
                        height: parseFloat(el.dataset.height),
                        scale: parseFloat(el.dataset.scale),
                        angle: parseFloat(el.dataset.angle),
                        opacity: parseFloat(el.dataset.opacity),
                        animation: el.dataset.animation || "none",
                        active: el.dataset.active === "true",
                        description: el.dataset.description,
                        rect: el.getBoundingClientRect(),
                    }};
                }});
            }};
            "#
        }

        div {
            style: "display: flex; flex-direction: column; align-items: center; gap: 16px; padding: 20px; background: #0f0f1a; min-height: 100vh;",

            // Controls
            div {
                style: "display: flex; gap: 12px; align-items: center;",
                button {
                    style: "padding: 8px 20px; background: #3b82f6; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px;",
                    onclick: regenerate,
                    "Generate New Page"
                }
                span {
                    style: "color: #9ca3af; font-size: 13px; font-family: monospace;",
                    "{pool_total} snippets in pool"
                }
                if let Some(ref id) = clicked() {
                    span {
                        style: "color: #22c55e; font-size: 13px; font-family: monospace;",
                        "clicked: {id}"
                    }
                }
            }

            // Background speed control
            div {
                style: "display: flex; gap: 8px; align-items: center;",
                label {
                    style: "color: #9ca3af; font-size: 13px; font-family: monospace;",
                    "BG cycle: {bg_speed}s"
                }
                input {
                    r#type: "range",
                    min: "3",
                    max: "120",
                    value: "{bg_speed}",
                    style: "width: 160px; accent-color: #3b82f6;",
                    oninput: move |e: Event<FormData>| {
                        if let Ok(v) = e.value().parse::<u32>() {
                            bg_speed.set(v);
                        }
                    },
                }
            }

            // Canvas
            div {
                style: "width: 1024px; height: 1024px; background: #1a1a2e; position: relative; border: 1px solid #2a2a4a; overflow: hidden; animation: bg-shift {bg_speed}s infinite ease-in-out;",

                for placed in elements() {
                    CanvasElement {
                        key: "{placed.snippet.id}-{placed.position.x}-{placed.position.y}",
                        placed: placed.clone(),
                        on_click: move |id: String| {
                            clicked.set(Some(id));
                        },
                    }
                }
            }

            // Ground truth descriptions
            div {
                style: "width: 1024px; background: #111827; border-radius: 8px; padding: 16px; font-family: monospace; font-size: 12px; color: #9ca3af;",
                h3 {
                    style: "margin: 0 0 8px 0; color: #e5e7eb; font-size: 13px;",
                    "Ground Truth"
                }
                for placed in elements() {
                    div {
                        style: "padding: 4px 0; border-bottom: 1px solid #1f2937;",
                        "{placed.describe()}"
                    }
                }
            }
        }
    }
}
