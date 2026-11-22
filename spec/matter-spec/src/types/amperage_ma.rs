//! Matter `amperage_ma` semantic type (milliamperes).

#[cfg(feature = "uom")]
use uom::si::{electric_current::ampere, f64::ElectricCurrent};

super::define_matter_milli_unit! {
    /// Electric current in milliamperes (`mA`, wire type `i64`).
    ///
    /// Construct explicitly with [`AmperageMa::from_raw`] or [`AmperageMa::from_amps`]
    /// (no `From<i64>` is implemented).
    ///
    /// ```rust
    /// use matter_spec::types::AmperageMa;
    ///
    /// let a = AmperageMa::from_amps(1.5);
    /// assert_eq!(a.raw(), 1500);
    /// ```
    pub struct AmperageMa(i64);
    display_fmt = "{} mA";
    from_unit = from_amps;
    to_unit = to_amps;
    unit_doc = "amperes";
    milli_doc = "milliampere";
}

super::impl_matter_uom_unit!(AmperageMa, ElectricCurrent, ampere, from_amps, to_amps);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amps_round_trip() {
        let a = AmperageMa::from_amps(1.5);
        assert_eq!(a.raw(), 1500);
        assert!((a.to_amps() - 1.5).abs() < 1e-6);
    }

    #[cfg(feature = "uom")]
    #[test]
    fn uom_round_trip() {
        let a = AmperageMa::from_amps(1.5);
        let u = ElectricCurrent::from(a);
        assert_eq!(AmperageMa::from(u), a);
    }
}
