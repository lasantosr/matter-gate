//! Matter `energy_mvarh` semantic type (millivolt-ampere-reactive-hours).

super::define_matter_milli_unit! {
    /// Reactive energy in millivolt-ampere-reactive-hours (`mVARh`, wire type `i64`).
    ///
    /// Construct explicitly with [`EnergyMvarh::from_raw`] or [`EnergyMvarh::from_varh`]
    /// (no `From<i64>` is implemented).
    pub struct EnergyMvarh(i64);
    display_fmt = "{} mVARh";
    from_unit = from_varh;
    to_unit = to_varh;
    unit_doc = "volt-ampere-reactive-hours";
    milli_doc = "`mVARh`";
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn energy_mvarh_round_trip() {
        let e = EnergyMvarh::from_raw(5_000);
        assert_eq!(e.raw(), 5_000);
        assert!((e.to_varh() - 5.0).abs() < 1e-6);
        assert_eq!(e.to_string(), "5000 mVARh");

        let av = AttributeValue::from(e);
        assert_eq!(EnergyMvarh::try_from(&av), Ok(e));
        assert_eq!(
            EnergyMvarh::try_from(&AttributeValue::U64(5_000)),
            Err(MatterStatus::InvalidDataType)
        );
    }
}
