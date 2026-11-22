//! Matter `energy_mwh` semantic type (milliwatt-hours).

#[cfg(feature = "uom")]
use uom::si::{energy::watt_hour, f64::Energy};

super::define_matter_milli_unit! {
    /// Active energy in milliwatt-hours (`mWh`, wire type `i64`).
    ///
    /// Construct explicitly with [`EnergyMwh::from_raw`], [`EnergyMwh::from_wh`], or
    /// [`EnergyMwh::from_kwh`] (no `From<i64>` is implemented).
    ///
    /// ```rust
    /// use matter_spec::types::EnergyMwh;
    ///
    /// let e = EnergyMwh::from_kwh(1.5);
    /// assert_eq!(e.raw(), 1_500_000);
    /// ```
    pub struct EnergyMwh(i64);
    display_fmt = "{} mWh";
    from_unit = from_wh;
    to_unit = to_wh;
    unit_doc = "watt-hours";
    milli_doc = "milliwatt-hour";
}

impl EnergyMwh {
    /// Construct from kilowatt-hours, rounding to the nearest milliwatt-hour.
    #[inline]
    #[must_use]
    pub fn from_kwh(kwh: f64) -> Self {
        Self::from_wh(kwh * 1000.0)
    }

    /// Convert to kilowatt-hours.
    #[inline]
    #[expect(clippy::as_conversions, clippy::cast_precision_loss, reason = "no_std float scaling")]
    #[must_use]
    pub fn to_kwh(self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
}

super::impl_matter_uom_unit!(EnergyMwh, Energy, watt_hour, from_wh, to_wh);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_round_trip() {
        let e = EnergyMwh::from_kwh(1.5);
        assert_eq!(e.raw(), 1_500_000);
        assert!((e.to_kwh() - 1.5).abs() < 1e-6);
        assert!((e.to_wh() - 1500.0).abs() < 1e-6);
    }

    #[cfg(feature = "uom")]
    #[test]
    fn uom_round_trip() {
        let e = EnergyMwh::from_kwh(1.5);
        let u = Energy::from(e);
        assert_eq!(EnergyMwh::from(u), e);
    }
}
