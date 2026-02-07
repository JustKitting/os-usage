use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn Landing() -> Element {
    use_effect(|| {
        document::eval(r#"
            if (!window.__landingDebugToggleInstalled) {
                window.__landingDebugToggleInstalled = true;
                const key = 'playgroundDebug';
                const toggle = document.getElementById('debug-toggle');
                const label = document.getElementById('debug-toggle-label');
                const seedInput = document.getElementById('seed-input');
                const seedApply = document.getElementById('seed-apply');
                const seedRandom = document.getElementById('seed-random');
                const seedCurrent = document.getElementById('seed-current');

                const apply = (enabled) => {
                    if (toggle) {
                        toggle.textContent = enabled ? 'Debug mode: On' : 'Debug mode: Off';
                        toggle.dataset.enabled = enabled ? 'true' : 'false';
                    }
                    if (label) {
                        label.textContent = enabled
                            ? 'Ground-truth overlays are visible (debug mode).'
                            : 'Ground-truth overlays are hidden (eval mode).';
                    }
                    if (window.__setDebugMode) {
                        window.__setDebugMode(enabled);
                    }
                };

                let enabled = false;
                try { enabled = localStorage.getItem(key) === '1'; } catch {}
                apply(enabled);

                if (toggle) {
                    toggle.addEventListener('click', () => {
                        enabled = !enabled;
                        try { localStorage.setItem(key, enabled ? '1' : '0'); } catch {}
                        apply(enabled);
                    });
                }

                const seedKey = 'playgroundSeed';
                const toSeed = (value) => {
                    const num = Number(value);
                    if (!Number.isFinite(num) || num < 0) return null;
                    return Math.floor(num);
                };
                const applySeed = (value) => {
                    const parsed = toSeed(value);
                    if (parsed === null) return null;
                    if (window.__setSeed) {
                        const next = window.__setSeed(parsed);
                        if (seedInput) seedInput.value = String(next);
                        if (seedCurrent) seedCurrent.textContent = `Current seed: ${next}`;
                        return next;
                    }
                    return null;
                };
                let seedValue = toSeed(window.__playgroundSeed);
                if (seedValue === null) {
                    try { seedValue = toSeed(localStorage.getItem(seedKey)); } catch {}
                }
                if (seedValue === null) seedValue = 0;
                if (seedInput) seedInput.value = String(seedValue);
                if (seedCurrent) seedCurrent.textContent = `Current seed: ${seedValue}`;

                if (seedApply) {
                    seedApply.addEventListener('click', () => {
                        applySeed(seedInput ? seedInput.value : seedValue);
                    });
                }
                if (seedRandom) {
                    seedRandom.addEventListener('click', () => {
                        const next = Math.floor(Math.random() * 1_000_000_000);
                        try { localStorage.setItem(seedKey, String(next)); } catch {}
                        applySeed(next);
                    });
                }
            }
        "#);
    });

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

            // Debug toggle
            div {
                style: "margin-top: 40px; background: #111827; border: 1px solid #2a2a4a; border-radius: 12px; padding: 20px 24px; max-width: 720px; width: 100%; text-align: center;",
                h3 {
                    style: "color: #e5e7eb; font-size: 16px; margin: 0 0 8px 0;",
                    "Debug Mode"
                }
                p {
                    id: "debug-toggle-label",
                    style: "color: #9ca3af; font-size: 14px; margin: 0 0 12px 0;",
                    "Ground-truth overlays are hidden (eval mode)."
                }
                button {
                    id: "debug-toggle",
                    style: "padding: 10px 18px; border: 1px solid #334155; border-radius: 8px; background: #0f172a; color: #e2e8f0; font-size: 14px; font-weight: 600; cursor: pointer;",
                    "Debug mode: Off"
                }
                p {
                    style: "color: #6b7280; font-size: 12px; margin: 10px 0 0 0;",
                    "Use this in a debug browser to surface ground-truth data. Eval mode keeps training targets hidden."
                }
            }

            // Seed controls
            div {
                style: "margin-top: 20px; background: #111827; border: 1px solid #2a2a4a; border-radius: 12px; padding: 20px 24px; max-width: 720px; width: 100%; text-align: center;",
                h3 {
                    style: "color: #e5e7eb; font-size: 16px; margin: 0 0 8px 0;",
                    "Deterministic Seed"
                }
                p {
                    style: "color: #9ca3af; font-size: 14px; margin: 0 0 12px 0;",
                    "Set a seed to make randomized layouts repeatable across sessions."
                }
                div {
                    style: "display: flex; gap: 8px; justify-content: center; flex-wrap: wrap;",
                    input {
                        id: "seed-input",
                        r#type: "number",
                        min: "0",
                        style: "padding: 8px 12px; border: 1px solid #334155; border-radius: 8px; background: #0f172a; color: #e2e8f0; font-size: 14px; width: 180px;",
                    }
                    button {
                        id: "seed-apply",
                        style: "padding: 8px 14px; border: 1px solid #334155; border-radius: 8px; background: #1f2937; color: #e2e8f0; font-size: 14px; font-weight: 600; cursor: pointer;",
                        "Apply"
                    }
                    button {
                        id: "seed-random",
                        style: "padding: 8px 14px; border: 1px solid #334155; border-radius: 8px; background: #0f172a; color: #e2e8f0; font-size: 14px; font-weight: 600; cursor: pointer;",
                        "Randomize"
                    }
                }
                p {
                    id: "seed-current",
                    style: "color: #6b7280; font-size: 12px; margin: 10px 0 0 0;",
                    "Current seed: 0"
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
