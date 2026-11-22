//! Matter `power_mva` semantic type (millivolt-amperes).

super::define_matter_milli_unit! {
    /// Apparent power in millivolt-amperes (`mVA`, wire type `i64`).
    ///
    /// Construct explicitly with [`PowerMva::from_raw`] or [`PowerMva::from_va`]
    /// (no `From<i64>` is implemented).
    pub struct PowerMva(i64);
    display_fmt = "{} mVA";
    from_unit = from_va;
    to_unit = to_va;
    unit_doc = "volt-amperes";
    milli_doc = "millivolt-ampere";
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn power_mva_round_trip() {
        let p = PowerMva::from_raw(2_500);
        assert_eq!(p.raw(), 2_500);
        assert!((p.to_va() - 2.5).abs() < 1e-6);
        assert_eq!(p.to_string(), "2500 mVA");

        let av = AttributeValue::from(p);
        assert_eq!(PowerMva::try_from(&av), Ok(p));
        assert_eq!(
            PowerMva::try_from(&AttributeValue::U64(2_500)),
            Err(MatterStatus::InvalidDataType)
        );
    }
}
