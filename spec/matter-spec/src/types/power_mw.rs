//! Matter `power_mw` semantic type (milliwatts).

#[cfg(feature = "uom")]
use uom::si::{f64::Power, power::watt};

super::define_matter_milli_unit! {
    /// Active power in milliwatts (`mW`, wire type `i64`).
    ///
    /// Construct explicitly with [`PowerMw::from_raw`] or [`PowerMw::from_watts`]
    /// (no `From<i64>` is implemented).
    ///
    /// ```rust
    /// use matter_spec::types::PowerMw;
    ///
    /// let p = PowerMw::from_watts(12.5);
    /// assert_eq!(p.raw(), 12_500);
    /// ```
    pub struct PowerMw(i64);
    display_fmt = "{} mW";
    from_unit = from_watts;
    to_unit = to_watts;
    unit_doc = "watts";
    milli_doc = "milliwatt";
}

super::impl_matter_uom_unit!(PowerMw, Power, watt, from_watts, to_watts);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn watts_round_trip() {
        let p = PowerMw::from_watts(12.5);
        assert_eq!(p.raw(), 12_500);
        assert!((p.to_watts() - 12.5).abs() < 1e-6);
        let av = AttributeValue::from(p);
        assert_eq!(PowerMw::try_from(&av), Ok(p));
        assert_eq!(
            PowerMw::try_from(&AttributeValue::U64(12_500)),
            Err(MatterStatus::InvalidDataType)
        );
    }

    #[cfg(feature = "uom")]
    #[test]
    fn uom_round_trip() {
        let p = PowerMw::from_watts(12.5);
        let u = Power::from(p);
        assert_eq!(PowerMw::from(u), p);
    }
}
