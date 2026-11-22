//! Matter `percent` semantic type (`u8`, `0..=100` or `0..=254`).

#[cfg(feature = "uom")]
use uom::si::{f64::Ratio, ratio::ratio};

super::define_matter_scalar_type! {
    /// Whole-percentage value (`1%` units, `0..=100` or `0..=254` for extended Matter ranges, wire type `u8`).
    ///
    /// Construct explicitly with [`Percent::from_raw`] or [`Percent::from_percent`]
    /// (no `From<u8>` is implemented).
    ///
    /// ```rust
    /// use matter_spec::types::Percent;
    ///
    /// let p = Percent::from_percent(75);
    /// assert_eq!(p.raw(), 75);
    /// assert_eq!(p.to_percent(), 75);
    /// ```
    pub struct Percent(u8);
    attr_variant = U8;
    display_fmt = "{}%";
}

impl Percent {
    /// Construct from a percentage clamped to `0..=100`.
    #[inline]
    #[must_use]
    pub const fn from_percent(percent: u8) -> Self {
        if percent > 100 { Self(100) } else { Self(percent) }
    }

    /// Return the percentage value.
    #[inline]
    #[must_use]
    pub const fn to_percent(self) -> u8 {
        self.0
    }

    /// Construct from a unit fraction (`0.0..=1.0`), clamped and rounded to the nearest integer percent.
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
        Self((clamped * 100.0 + 0.5) as u8)
    }

    /// Convert to a unit fraction (`0.0..=1.0`, or higher for extended raw values).
    #[inline]
    #[must_use]
    pub fn to_fraction(self) -> f64 {
        f64::from(self.0) / 100.0
    }
}

super::impl_matter_uom_unit!(Percent, Ratio, ratio, from_fraction, to_fraction);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn percent_clamping_and_round_trip() {
        assert_eq!(Percent::from_percent(150).raw(), 100);
        assert_eq!(Percent::from_percent(42).to_percent(), 42);
        assert_eq!(Percent::from_fraction(0.755).raw(), 76);
        assert_eq!(Percent::from_fraction(-0.5).raw(), 0);
        assert_eq!(Percent::from_fraction(1.5).raw(), 100);
        assert!((Percent::from_percent(50).to_fraction() - 0.5).abs() < 1e-6);
        let p = Percent::from_raw(75);
        let av = AttributeValue::from(p);
        assert_eq!(Percent::try_from(&av), Ok(p));
        assert_eq!(
            Percent::try_from(&AttributeValue::U16(75)),
            Err(MatterStatus::InvalidDataType)
        );
    }

    #[cfg(feature = "uom")]
    #[test]
    fn uom_round_trip() {
        let p = Percent::from_percent(75);
        let u = Ratio::from(p);
        assert_eq!(Percent::from(u), p);
    }
}
