use dioxus::prelude::*;

use crate::levels::{CustomSelect, GroundTruth};
use crate::ui_node::{self, Rect};

#[component]
pub fn TestDropdown() -> Element {
    let mut selected = use_signal(|| String::new());
    let options = vec!["Apple".to_string(), "Banana".to_string(), "Cherry".to_string()];
    let target = "Banana".to_string();

    let result = if selected.read().is_empty() {
        "none".to_string()
    } else {
        format!("selected:{}", selected.read())
    };

    let tree = ui_node::dropdown(
        "Fruit",
        Rect::new(290.0, 270.0, 220.0, 36.0),
        options.clone(),
        "Banana",
    );

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: system-ui, sans-serif;",

            div {
                id: "viewport",
                "data-fixed": "true",
                style: "width: 800px; height: 600px; background: #1a1a2e; position: relative; overflow: hidden;",

                div {
                    style: "position: absolute; left: 290px; top: 270px; width: 220px;",

                    CustomSelect {
                        options: options,
                        is_target: true,
                        target_option: target,
                        border_color: "#d1d5db".to_string(),
                        on_select: move |val: String| {
                            selected.set(val);
                        },
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
                target_x: 290.0,
                target_y: 270.0,
                target_w: 220.0,
                target_h: 36.0,
                tree: Some(tree),
            }
        }
    }
}
