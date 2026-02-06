//! Transform primitives - applied to wrapper divs around pool elements
//!
//! Each primitive:
//! - `to_css()` → CSS property string
//! - `describe()` → English for ground truth labels
//! - `VOCABULARY` → closed set for random sampling

#[macro_use]
pub mod bounded;
pub mod angle;
pub mod animation;
pub mod opacity;
pub mod position;
pub mod scale;

pub use angle::Angle;
pub use animation::Animation;
pub use opacity::Opacity;
pub use position::{Position, viewport_size};
pub use scale::Scale;
