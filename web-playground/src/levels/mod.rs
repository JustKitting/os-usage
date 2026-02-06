mod custom_select;
mod ground_truth;
mod level1;
mod level2;
mod level3;
mod level4;
mod level5;
mod level6;
mod level7;
mod level8;
mod level9;
mod level10;
mod level11;
mod level12;
mod level13;
mod level14;
mod level15;
mod level16;
mod level17;
mod level18;
mod level19;
mod level20;
mod level21;
mod level22;
mod level23;
mod level24;
mod level25;
mod level26;
mod level27;
mod level_scroll;

pub(crate) use custom_select::CustomSelect;
pub(crate) use ground_truth::GroundTruth;
pub use level1::Level1;
pub use level2::Level2;
pub use level3::Level3;
pub use level4::Level4;
pub use level5::Level5;
pub use level6::Level6;
pub use level7::Level7;
pub use level8::Level8;
pub use level9::Level9;
pub use level10::Level10;
pub use level11::Level11;
pub use level12::Level12;
pub use level13::Level13;
pub use level14::Level14;
pub use level15::Level15;
pub use level16::Level16;
pub use level17::Level17;
pub use level18::Level18;
pub use level19::Level19;
pub use level20::Level20;
pub use level21::Level21;
pub use level22::Level22;
pub use level23::Level23;
pub use level24::Level24;
pub use level25::Level25;
pub use level26::Level26;
pub use level27::Level27;
pub use level_scroll::LevelScroll;

use rand::SeedableRng;
use rand::Rng;
use rand::rngs::SmallRng;
use std::cell::{Cell, RefCell};
use js_sys::Reflect;
use web_sys::wasm_bindgen::JsValue;

use crate::pool::{ElementPool, ElementKind};
use crate::primitives::{Position, viewport_size};
use crate::transform::{PlacedElement, Sampler};

const CANVAS_COLORS: &[&str] = &[
    "#1a1a2e", "#2d1b69", "#0f3460", "#1b4332", "#4a1942",
    "#1a5276", "#6c3483", "#117a65", "#7b241c", "#1f618d",
    "#d4ac0d", "#2e86c1", "#a93226", "#148f77", "#7d3c98",
    "#d35400", "#1abc9c", "#8e44ad", "#2980b9", "#27ae60",
    "#c0392b", "#16a085", "#2c3e50", "#e74c3c", "#3498db", "#ffffff",
];

pub fn random_canvas_bg() -> String {
    reroll_viewport();
    let mut rng = fresh_rng();
    CANVAS_COLORS[rng.random_range(0..CANVAS_COLORS.len())].to_string()
}

/// Re-randomize the viewport scale factor for the next round.
fn reroll_viewport() {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = js_sys::eval("window.__rerollVpScale && window.__rerollVpScale()");
    }
}

pub fn fresh_rng() -> SmallRng {
    if let Some(seed) = current_seed() {
        let counter = SEED_COUNTER.with(|c| {
            let value = c.get();
            c.set(value + 1);
            value
        });
        SmallRng::from_seed(expand_seed(seed, counter))
    } else {
        let mut buf = [0u8; 32];
        getrandom::fill(&mut buf).expect("getrandom");
        SmallRng::from_seed(buf)
    }
}

thread_local! {
    static SEED: RefCell<Option<u64>> = RefCell::new(None);
    static SEED_COUNTER: Cell<u64> = Cell::new(0);
}

fn current_seed() -> Option<u64> {
    SEED.with(|seed| {
        if seed.borrow().is_none() {
            let next = seed_from_window();
            *seed.borrow_mut() = next;
        }
        *seed.borrow()
    })
}

fn seed_from_window() -> Option<u64> {
    let window = web_sys::window()?;
    let value = Reflect::get(&window, &JsValue::from_str("__playgroundSeed")).ok()?;
    let number = value.as_f64()?;
    if number.is_finite() && number >= 0.0 {
        Some(number as u64)
    } else {
        None
    }
}

fn expand_seed(seed: u64, counter: u64) -> [u8; 32] {
    let mut state = seed ^ counter.wrapping_mul(0x9e3779b97f4a7c15);
    let mut out = [0u8; 32];
    for chunk in out.chunks_exact_mut(8) {
        let value = splitmix64(&mut state).to_le_bytes();
        chunk.copy_from_slice(&value);
    }
    out
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// Pick a random position that keeps an element of size `(w, h)` fully inside
/// the viewport.  Padding shrinks automatically when the element is large
/// relative to the viewport, and the element is centred when it barely fits.
pub fn safe_position(rng: &mut impl Rng, w: f32, h: f32, pad: f32) -> (f32, f32) {
    let (vp_w, vp_h) = viewport_size();
    safe_position_in(rng, w, h, pad, vp_w, vp_h)
}

/// Like `safe_position` but positions within a custom canvas size instead of
/// the viewport.  Use with a canvas larger than the viewport for scrollable
/// levels so elements may land off-screen.
pub fn safe_position_in(rng: &mut impl Rng, w: f32, h: f32, pad: f32, canvas_w: f32, canvas_h: f32) -> (f32, f32) {
    let max_x = (canvas_w - w).max(0.0);
    let max_y = (canvas_h - h).max(0.0);
    let pad_x = pad.min(max_x / 2.0);
    let pad_y = pad.min(max_y / 2.0);
    let x = if max_x < 1.0 { 0.0 } else { rng.random_range(pad_x..(max_x - pad_x).max(pad_x + 1.0)) };
    let y = if max_y < 1.0 { 0.0 } else { rng.random_range(pad_y..(max_y - pad_y).max(pad_y + 1.0)) };
    (x, y)
}

pub fn random_element(pool: &ElementPool, kind: ElementKind) -> PlacedElement {
    let mut rng = fresh_rng();
    let snippet = Sampler::pick_kind(&mut rng, pool, kind)
        .expect("pool has this kind");

    let (vp_w, vp_h) = viewport_size();
    let pad = 150.0f32.min(vp_w.min(vp_h) / 4.0);
    let (x, y) = safe_position(&mut rng, snippet.approx_width, snippet.approx_height, pad);
    let pos = Position::new(x, y);

    PlacedElement::new(snippet, pos)
}

/// Generate the standard viewport div style with dynamic sizing.
/// When `scrollable` is true the viewport gets `overflow: auto` so content
/// that extends past the edges produces scrollbars instead of being clipped.
pub fn viewport_style(bg: &str, scrollable: bool) -> String {
    let (vp_w, vp_h) = viewport_size();
    let overflow = if scrollable { "auto" } else { "hidden" };
    format!(
        "width: {vp_w}px; height: {vp_h}px; background: {bg}; position: relative; border: 1px solid #2a2a4a; overflow: {overflow}; transition: background 0.4s;",
    )
}


pub fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, 11) => "th",
        (2, 12) => "th",
        (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}
