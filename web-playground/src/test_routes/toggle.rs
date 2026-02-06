use dioxus::prelude::*;

use crate::levels::GroundTruth;
use crate::ui_node::{self, Rect};

#[component]
pub fn TestToggle() -> Element {
    let mut is_on = use_signal(|| false);

    let on = is_on();
    let track_color = if on { "#3b82f6" } else { "#d1d5db" };
    let knob_left = if on { "22px" } else { "2px" };
    let result = if on { "on" } else { "off" };

    let tree = ui_node::toggle("Dark mode", Rect::new(340.0, 285.0, 120.0, 30.0), false);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: system-ui, sans-serif;",

            div {
                id: "viewport",
                "data-fixed": "true",
                style: "width: 800px; height: 600px; background: #1a1a2e; position: relative; overflow: hidden;",

                div {
                    class: "target",
                    "data-label": "Dark mode",
                    style: "position: absolute; left: 340px; top: 285px; display: flex; align-items: center; gap: 10px; cursor: pointer; user-select: none;",
                    onclick: move |_| { is_on.set(!on); },

                    span { style: "color: #e5e7eb; font-size: 14px;", "Dark mode" }

                    div {
                        style: "width: 44px; height: 24px; background: {track_color}; border-radius: 12px; position: relative; transition: background 0.15s;",
                        div {
                            style: "width: 20px; height: 20px; background: white; border-radius: 50%; position: absolute; top: 2px; left: {knob_left}; box-shadow: 0 1px 3px rgba(0,0,0,0.2); transition: left 0.15s;",
                        }
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
                target_x: 340.0,
                target_y: 285.0,
                target_w: 120.0,
                target_h: 30.0,
                tree: Some(tree),
            }
        }
    }
}
