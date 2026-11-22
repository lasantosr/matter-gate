//! Matter `voltage_mv` semantic type (millivolts).

#[cfg(feature = "uom")]
use uom::si::{electric_potential::volt, f64::ElectricPotential};

super::define_matter_milli_unit! {
    /// Electric potential in millivolts (`mV`, wire type `i64`).
    ///
    /// Construct explicitly with [`VoltageMv::from_raw`] or [`VoltageMv::from_volts`]
    /// (no `From<i64>` is implemented).
    ///
    /// ```rust
    /// use matter_spec::types::VoltageMv;
    ///
    /// let v = VoltageMv::from_volts(230.0);
    /// assert_eq!(v.raw(), 230_000);
    /// ```
    pub struct VoltageMv(i64);
    display_fmt = "{} mV";
    from_unit = from_volts;
    to_unit = to_volts;
    unit_doc = "volts";
    milli_doc = "millivolt";
}

super::impl_matter_uom_unit!(VoltageMv, ElectricPotential, volt, from_volts, to_volts);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volts_round_trip() {
        let v = VoltageMv::from_volts(230.0);
        assert_eq!(v.raw(), 230_000);
        assert!((v.to_volts() - 230.0).abs() < 1e-6);
    }

    #[cfg(feature = "uom")]
    #[test]
    fn uom_round_trip() {
        let v = VoltageMv::from_volts(230.0);
        let u = ElectricPotential::from(v);
        assert_eq!(VoltageMv::from(u), v);
    }
}
