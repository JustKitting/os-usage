use dioxus::prelude::*;

use crate::levels::GroundTruth;
use crate::ui_node::{self, Rect};

#[component]
pub fn TestTextInput() -> Element {
    let mut value = use_signal(|| String::new());
    let mut correct = use_signal(|| false);
    let target_word = "hello";

    let result = if correct() {
        "correct".to_string()
    } else if value.read().is_empty() {
        "empty".to_string()
    } else {
        format!("typing:{}", value.read())
    };

    let tree = ui_node::text_input("Test Input", Rect::new(290.0, 280.0, 220.0, 36.0), "Type here...", target_word);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: system-ui, sans-serif;",

            div {
                id: "viewport",
                "data-fixed": "true",
                style: "width: 800px; height: 600px; background: #1a1a2e; position: relative; overflow: hidden;",

                input {
                    class: "target",
                    "data-label": "Test Input",
                    r#type: "text",
                    tabindex: "-1",
                    style: "position: absolute; left: 290px; top: 280px; padding: 10px 14px; border: 1px solid #d1d5db; border-radius: 6px; font-size: 14px; font-family: system-ui, sans-serif; outline: none; width: 220px; background: white; color: #111; box-sizing: border-box;",
                    placeholder: "Type here...",
                    value: "{value}",
                    oninput: move |e: Event<FormData>| {
                        let val = e.value();
                        value.set(val.clone());
                        if val == target_word {
                            correct.set(true);
                        }
                    },
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
                target_y: 280.0,
                target_w: 220.0,
                target_h: 36.0,
                tree: Some(tree),
            }
        }
    }
}
