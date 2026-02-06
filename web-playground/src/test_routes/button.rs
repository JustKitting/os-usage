use dioxus::prelude::*;

use crate::levels::GroundTruth;
use crate::ui_node::{self, Rect};

#[component]
pub fn TestButton() -> Element {
    let mut clicked = use_signal(|| false);

    let is_clicked = clicked();
    let bg = if is_clicked { "#22c55e" } else { "#3b82f6" };
    let label = if is_clicked { "Clicked!" } else { "Click me" };
    let cursor = if is_clicked { "default" } else { "pointer" };

    let tree = ui_node::target_button("Click me", Rect::new(360.0, 270.0, 80.0, 40.0));

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: system-ui, sans-serif;",

            div {
                id: "viewport",
                "data-fixed": "true",
                style: "width: 800px; height: 600px; background: #1a1a2e; position: relative; overflow: hidden;",

                button {
                    class: "target",
                    "data-label": "{label}",
                    style: "position: absolute; left: 360px; top: 270px; padding: 10px 24px; background: {bg}; color: white; border: none; border-radius: 6px; cursor: {cursor}; font-size: 14px; font-family: system-ui, sans-serif;",
                    onclick: move |_| { clicked.set(true); },
                    "{label}"
                }

                div {
                    id: "result",
                    style: "display: none;",
                    if is_clicked { "clicked" } else { "idle" }
                }
            }

            GroundTruth {
                description: String::new(),
                target_x: 360.0,
                target_y: 270.0,
                target_w: 80.0,
                target_h: 40.0,
                tree: Some(tree),
            }
        }
    }
}
