//! Transform - combines a pool snippet with transform primitives
//!
//! A PlacedElement is a snippet positioned on the canvas with
//! scale, rotation, opacity applied. The transform is rendered
//! as CSS on a wrapper div; the snippet HTML goes inside via
//! dangerous_inner_html.

pub mod placed;
pub mod sampler;

pub use placed::PlacedElement;
pub use sampler::Sampler;
