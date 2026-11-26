//! Matter `systime_ms` semantic type (system uptime in milliseconds).

use core::time::Duration;

use super::SystimeUs;
use crate::model::MatterStatus;

super::define_matter_scalar_type! {
    /// Monotonic system uptime in milliseconds (`ms`, wire type `u64`).
    ///
    /// Measures time elapsed since device boot rather than wall-clock epoch time. Construct
    /// explicitly with [`SystimeMs::from_raw`] (no `From<u64>` is implemented).
    pub struct SystimeMs(u64);
    attr_variant = U64;
    display_fmt = "{} ms";
}

impl SystimeMs {
    /// Convert to whole seconds.
    #[inline]
    #[must_use]
    pub const fn as_secs(self) -> u64 {
        self.0.wrapping_div(1_000)
    }

    /// Convert to a [`core::time::Duration`].
    #[inline]
    #[must_use]
    pub const fn as_duration(self) -> Duration {
        Duration::from_millis(self.0)
    }

    /// Construct from a [`core::time::Duration`], saturating at `u64::MAX` ms.
    #[inline]
    #[must_use]
    pub fn from_duration_saturating(duration: Duration) -> Self {
        Self(u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
    }

    /// Convert to [`SystimeUs`], returning `None` on `u64` overflow.
    #[inline]
    #[must_use]
    pub const fn to_systime_us(self) -> Option<SystimeUs> {
        match self.0.checked_mul(1_000) {
            Some(us) => Some(SystimeUs::from_raw(us)),
            None => None,
        }
    }
}

impl From<SystimeMs> for Duration {
    #[inline]
    fn from(val: SystimeMs) -> Self {
        val.as_duration()
    }
}

impl TryFrom<Duration> for SystimeMs {
    type Error = MatterStatus;

    #[inline]
    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        u64::try_from(duration.as_millis())
            .map(Self)
            .map_err(|_| MatterStatus::ConstraintError)
    }
}

impl TryFrom<SystimeMs> for SystimeUs {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: SystimeMs) -> Result<Self, Self::Error> {
        val.to_systime_us().ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn systime_ms_round_trip_and_duration() {
        let t = SystimeMs::from_raw(3_500);
        assert_eq!(t.raw(), 3_500);
        assert_eq!(t.as_secs(), 3);
        assert_eq!(t.as_duration(), Duration::from_millis(3_500));
        assert_eq!(SystimeMs::from_duration_saturating(Duration::from_millis(3_500)), t);
        assert_eq!(t.to_string(), "3500 ms");

        let av = AttributeValue::from(t);
        assert_eq!(SystimeMs::try_from(&av), Ok(t));
        assert_eq!(SystimeMs::try_from(&AttributeValue::U32(3_500)), Ok(t));
        assert_eq!(
            SystimeMs::try_from(&AttributeValue::I64(3_500)),
            Err(MatterStatus::InvalidDataType)
        );
        assert_eq!(Duration::from(t), Duration::from_millis(3_500));
        assert_eq!(SystimeMs::try_from(Duration::from_millis(3_500)), Ok(t));
        assert_eq!(SystimeUs::try_from(t), Ok(SystimeUs::from_raw(3_500_000)));
        assert_eq!(
            SystimeUs::try_from(SystimeMs::from_raw(u64::MAX)),
            Err(MatterStatus::ConstraintError)
        );
    }
}
