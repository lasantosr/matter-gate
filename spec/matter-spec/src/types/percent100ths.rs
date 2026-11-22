//! Matter `percent100ths` semantic type (`0.01%` units, `0..=10000`).

use core::fmt;

#[cfg(feature = "uom")]
use uom::si::{f64::Ratio, ratio::ratio};

super::define_matter_scalar_type! {
    /// Percentage in hundredths of a percent (`0.01%` units, `0..=10000`, wire type `u16`).
    ///
    /// Construct explicitly with [`Percent100ths::from_raw`] or [`Percent100ths::from_percent`]
    /// (no `From<u16>` is implemented).
    ///
    /// ```rust
    /// use matter_spec::types::Percent100ths;
    ///
    /// let p = Percent100ths::from_percent(50.25);
    /// assert_eq!(p.raw(), 5025);
    /// assert!((p.to_percent() - 50.25).abs() < 0.01);
    /// ```
    pub struct Percent100ths(u16);
    attr_variant = U16;
}

impl Percent100ths {
    /// Construct from a floating-point percentage, clamped to `0.0..=100.0`.
    #[inline]
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "no_std float scaling"
    )]
    #[must_use]
    pub fn from_percent(percent: f32) -> Self {
        if percent.is_nan() {
            return Self(0);
        }
        let clamped = percent.clamp(0.0, 100.0);
        Self((clamped * 100.0 + 0.5) as u16)
    }

    /// Convert to a floating-point percentage (`0.0..=100.0`).
    #[inline]
    #[must_use]
    pub fn to_percent(self) -> f32 {
        f32::from(self.0) / 100.0
    }

    /// Construct from a unit fraction (`0.0..=1.0`), clamped and rounded to `0.01%` units.
    #[inline]
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "no_std float scaling"
    )]
    #[must_use]
    pub fn from_fraction(fraction: f64) -> Self {
        if fraction.is_nan() {
            return Self(0);
        }
        let clamped = fraction.clamp(0.0, 1.0);
        Self((clamped * 10000.0 + 0.5) as u16)
    }

    /// Convert to a unit fraction (`0.0..=1.0`, or higher for extended raw values).
    #[inline]
    #[must_use]
    pub fn to_fraction(self) -> f64 {
        f64::from(self.0) / 10000.0
    }
}

super::impl_matter_uom_unit!(Percent100ths, Ratio, ratio, from_fraction, to_fraction);

impl fmt::Display for Percent100ths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}%", self.to_percent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn percent100ths_clamping_and_round_trip() {
        assert_eq!(Percent100ths::from_percent(120.0).raw(), 10000);
        assert_eq!(Percent100ths::from_percent(-5.0).raw(), 0);
        let p = Percent100ths::from_percent(50.25);
        assert_eq!(p.raw(), 5025);
        assert!((p.to_percent() - 50.25).abs() < 0.01);
        assert_eq!(Percent100ths::from_fraction(0.5025).raw(), 5025);
        assert_eq!(Percent100ths::from_fraction(-0.1).raw(), 0);
        assert_eq!(Percent100ths::from_fraction(1.2).raw(), 10000);
        assert!((p.to_fraction() - 0.5025).abs() < 1e-6);
        let av = AttributeValue::from(p);
        assert_eq!(Percent100ths::try_from(&av), Ok(p));
        assert_eq!(
            Percent100ths::try_from(&AttributeValue::U8(50)),
            Err(MatterStatus::InvalidDataType)
        );
    }

    #[cfg(feature = "uom")]
    #[test]
    fn uom_round_trip() {
        let p = Percent100ths::from_percent(50.25);
        let u = Ratio::from(p);
        assert_eq!(Percent100ths::from(u), p);
    }
}
