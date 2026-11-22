//! Matter `energy_mvah` semantic type (millivolt-ampere-hours).

super::define_matter_milli_unit! {
    /// Apparent energy in millivolt-ampere-hours (`mVAh`, wire type `i64`).
    ///
    /// Construct explicitly with [`EnergyMvah::from_raw`] or [`EnergyMvah::from_vah`]
    /// (no `From<i64>` is implemented).
    pub struct EnergyMvah(i64);
    display_fmt = "{} mVAh";
    from_unit = from_vah;
    to_unit = to_vah;
    unit_doc = "volt-ampere-hours";
    milli_doc = "`mVAh`";
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn energy_mvah_round_trip() {
        let e = EnergyMvah::from_raw(5_000);
        assert_eq!(e.raw(), 5_000);
        assert!((e.to_vah() - 5.0).abs() < 1e-6);
        assert_eq!(e.to_string(), "5000 mVAh");

        let av = AttributeValue::from(e);
        assert_eq!(EnergyMvah::try_from(&av), Ok(e));
        assert_eq!(
            EnergyMvah::try_from(&AttributeValue::U64(5_000)),
            Err(MatterStatus::InvalidDataType)
        );
    }
}
