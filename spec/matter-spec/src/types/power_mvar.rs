//! Matter `power_mvar` semantic type (millivolt-amperes reactive).

super::define_matter_milli_unit! {
    /// Reactive power in millivolt-amperes reactive (`mVAR`, wire type `i64`).
    ///
    /// Construct explicitly with [`PowerMvar::from_raw`] or [`PowerMvar::from_var`]
    /// (no `From<i64>` is implemented).
    pub struct PowerMvar(i64);
    display_fmt = "{} mVAR";
    from_unit = from_var;
    to_unit = to_var;
    unit_doc = "volt-amperes reactive";
    milli_doc = "`mVAR`";
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn power_mvar_round_trip() {
        let p = PowerMvar::from_raw(2_500);
        assert_eq!(p.raw(), 2_500);
        assert!((p.to_var() - 2.5).abs() < 1e-6);
        assert_eq!(p.to_string(), "2500 mVAR");

        let av = AttributeValue::from(p);
        assert_eq!(PowerMvar::try_from(&av), Ok(p));
        assert_eq!(
            PowerMvar::try_from(&AttributeValue::U64(2_500)),
            Err(MatterStatus::InvalidDataType)
        );
    }
}
