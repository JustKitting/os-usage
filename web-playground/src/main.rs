mod canvas;
mod landing;
mod level_select;
mod levels;
mod pool;
mod primitives;
mod test_routes;
mod transform;
pub mod ui_node;

use dioxus::prelude::*;
use canvas::Playground;
use landing::Landing;
use level_select::LevelSelect;
use levels::{Level1, Level2, Level3, Level4, Level5, Level6, Level7, Level8, Level9, Level10, Level11, Level12, Level13, Level14, Level15, Level16, Level17, Level18, Level19, Level20, Level21, Level22, Level23, Level24, Level25, Level26, Level27, LevelScroll};
use test_routes::{TestButton, TestTextInput, TestToggle, TestDropdown, TestDrag, TestReorder};

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
    #[route("/level-scroll")]
    LevelScroll {},
    #[route("/playground")]
    Playground {},
    #[route("/test/button")]
    TestButton {},
    #[route("/test/text-input")]
    TestTextInput {},
    #[route("/test/toggle")]
    TestToggle {},
    #[route("/test/dropdown")]
    TestDropdown {},
    #[route("/test/drag")]
    TestDrag {},
    #[route("/test/reorder")]
    TestReorder {},
}

#[allow(non_snake_case)]
fn App() -> Element {
    // Install global event listeners once (capture phase to see everything)
    use_effect(|| {
        document::eval(r#"
            if (!window.__playgroundListeners) {
                window.__playgroundListeners = true;
                const log = (type, data) => console.log(JSON.stringify({ event: type, ...data, ts: Date.now() }));

                // Store listener refs so we can remove them on unload
                const listeners = [];
                function addCapture(target, type, fn) {
                    target.addEventListener(type, fn, true);
                    listeners.push({ target, type, fn });
                }

                addCapture(document, 'mousedown', (e) => log('mousedown', { x: e.clientX, y: e.clientY, button: e.button }));
                addCapture(document, 'mouseup', (e) => log('mouseup', { x: e.clientX, y: e.clientY, button: e.button }));
                addCapture(document, 'click', (e) => log('click', { x: e.clientX, y: e.clientY, button: e.button }));
                addCapture(document, 'dblclick', (e) => log('dblclick', { x: e.clientX, y: e.clientY }));
                addCapture(document, 'contextmenu', (e) => log('contextmenu', { x: e.clientX, y: e.clientY }));
                addCapture(document, 'keydown', (e) => log('keydown', { key: e.key, code: e.code }));
                addCapture(document, 'keyup', (e) => log('keyup', { key: e.key, code: e.code }));
                addCapture(document, 'input', (e) => log('input', { value: e.target.value || '' }));
                addCapture(document, 'change', (e) => log('change', { value: e.target.value || '' }));
                addCapture(document, 'wheel', (e) => log('wheel', { x: e.clientX, y: e.clientY, deltaX: e.deltaX, deltaY: e.deltaY }));
                addCapture(document, 'dragstart', (e) => log('dragstart', { x: e.clientX, y: e.clientY }));
                addCapture(document, 'dragend', (e) => log('dragend', { x: e.clientX, y: e.clientY }));
                addCapture(document, 'drop', (e) => log('drop', { x: e.clientX, y: e.clientY }));

                window.__playgroundCleanupListeners = listeners;
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
                        // Dispatch both pointer and mouse events (matching _doClick pattern)
                        // — Dioxus may listen for pointer events rather than mouse events.
                        this._dispatchAt(from.cx, from.cy, 'pointerdown');
                        const el = this._dispatchAt(from.cx, from.cy, 'mousedown');
                        if (!el) return;
                        await new Promise(r => setTimeout(r, 30));
                        const steps = 10;
                        for (let i = 1; i <= steps; i++) {
                            const t = i / steps;
                            const mx = from.cx + (to.cx - from.cx) * t;
                            const my = from.cy + (to.cy - from.cy) * t;
                            const moveEl = document.elementFromPoint(mx, my) || el;
                            moveEl.dispatchEvent(new PointerEvent('pointermove', {
                                clientX: mx, clientY: my, bubbles: true, cancelable: true, view: window
                            }));
                            moveEl.dispatchEvent(new MouseEvent('mousemove', {
                                clientX: mx, clientY: my, bubbles: true, cancelable: true, view: window
                            }));
                            await new Promise(r => setTimeout(r, 15));
                        }
                        const endEl = document.elementFromPoint(to.cx, to.cy) || el;
                        endEl.dispatchEvent(new PointerEvent('pointerup', {
                            clientX: to.cx, clientY: to.cy, bubbles: true, cancelable: true, view: window
                        }));
                        endEl.dispatchEvent(new MouseEvent('mouseup', {
                            clientX: to.cx, clientY: to.cy, bubbles: true, cancelable: true, view: window
                        }));
                    },

                    async _doRightClick(label, targets) {
                        const b = this._bbox(label, targets);
                        if (!b) { console.warn('solver: target not found:', label); return; }
                        this._dispatchAt(b.cx, b.cy, 'contextmenu');
                    },

                    async _doScroll(label, targets) {
                        const b = this._bbox(label, targets);
                        if (!b) { console.warn('solver: scroll target not found:', label); return; }
                        const vp = document.getElementById('viewport');
                        if (!vp) return;
                        const rect = vp.getBoundingClientRect();
                        // Scroll so the target center is visible in the viewport
                        const scrollX = b.cx - rect.left - rect.width / 2;
                        const scrollY = b.cy - rect.top - rect.height / 2;
                        vp.scrollBy({ left: scrollX, top: scrollY, behavior: 'smooth' });
                        await new Promise(r => setTimeout(r, 400));
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
                            case 'scroll':      await this._doScroll(action.target, gt.targets); break;
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

            // Auto-fit: set actual CSS width/height on #viewport.
            // The canvas is lowest-priority — header and debug panel keep their
            // natural size, viewport shrinks to fit whatever is left.
            // No CSS transform — positions are actual pixel coordinates.
            if (!window.__autoFitInstalled) {
                window.__autoFitInstalled = true;
                document.documentElement.style.overflow = 'hidden';
                document.body.style.overflow = 'hidden';
                document.body.style.margin = '0';
                document.body.style.padding = '0';

                let rafId = 0;
                function scheduleAutoFit() {
                    cancelAnimationFrame(rafId);
                    rafId = requestAnimationFrame(autoFit);
                }

                function autoFit() {
                    const vp = document.getElementById('viewport');
                    if (!vp) return;
                    if (vp.dataset.fixed) return;
                    const parent = vp.parentElement;
                    if (!parent) return;

                    // Lock parent to exactly one screen
                    parent.style.height = '100vh';
                    parent.style.maxHeight = '100vh';
                    parent.style.boxSizing = 'border-box';

                    // Temporarily collapse viewport to measure sibling heights
                    const savedH = vp.style.height;
                    const savedW = vp.style.width;
                    vp.style.height = '0';
                    vp.style.width = '0';

                    let othersH = 0;
                    for (const child of parent.children) {
                        if (child === vp) continue;
                        const cs = getComputedStyle(child);
                        othersH += child.getBoundingClientRect().height
                                 + (parseFloat(cs.marginTop) || 0)
                                 + (parseFloat(cs.marginBottom) || 0);
                    }

                    // Restore while we compute
                    vp.style.height = savedH;
                    vp.style.width = savedW;

                    const ps = getComputedStyle(parent);
                    const pt = parseFloat(ps.paddingTop) || 0;
                    const pb = parseFloat(ps.paddingBottom) || 0;
                    const pl = parseFloat(ps.paddingLeft) || 0;
                    const pr = parseFloat(ps.paddingRight) || 0;

                    let availH = Math.floor(Math.max(window.innerHeight - pt - pb - othersH, 150));
                    let availW = Math.floor(Math.max(window.innerWidth - pl - pr, 200));

                    // Randomize viewport size for VLM training diversity.
                    // Scale factor: 25%–100% of available space.
                    // Re-rolls on route change and on scoring (via __rerollVpScale).
                    // Deterministic from seed+generation when seeded, otherwise random.
                    // Read: window.__vpScale   Set: window.__setVpScale(0.5)
                    if (location.pathname !== window.__lastVpPath) {
                        window.__lastVpPath = location.pathname;
                        window.__vpScale = null;
                        window.__vpScaleGen = (window.__vpScaleGen || 0) + 1;
                    }
                    if (window.__vpScale == null) {
                        const gen = window.__vpScaleGen || 0;
                        if (window.__playgroundSeed != null) {
                            let s = ((window.__playgroundSeed + gen * 0x517cc1b7) ^ 0x9e3779b9) >>> 0;
                            s = Math.imul(s ^ (s >>> 16), 0x45d9f3b) >>> 0;
                            s = Math.imul(s ^ (s >>> 16), 0x45d9f3b) >>> 0;
                            s = (s ^ (s >>> 16)) >>> 0;
                            window.__vpScale = 0.25 + (s / 0xFFFFFFFF) * 0.75;
                        } else {
                            window.__vpScale = 0.25 + Math.random() * 0.75;
                        }
                    }
                    availW = Math.floor(Math.max(availW * window.__vpScale, 200));
                    availH = Math.floor(Math.max(availH * window.__vpScale, 150));

                    // Set actual dimensions — no transform
                    vp.style.width = availW + 'px';
                    vp.style.height = availH + 'px';

                    // Store for Rust/WASM to read via js_sys::Reflect
                    window.__vpW = availW;
                    window.__vpH = availH;

                    // Match ground truth panel width
                    const gt = document.getElementById('ground-truth');
                    if (gt) gt.style.width = availW + 'px';

                    // Post-layout fixup: center any elements that ended up
                    // off-screen (e.g. positioned before viewport scale applied).
                    // Skip when viewport is scrollable — off-screen elements are
                    // intentional and reachable via scroll.
                    requestAnimationFrame(() => {
                        if (!vp || !vp.parentElement) return;
                        const vpOverflow = getComputedStyle(vp).overflow;
                        if (vpOverflow === 'auto' || vpOverflow === 'scroll') return;
                        const vr = vp.getBoundingClientRect();
                        for (const child of vp.querySelectorAll(':scope > div[style*=\"position: absolute\"]')) {
                            const cr = child.getBoundingClientRect();
                            if (cr.right > vr.right + 2 || cr.bottom > vr.bottom + 2) {
                                const cw = cr.width, ch = cr.height;
                                child.style.left = Math.max(8, Math.floor((vr.width - cw) / 2)) + 'px';
                                child.style.top = Math.max(8, Math.floor((vr.height - ch) / 2)) + 'px';
                            }
                        }
                    });
                }

                window.__setVpScale = (s) => {
                    window.__vpScale = Math.max(0.25, Math.min(1.0, s));
                    scheduleAutoFit();
                };
                window.__rerollVpScale = () => {
                    window.__vpScaleGen = (window.__vpScaleGen || 0) + 1;
                    window.__vpScale = null;
                    scheduleAutoFit();
                };

                scheduleAutoFit();
                window.addEventListener('resize', scheduleAutoFit);
                // Re-run after route changes (Dioxus updates DOM async)
                const observer = new MutationObserver(() => scheduleAutoFit());
                observer.observe(
                    document.getElementById('main') || document.body,
                    { childList: true, subtree: true }
                );
                window.__autoFitObserver = observer;
                window.__autoFitSchedule = scheduleAutoFit;
            }

            // Cleanup on page unload — release all JS references to help
            // the browser GC the old WASM instance faster on refresh.
            if (!window.__unloadCleanupInstalled) {
                window.__unloadCleanupInstalled = true;
                window.addEventListener('pagehide', () => {
                    // Remove event listeners
                    if (window.__playgroundCleanupListeners) {
                        for (const { target, type, fn } of window.__playgroundCleanupListeners) {
                            target.removeEventListener(type, fn, true);
                        }
                        window.__playgroundCleanupListeners = null;
                    }
                    // Disconnect MutationObserver
                    if (window.__autoFitObserver) {
                        window.__autoFitObserver.disconnect();
                        window.__autoFitObserver = null;
                    }
                    // Remove resize listener
                    if (window.__autoFitSchedule) {
                        window.removeEventListener('resize', window.__autoFitSchedule);
                        window.__autoFitSchedule = null;
                    }
                    // Clear solver bar
                    const bar = document.getElementById('__solver-bar');
                    if (bar) bar.remove();
                    // Reset guards so fresh init on next load
                    window.__playgroundListeners = false;
                    window.__autoFitInstalled = false;
                    window.__debugModeInstalled = false;
                });
            }

            // Debug mode: control ground-truth visibility via localStorage + data attribute
            if (!window.__debugModeInstalled) {
                window.__debugModeInstalled = true;
                const key = 'playgroundDebug';
                const style = document.createElement('style');
                style.textContent = '#ground-truth{display:none;} #__solver-bar{display:none;} body[data-debug="true"] #ground-truth{display:block;} body[data-debug="true"] #__solver-bar{display:flex;}';
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
