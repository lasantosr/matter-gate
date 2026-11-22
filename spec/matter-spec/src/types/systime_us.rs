//! Matter `systime_us` semantic type (system uptime in microseconds).

use core::time::Duration;

use super::SystimeMs;
use crate::model::MatterStatus;

super::define_matter_scalar_type! {
    /// Monotonic system uptime in microseconds (`us`, wire type `u64`).
    ///
    /// Measures time elapsed since device boot rather than wall-clock epoch time. Construct
    /// explicitly with [`SystimeUs::from_raw`] (no `From<u64>` is implemented).
    pub struct SystimeUs(u64);
    attr_variant = U64;
    display_fmt = "{} us";
}

impl SystimeUs {
    /// Convert to whole milliseconds.
    #[inline]
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0.wrapping_div(1_000)
    }

    /// Convert to whole seconds.
    #[inline]
    #[must_use]
    pub const fn as_secs(self) -> u64 {
        self.0.wrapping_div(1_000_000)
    }

    /// Convert to a [`core::time::Duration`].
    #[inline]
    #[must_use]
    pub const fn as_duration(self) -> Duration {
        Duration::from_micros(self.0)
    }

    /// Construct from a [`core::time::Duration`], saturating at `u64::MAX` us.
    #[inline]
    #[must_use]
    pub fn from_duration_saturating(duration: Duration) -> Self {
        Self(u64::try_from(duration.as_micros()).unwrap_or(u64::MAX))
    }

    /// Convert to [`SystimeMs`], truncating sub-millisecond microseconds.
    #[inline]
    #[must_use]
    pub const fn to_systime_ms(self) -> SystimeMs {
        SystimeMs::from_raw(self.as_millis())
    }
}

impl From<SystimeUs> for Duration {
    #[inline]
    fn from(val: SystimeUs) -> Self {
        val.as_duration()
    }
}

impl TryFrom<Duration> for SystimeUs {
    type Error = MatterStatus;

    #[inline]
    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        u64::try_from(duration.as_micros())
            .map(Self)
            .map_err(|_| MatterStatus::ConstraintError)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn systime_us_round_trip_and_duration() {
        let t = SystimeUs::from_raw(3_500_000);
        assert_eq!(t.raw(), 3_500_000);
        assert_eq!(t.as_millis(), 3_500);
        assert_eq!(t.as_secs(), 3);
        assert_eq!(t.as_duration(), Duration::from_millis(3_500));
        assert_eq!(SystimeUs::from_duration_saturating(Duration::from_millis(3_500)), t);
        assert_eq!(t.to_string(), "3500000 us");

        let av = AttributeValue::from(t);
        assert_eq!(SystimeUs::try_from(&av), Ok(t));
        assert_eq!(
            SystimeUs::try_from(&AttributeValue::U32(3_500)),
            Err(MatterStatus::InvalidDataType)
        );
        assert_eq!(Duration::from(t), Duration::from_millis(3_500));
        assert_eq!(SystimeUs::try_from(Duration::from_millis(3_500)), Ok(t));
        assert_eq!(t.to_systime_ms(), SystimeMs::from_raw(3_500));
    }
}
