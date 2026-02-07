mod canvas;
mod landing;
mod level_select;
mod levels;
mod pool;
mod primitives;
mod transform;

use dioxus::prelude::*;
use canvas::Playground;
use landing::Landing;
use level_select::LevelSelect;
use levels::{Level1, Level2, Level3, Level4, Level5, Level6, Level7, Level8, Level9, Level10, Level11, Level12, Level13, Level14, Level15, Level16, Level17, Level18, Level19, Level20, Level21, Level22, Level23, Level24, Level25, Level26, Level27};

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Landing {},
    #[route("/levels")]
    LevelSelect {},
    #[route("/level1")]
    Level1 {},
    #[route("/level2")]
    Level2 {},
    #[route("/level3")]
    Level3 {},
    #[route("/level4")]
    Level4 {},
    #[route("/level5")]
    Level5 {},
    #[route("/level6")]
    Level6 {},
    #[route("/level7")]
    Level7 {},
    #[route("/level8")]
    Level8 {},
    #[route("/level9")]
    Level9 {},
    #[route("/level10")]
    Level10 {},
    #[route("/level11")]
    Level11 {},
    #[route("/level12")]
    Level12 {},
    #[route("/level13")]
    Level13 {},
    #[route("/level14")]
    Level14 {},
    #[route("/level15")]
    Level15 {},
    #[route("/level16")]
    Level16 {},
    #[route("/level17")]
    Level17 {},
    #[route("/level18")]
    Level18 {},
    #[route("/level19")]
    Level19 {},
    #[route("/level20")]
    Level20 {},
    #[route("/level21")]
    Level21 {},
    #[route("/level22")]
    Level22 {},
    #[route("/level23")]
    Level23 {},
    #[route("/level24")]
    Level24 {},
    #[route("/level25")]
    Level25 {},
    #[route("/level26")]
    Level26 {},
    #[route("/level27")]
    Level27 {},
    #[route("/playground")]
    Playground {},
}

#[allow(non_snake_case)]
fn App() -> Element {
    // Install global event listeners once (capture phase to see everything)
    use_effect(|| {
        document::eval(r#"
            if (!window.__playgroundListeners) {
                window.__playgroundListeners = true;
                const log = (type, data) => console.log(JSON.stringify({ event: type, ...data, ts: Date.now() }));

                document.addEventListener('mousedown', (e) => {
                    log('mousedown', { x: e.clientX, y: e.clientY, button: e.button });
                }, true);

                document.addEventListener('mouseup', (e) => {
                    log('mouseup', { x: e.clientX, y: e.clientY, button: e.button });
                }, true);

                document.addEventListener('click', (e) => {
                    log('click', { x: e.clientX, y: e.clientY, button: e.button });
                }, true);

                document.addEventListener('dblclick', (e) => {
                    log('dblclick', { x: e.clientX, y: e.clientY });
                }, true);

                document.addEventListener('contextmenu', (e) => {
                    log('contextmenu', { x: e.clientX, y: e.clientY });
                }, true);

                document.addEventListener('keydown', (e) => {
                    log('keydown', { key: e.key, code: e.code });
                }, true);

                document.addEventListener('keyup', (e) => {
                    log('keyup', { key: e.key, code: e.code });
                }, true);

                document.addEventListener('input', (e) => {
                    log('input', { value: e.target.value || '' });
                }, true);

                document.addEventListener('change', (e) => {
                    log('change', { value: e.target.value || '' });
                }, true);

                document.addEventListener('wheel', (e) => {
                    log('wheel', { x: e.clientX, y: e.clientY, deltaX: e.deltaX, deltaY: e.deltaY });
                }, true);

                document.addEventListener('dragstart', (e) => {
                    log('dragstart', { x: e.clientX, y: e.clientY });
                }, true);

                document.addEventListener('dragend', (e) => {
                    log('dragend', { x: e.clientX, y: e.clientY });
                }, true);

                document.addEventListener('drop', (e) => {
                    log('drop', { x: e.clientX, y: e.clientY });
                }, true);
            }

            // ── Solver: step-through automation for VLM training data ──
            if (!window.__solver) {
                window.__solver = {
                    _stepIndex: 0,
                    _lastStepsJson: '',

                    getGroundTruth() {
                        const panel = document.getElementById('ground-truth');
                        if (!panel) { console.warn('solver: ground-truth panel not found'); return { targets: [], steps: [] }; }
                        let targets = [], steps = [];
                        for (const div of panel.querySelectorAll(':scope > div')) {
                            const t = div.textContent;
                            if (t.startsWith('targets: ')) {
                                try { targets = JSON.parse(t.slice(9)); } catch (e) { console.warn('solver: failed to parse targets', e); }
                            } else if (t.startsWith('steps: ')) {
                                try { steps = JSON.parse(t.slice(7)); } catch (e) { console.warn('solver: failed to parse steps', e); }
                            }
                        }
                        return { targets, steps };
                    },

                    _bbox(label, targets) {
                        const t = targets.find(t => t.label === label);
                        if (!t) return null;
                        const [x, y, w, h] = t.bbox;
                        return { x, y, w, h, cx: x + w / 2, cy: y + h / 2 };
                    },

                    _dispatchAt(x, y, type, opts) {
                        const el = document.elementFromPoint(x, y);
                        if (!el) { console.warn('solver: nothing at', x, y); return null; }
                        const ev = new MouseEvent(type, {
                            clientX: x, clientY: y, screenX: x, screenY: y,
                            bubbles: true, cancelable: true, view: window, ...opts
                        });
                        el.dispatchEvent(ev);
                        return el;
                    },

                    async _doClick(label, targets) {
                        const b = this._bbox(label, targets);
                        if (!b) { console.warn('solver: target not found:', label, 'available:', targets.map(t=>t.label)); return; }
                        const cx = b.cx, cy = b.cy;
                        console.log('solver: click "' + label + '" at (' + cx + ', ' + cy + ') bbox [' + b.x + ',' + b.y + ',' + b.w + ',' + b.h + ']');
                        // Full mouse event sequence at ground truth coordinates
                        const el = this._dispatchAt(cx, cy, 'pointerdown');
                        this._dispatchAt(cx, cy, 'mousedown');
                        this._dispatchAt(cx, cy, 'pointerup');
                        this._dispatchAt(cx, cy, 'mouseup');
                        this._dispatchAt(cx, cy, 'click');
                        if (el) console.log('solver: hit', el.tagName, el.className, el.getAttribute('data-label') || el.textContent?.slice(0,30));
                    },

                    async _doType(label, value, targets) {
                        const b = this._bbox(label, targets);
                        if (!b) { console.warn('solver: target not found:', label); return; }
                        const el = document.elementFromPoint(b.cx, b.cy);
                        if (!el) return;
                        el.focus();
                        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set
                                     || Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value')?.set;
                        if (setter) setter.call(el, value);
                        else el.value = value;
                        el.dispatchEvent(new Event('input', { bubbles: true }));
                    },

                    async _doDrag(fromLabel, toLabel, targets) {
                        const from = this._bbox(fromLabel, targets);
                        const to = this._bbox(toLabel, targets);
                        if (!from || !to) { console.warn('solver: drag targets not found:', fromLabel, toLabel); return; }
                        const el = this._dispatchAt(from.cx, from.cy, 'mousedown');
                        if (!el) return;
                        await new Promise(r => setTimeout(r, 30));
                        const steps = 10;
                        for (let i = 1; i <= steps; i++) {
                            const t = i / steps;
                            const mx = from.cx + (to.cx - from.cx) * t;
                            const my = from.cy + (to.cy - from.cy) * t;
                            const moveEl = document.elementFromPoint(mx, my) || el;
                            moveEl.dispatchEvent(new MouseEvent('mousemove', {
                                clientX: mx, clientY: my, bubbles: true, cancelable: true, view: window
                            }));
                            await new Promise(r => setTimeout(r, 15));
                        }
                        (document.elementFromPoint(to.cx, to.cy) || el).dispatchEvent(new MouseEvent('mouseup', {
                            clientX: to.cx, clientY: to.cy, bubbles: true, cancelable: true, view: window
                        }));
                    },

                    async _doRightClick(label, targets) {
                        const b = this._bbox(label, targets);
                        if (!b) { console.warn('solver: target not found:', label); return; }
                        this._dispatchAt(b.cx, b.cy, 'contextmenu');
                    },

                    async step() {
                        const gt = this.getGroundTruth();
                        const stepsJson = JSON.stringify(gt.steps);
                        if (stepsJson !== this._lastStepsJson) {
                            this._stepIndex = 0;
                            this._lastStepsJson = stepsJson;
                        }
                        if (!gt.steps.length || this._stepIndex >= gt.steps.length) {
                            console.log('solver: no more steps');
                            return null;
                        }
                        const action = gt.steps[this._stepIndex];
                        switch (action.action) {
                            case 'click':       await this._doClick(action.target, gt.targets); break;
                            case 'type':        await this._doType(action.target, action.value, gt.targets); break;
                            case 'drag':        await this._doDrag(action.from, action.to, gt.targets); break;
                            case 'right_click': await this._doRightClick(action.target, gt.targets); break;
                        }
                        this._stepIndex++;
                        await new Promise(r => setTimeout(r, 300));
                        return { step: this._stepIndex, ...action };
                    },

                    async solve() {
                        this._stepIndex = 0;
                        while (this._stepIndex < 50) {
                            const gt = this.getGroundTruth();
                            if (!gt.steps.length || this._stepIndex >= gt.steps.length) break;
                            await this.step();
                        }
                    },

                    reset() { this._stepIndex = 0; }
                };
                console.log('solver: ready — use __solver.step() / __solver.solve() / __solver.reset()');

                // Inject step toolbar
                const bar = document.createElement('div');
                bar.id = '__solver-bar';
                bar.style.cssText = 'position:fixed;top:8px;right:8px;z-index:99999;display:flex;gap:6px;font-family:system-ui,sans-serif;';
                const mkBtn = (label, fn) => {
                    const b = document.createElement('button');
                    b.textContent = label;
                    b.style.cssText = 'padding:6px 14px;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer;color:white;background:#4f46e5;opacity:0.9;transition:opacity 0.1s;';
                    b.onmouseenter = () => b.style.opacity = '1';
                    b.onmouseleave = () => b.style.opacity = '0.9';
                    b.onclick = fn;
                    return b;
                };
                bar.appendChild(mkBtn('Step', async () => {
                    try {
                        console.log('solver: Step button clicked');
                        const gt = window.__solver.getGroundTruth();
                        console.log('solver: ground truth =', JSON.stringify(gt).slice(0, 200));
                        const r = await window.__solver.step();
                        console.log('solver: step result =', r);
                    } catch (e) { console.error('solver: step error', e); }
                }));
                bar.appendChild(mkBtn('Solve', async () => {
                    try {
                        console.log('solver: Solve button clicked');
                        await window.__solver.solve();
                        console.log('solver: solve done');
                    } catch (e) { console.error('solver: solve error', e); }
                }));
                const resetBtn = mkBtn('Reset', () => {
                    console.log('solver: Reset button clicked');
                    window.__solver.reset();
                });
                resetBtn.style.background = '#6b7280';
                bar.appendChild(resetBtn);
                document.body.appendChild(bar);
            }

            // Auto-fit: prevent scrolling by zooming content to fit the viewport
            if (!window.__autoFitInstalled) {
                window.__autoFitInstalled = true;
                document.documentElement.style.overflow = 'hidden';
                document.body.style.overflow = 'hidden';

                function autoFit() {
                    const root = document.getElementById('main');
                    if (!root) return;
                    // Reset zoom to measure natural height
                    root.style.zoom = '1';
                    const contentHeight = root.scrollHeight;
                    const windowHeight = window.innerHeight;
                    if (contentHeight > windowHeight) {
                        const zoom = windowHeight / contentHeight;
                        root.style.zoom = String(zoom);
                    } else {
                        root.style.zoom = '1';
                    }
                }

                // Run on load and resize
                autoFit();
                new ResizeObserver(autoFit).observe(document.body);
                window.addEventListener('resize', autoFit);
                // Re-run after route changes (Dioxus updates DOM async)
                new MutationObserver(autoFit).observe(document.getElementById('main') || document.body, { childList: true, subtree: true });
            }

            // Debug mode: control ground-truth visibility via localStorage + data attribute
            if (!window.__debugModeInstalled) {
                window.__debugModeInstalled = true;
                const key = 'playgroundDebug';
                const style = document.createElement('style');
                style.textContent = '#ground-truth{display:none;} body[data-debug="true"] #ground-truth{display:block;}';
                document.head.appendChild(style);

                window.__setDebugMode = (enabled) => {
                    const isEnabled = !!enabled;
                    document.body.dataset.debug = isEnabled ? 'true' : 'false';
                    window.__debugMode = isEnabled;
                    try { localStorage.setItem(key, isEnabled ? '1' : '0'); } catch {}
                };

                const params = new URLSearchParams(window.location.search);
                const urlFlag = params.get('debug');
                let enabled = false;
                try { enabled = localStorage.getItem(key) === '1'; } catch {}
                if (urlFlag === '1') enabled = true;
                if (urlFlag === '0') enabled = false;
                window.__setDebugMode(enabled);
            }
        "#);
    });

    rsx! {
        div {
            id: "main",
            Router::<Route> {}
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus::launch(App);
}
