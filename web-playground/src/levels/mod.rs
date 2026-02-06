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

use rand::SeedableRng;
use rand::Rng;
use rand::rngs::SmallRng;

use crate::pool::{ElementPool, ElementKind};
use crate::primitives::Position;
use crate::transform::{PlacedElement, Sampler};

const CANVAS_COLORS: &[&str] = &[
    "#1a1a2e", "#2d1b69", "#0f3460", "#1b4332", "#4a1942",
    "#1a5276", "#6c3483", "#117a65", "#7b241c", "#1f618d",
    "#d4ac0d", "#2e86c1", "#a93226", "#148f77", "#7d3c98",
    "#d35400", "#1abc9c", "#8e44ad", "#2980b9", "#27ae60",
    "#c0392b", "#16a085", "#2c3e50", "#e74c3c", "#3498db", "#ffffff",
];

pub fn random_canvas_bg() -> String {
    let mut rng = fresh_rng();
    CANVAS_COLORS[rng.random_range(0..CANVAS_COLORS.len())].to_string()
}

pub fn fresh_rng() -> SmallRng {
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf).expect("getrandom");
    SmallRng::from_seed(buf)
}

pub fn random_element(pool: &ElementPool, kind: ElementKind) -> PlacedElement {
    let mut rng = fresh_rng();
    let snippet = Sampler::pick_kind(&mut rng, pool, kind)
        .expect("pool has this kind");

    let pad = 150.0;
    let x = rng.random_range(pad..(Position::VIEWPORT - pad));
    let y = rng.random_range(pad..(Position::VIEWPORT - pad));
    let pos = Position::new(x, y);

    PlacedElement::new(snippet, pos)
}

pub fn describe_position(x: f32, y: f32, w: f32, h: f32) -> &'static str {
    Position::new(x + w / 2.0, y + h / 2.0).describe()
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
