//! Sampler - random page generation from the pool

use rand::Rng;

use crate::pool::{ElementPool, DesignSnippet, ElementKind};
use crate::primitives::{Angle, Animation, Opacity, Position, Scale};
use super::placed::PlacedElement;

/// Generates random page layouts by sampling from the pool
pub struct Sampler;

impl Sampler {
    /// Pick a random snippet from the pool
    pub fn pick_snippet<R: Rng>(rng: &mut R, pool: &ElementPool) -> Option<DesignSnippet> {
        let all = pool.all();
        if all.is_empty() {
            return None;
        }
        let idx = rng.random_range(0..all.len());
        Some(all[idx].clone())
    }

    /// Pick a random snippet of a specific kind
    pub fn pick_kind<R: Rng>(
        rng: &mut R,
        pool: &ElementPool,
        kind: ElementKind,
    ) -> Option<DesignSnippet> {
        let snippets = pool.get(kind);
        if snippets.is_empty() {
            return None;
        }
        let idx = rng.random_range(0..snippets.len());
        Some(snippets[idx].clone())
    }

    /// Sample a random position that keeps the element on-canvas
    pub fn random_position<R: Rng>(rng: &mut R, elem_w: f32, elem_h: f32) -> Position {
        let margin = 40.0;
        let max_x = (Position::VIEWPORT - elem_w - margin).max(margin);
        let max_y = (Position::VIEWPORT - elem_h - margin).max(margin);
        let x = rng.random_range(margin..=max_x);
        let y = rng.random_range(margin..=max_y);
        Position::new(x, y)
    }

    /// Sample a random scale from vocabulary
    pub fn random_scale<R: Rng>(rng: &mut R) -> Scale {
        let vocab = Scale::VOCABULARY;
        vocab[rng.random_range(0..vocab.len())]
    }

    /// Sample a random angle from vocabulary
    pub fn random_angle<R: Rng>(rng: &mut R) -> Angle {
        let vocab = Angle::VOCABULARY;
        vocab[rng.random_range(0..vocab.len())]
    }

    /// Sample a random opacity from vocabulary
    pub fn random_opacity<R: Rng>(rng: &mut R) -> Opacity {
        let vocab = Opacity::ALL;
        vocab[rng.random_range(0..vocab.len())]
    }

    /// Sample a random animation from vocabulary (weighted toward None)
    pub fn random_animation<R: Rng>(rng: &mut R) -> Animation {
        let vocab = Animation::VOCABULARY;
        vocab[rng.random_range(0..vocab.len())]
    }

    /// Generate a fully randomized placed element
    pub fn random_placed<R: Rng>(rng: &mut R, pool: &ElementPool) -> Option<PlacedElement> {
        let snippet = Self::pick_snippet(rng, pool)?;
        let scale = Self::random_scale(rng);
        let pos = Self::random_position(
            rng,
            snippet.approx_width * scale.value(),
            snippet.approx_height * scale.value(),
        );
        let angle = Self::random_angle(rng);
        let opacity = Self::random_opacity(rng);
        let animation = Self::random_animation(rng);

        Some(
            PlacedElement::new(snippet, pos)
                .with_scale(scale)
                .with_angle(angle)
                .with_opacity(opacity)
                .with_animation(animation),
        )
    }

    /// Generate a page with N random elements, avoiding overlaps
    pub fn random_page<R: Rng>(
        rng: &mut R,
        pool: &ElementPool,
        count: usize,
    ) -> Vec<PlacedElement> {
        let mut elements = Vec::with_capacity(count);
        let mut attempts = 0;
        let max_attempts = count * 10;

        while elements.len() < count && attempts < max_attempts {
            attempts += 1;
            if let Some(placed) = Self::random_placed(rng, pool) {
                // Simple overlap check
                let (x, y, w, h) = placed.bounds();
                let overlaps = elements.iter().any(|existing: &PlacedElement| {
                    let (ex, ey, ew, eh) = existing.bounds();
                    x < ex + ew && x + w > ex && y < ey + eh && y + h > ey
                });

                if !overlaps {
                    elements.push(placed);
                }
            }
        }

        elements
    }
}
