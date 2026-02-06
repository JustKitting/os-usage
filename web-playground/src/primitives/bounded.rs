//! Bounded - macro for creating range-constrained numeric types
//!
//! Generates types with compile-time validation for const contexts.
//! Operators clamp to bounds instead of panicking.

/// Creates a bounded f32 type with min/max constraints.
///
/// # Example
/// ```ignore
/// bounded_f32!(Opacity, 0.0, 1.0);
/// let o = Opacity::new(0.5);
/// let o2 = o + 0.7;  // Opacity(1.0) - clamped to max
/// ```
macro_rules! bounded_f32 {
    ($name:ident, $min:expr, $max:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name(f32);

        impl $name {
            pub const MIN: f32 = $min;
            pub const MAX: f32 = $max;

            #[track_caller]
            pub const fn new(value: f32) -> Self {
                if value < Self::MIN || value > Self::MAX {
                    panic!(concat!(
                        stringify!($name),
                        " value out of bounds [",
                        stringify!($min),
                        ", ",
                        stringify!($max),
                        "]"
                    ));
                }
                Self(value)
            }

            pub fn clamped(value: f32) -> Self {
                Self(value.clamp(Self::MIN, Self::MAX))
            }

            pub const fn value(&self) -> f32 {
                self.0
            }

            pub const fn is_min(&self) -> bool {
                self.0 == Self::MIN
            }

            pub const fn is_max(&self) -> bool {
                self.0 == Self::MAX
            }

            /// Ratio within the range [0.0, 1.0]
            pub const fn ratio(&self) -> f32 {
                (self.0 - Self::MIN) / (Self::MAX - Self::MIN)
            }

            /// Lerp between min and max
            pub fn lerp(t: f32) -> Self {
                Self::clamped(Self::MIN + t * (Self::MAX - Self::MIN))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(Self::MIN)
            }
        }

        impl std::ops::Add<f32> for $name {
            type Output = Self;
            fn add(self, rhs: f32) -> Self::Output {
                Self::clamped(self.0 + rhs)
            }
        }

        impl std::ops::Sub<f32> for $name {
            type Output = Self;
            fn sub(self, rhs: f32) -> Self::Output {
                Self::clamped(self.0 - rhs)
            }
        }

        impl std::ops::Mul<f32> for $name {
            type Output = Self;
            fn mul(self, rhs: f32) -> Self::Output {
                Self::clamped(self.0 * rhs)
            }
        }

        impl std::ops::Div<f32> for $name {
            type Output = Self;
            fn div(self, rhs: f32) -> Self::Output {
                Self::clamped(self.0 / rhs)
            }
        }
    };
}

pub(crate) use bounded_f32;

#[cfg(test)]
mod tests {
    use super::*;

    bounded_f32!(TestPercent, 0.0, 100.0);
    bounded_f32!(TestNorm, 0.0, 1.0);

    const FIFTY: TestPercent = TestPercent::new(50.0);

    #[test]
    fn bounded_const_valid() {
        assert_eq!(FIFTY.value(), 50.0);
        assert_eq!(FIFTY.ratio(), 0.5);
    }

    #[test]
    fn bounded_clamps_on_overflow() {
        let p = TestPercent::new(90.0);
        assert_eq!((p + 20.0).value(), 100.0);
    }

    #[test]
    fn bounded_clamps_on_underflow() {
        let p = TestPercent::new(10.0);
        assert_eq!((p - 20.0).value(), 0.0);
    }

    #[test]
    fn bounded_lerp() {
        let mid = TestNorm::lerp(0.5);
        assert_eq!(mid.value(), 0.5);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn bounded_rejects_invalid() {
        let _ = TestPercent::new(101.0);
    }
}
