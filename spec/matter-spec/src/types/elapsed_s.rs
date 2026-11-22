//! Matter `elapsed_s` semantic type (elapsed time in seconds).

use core::time::Duration;

#[cfg(feature = "uom")]
use uom::si::{f64::Time as UomTime, time::second};

use crate::model::MatterStatus;

super::define_matter_scalar_type! {
    /// Elapsed duration in seconds (`s`, wire type `u32`).
    ///
    /// Represents a relative duration rather than an epoch timestamp. Construct explicitly
    /// with [`ElapsedS::from_raw`] (no `From<u32>` is implemented).
    pub struct ElapsedS(u32);
    attr_variant = U32;
    display_fmt = "{} s";
}

impl ElapsedS {
    /// Convert to milliseconds.
    #[inline]
    #[must_use]
    pub fn as_millis(self) -> u64 {
        u64::from(self.0).saturating_mul(1_000)
    }

    /// Convert to a [`core::time::Duration`].
    #[inline]
    #[must_use]
    pub fn as_duration(self) -> Duration {
        Duration::from_secs(u64::from(self.0))
    }

    /// Construct from a [`core::time::Duration`], saturating at `u32::MAX` seconds.
    #[inline]
    #[must_use]
    pub fn from_duration_saturating(duration: Duration) -> Self {
        Self(u32::try_from(duration.as_secs()).unwrap_or(u32::MAX))
    }
}

impl From<ElapsedS> for Duration {
    #[inline]
    fn from(val: ElapsedS) -> Self {
        val.as_duration()
    }
}

impl TryFrom<Duration> for ElapsedS {
    type Error = MatterStatus;

    #[inline]
    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        u32::try_from(duration.as_secs())
            .map(Self)
            .map_err(|_| MatterStatus::ConstraintError)
    }
}

#[cfg(feature = "uom")]
impl From<ElapsedS> for UomTime {
    #[inline]
    fn from(val: ElapsedS) -> Self {
        Self::new::<second>(f64::from(val.0))
    }
}

#[cfg(feature = "uom")]
impl TryFrom<UomTime> for ElapsedS {
    type Error = MatterStatus;

    #[inline]
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "no_std float scaling"
    )]
    fn try_from(val: UomTime) -> Result<Self, Self::Error> {
        let secs = val.get::<second>();
        if secs.is_nan() || secs < 0.0 || secs > f64::from(u32::MAX) {
            return Err(MatterStatus::ConstraintError);
        }
        Ok(Self((secs + 0.5).clamp(0.0, f64::from(u32::MAX)) as u32))
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn elapsed_s_round_trip_and_duration() {
        let e = ElapsedS::from_raw(42);
        assert_eq!(e.raw(), 42);
        assert_eq!(e.as_millis(), 42_000);
        assert_eq!(e.as_duration(), Duration::from_secs(42));
        assert_eq!(ElapsedS::from_duration_saturating(Duration::from_secs(42)), e);
        assert_eq!(
            ElapsedS::from_duration_saturating(Duration::from_secs(u64::MAX)),
            ElapsedS::from_raw(u32::MAX)
        );
        assert_eq!(e.to_string(), "42 s");

        let av = AttributeValue::from(e);
        assert_eq!(ElapsedS::try_from(&av), Ok(e));
        assert_eq!(
            ElapsedS::try_from(&AttributeValue::U16(42)),
            Err(MatterStatus::InvalidDataType)
        );
        assert_eq!(Duration::from(e), Duration::from_secs(42));
        assert_eq!(ElapsedS::try_from(Duration::from_secs(42)), Ok(e));
    }

    #[cfg(feature = "uom")]
    #[test]
    fn elapsed_s_uom_round_trip() {
        let e = ElapsedS::from_raw(42);
        let u = UomTime::from(e);
        assert_eq!(ElapsedS::try_from(u), Ok(e));
        assert_eq!(
            ElapsedS::try_from(UomTime::new::<second>(-1.0)),
            Err(MatterStatus::ConstraintError)
        );
    }
}
