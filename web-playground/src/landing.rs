use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn Landing() -> Element {
    rsx! {
        div {
            style: "min-height: 100vh; background: #0f0f1a; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 40px 20px; font-family: system-ui, -apple-system, sans-serif;",

            // Hero
            div {
                style: "text-align: center; max-width: 720px;",
                h1 {
                    style: "font-size: 48px; font-weight: 700; color: #e5e7eb; margin: 0 0 16px 0; letter-spacing: -1px;",
                    "Web Playground"
                }
                p {
                    style: "font-size: 20px; color: #9ca3af; margin: 0 0 40px 0; line-height: 1.6;",
                    "A training ground for vision-language models. Randomized UI elements with transforms, animations, and interactive states \u{2014} everything a browser agent needs to learn."
                }
                div {
                    style: "display: flex; gap: 16px; justify-content: center;",
                    Link {
                        to: Route::LevelSelect {},
                        style: "display: inline-block; padding: 14px 36px; background: linear-gradient(135deg, #22c55e, #16a34a); color: white; text-decoration: none; border-radius: 8px; font-size: 18px; font-weight: 600;",
                        "Play \u{2192}"
                    }
                    Link {
                        to: Route::Playground {},
                        style: "display: inline-block; padding: 14px 36px; background: linear-gradient(135deg, #3b82f6, #6366f1); color: white; text-decoration: none; border-radius: 8px; font-size: 18px; font-weight: 600;",
                        "Sandbox \u{2192}"
                    }
                }
            }

            // Feature grid
            div {
                style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; max-width: 800px; margin-top: 64px;",

                // Card 1
                div {
                    style: "background: #1a1a2e; border: 1px solid #2a2a4a; border-radius: 10px; padding: 24px;",
                    h3 {
                        style: "color: #e5e7eb; font-size: 16px; margin: 0 0 8px 0;",
                        "Element Pool"
                    }
                    p {
                        style: "color: #6b7280; font-size: 14px; margin: 0; line-height: 1.5;",
                        "Buttons, inputs, checkboxes, toggles, dropdowns \u{2014} diverse HTML snippets from real design systems."
                    }
                }

                // Card 2
                div {
                    style: "background: #1a1a2e; border: 1px solid #2a2a4a; border-radius: 10px; padding: 24px;",
                    h3 {
                        style: "color: #e5e7eb; font-size: 16px; margin: 0 0 8px 0;",
                        "5 Transform Axes"
                    }
                    p {
                        style: "color: #6b7280; font-size: 14px; margin: 0; line-height: 1.5;",
                        "Position, scale, rotation, opacity, and animation. Each element gets randomized transforms for maximum diversity."
                    }
                }

                // Card 3
                div {
                    style: "background: #1a1a2e; border: 1px solid #2a2a4a; border-radius: 10px; padding: 24px;",
                    h3 {
                        style: "color: #e5e7eb; font-size: 16px; margin: 0 0 8px 0;",
                        "DOM Query API"
                    }
                    p {
                        style: "color: #6b7280; font-size: 14px; margin: 0; line-height: 1.5;",
                        "Call window.getElements() from any debugger client to get live positions, bounds, and state of every element."
                    }
                }
            }

            // Footer
            p {
                style: "color: #4b5563; font-size: 13px; margin-top: 64px;",
                "Built for training browser-use VLMs"
            }
        }
    }
}
