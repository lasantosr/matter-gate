//! Matter `posix_ms` semantic type (milliseconds since Unix epoch).

use core::time::Duration;
#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};
#[cfg(feature = "time")]
use time::OffsetDateTime;

use super::{EpochS, EpochUs};
use crate::model::MatterStatus;

super::define_matter_scalar_type! {
    /// Milliseconds since the POSIX/Unix epoch (`1970-01-01T00:00:00Z`, wire type `u64`).
    ///
    /// Unlike [`EpochS`] / [`EpochUs`] (Matter epoch `2000-01-01T00:00:00Z`),
    /// `PosixMs` uses the standard Unix epoch. Construct explicitly with [`PosixMs::from_raw`]
    /// (no `From<u64>` is implemented).
    pub struct PosixMs(u64);
    attr_variant = U64;
    display_fmt = "{} ms";
}

impl PosixMs {
    /// Convert to whole seconds.
    #[inline]
    #[must_use]
    pub const fn as_secs(self) -> u64 {
        self.0.wrapping_div(1_000)
    }

    /// Convert to a [`core::time::Duration`] since the POSIX/Unix epoch.
    #[inline]
    #[must_use]
    pub const fn as_duration(self) -> Duration {
        Duration::from_millis(self.0)
    }

    /// Construct from a [`core::time::Duration`] since the POSIX/Unix epoch, saturating at `u64::MAX` ms.
    #[inline]
    #[must_use]
    pub fn from_duration_saturating(duration: Duration) -> Self {
        Self(u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
    }

    /// Convert to [`EpochS`] (truncating sub-second milliseconds), returning `None` if before the Matter
    /// epoch or overflowing `u32::MAX`.
    #[inline]
    #[must_use]
    pub fn to_epoch_s(self) -> Option<EpochS> {
        EpochS::from_unix_secs(self.as_secs())
    }

    /// Convert to [`EpochUs`], returning `None` if before the Matter epoch or on `u64` overflow.
    #[inline]
    #[must_use]
    pub const fn to_epoch_us(self) -> Option<EpochUs> {
        match self.0.checked_mul(1_000) {
            Some(unix_us) => EpochUs::from_unix_micros(unix_us),
            None => None,
        }
    }

    /// Convert to [`std::time::SystemTime`], returning `None` if the target platform cannot represent the timestamp.
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn to_system_time(self) -> Option<SystemTime> {
        UNIX_EPOCH.checked_add(Duration::from_millis(self.0))
    }

    /// Construct from [`std::time::SystemTime`], returning `None` if before the Unix epoch or overflowing `u64`.
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn from_system_time(time: SystemTime) -> Option<Self> {
        let elapsed = time.duration_since(UNIX_EPOCH).ok()?;
        u64::try_from(elapsed.as_millis()).ok().map(Self)
    }

    /// Convert to a UTC [`chrono::DateTime<chrono::Utc>`], returning `None` on overflow.
    #[cfg(feature = "chrono")]
    #[inline]
    #[must_use]
    pub fn to_chrono_utc(self) -> Option<DateTime<Utc>> {
        let millis = i64::try_from(self.0).ok()?;
        DateTime::from_timestamp_millis(millis)
    }

    /// Construct from a UTC [`chrono::DateTime<chrono::Utc>`], returning `None` if before the Unix epoch.
    #[cfg(feature = "chrono")]
    #[inline]
    #[must_use]
    pub fn from_chrono_utc(dt: DateTime<Utc>) -> Option<Self> {
        u64::try_from(dt.timestamp_millis()).ok().map(Self)
    }

    /// Convert to a [`time::OffsetDateTime`] in UTC, returning `None` on overflow.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn to_time_offset_datetime(self) -> Option<OffsetDateTime> {
        let unix_ns = i128::from(self.0).checked_mul(1_000_000)?;
        OffsetDateTime::from_unix_timestamp_nanos(unix_ns).ok()
    }

    /// Construct from a [`time::OffsetDateTime`], returning `None` if before the Unix epoch or overflowing `u64`.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn from_time_offset_datetime(dt: OffsetDateTime) -> Option<Self> {
        u64::try_from(dt.unix_timestamp_nanos().div_euclid(1_000_000))
            .ok()
            .map(Self)
    }
}

super::impl_matter_epoch_conversions!(PosixMs, u64, as_millis);

impl TryFrom<PosixMs> for EpochS {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: PosixMs) -> Result<Self, Self::Error> {
        val.to_epoch_s().ok_or(MatterStatus::ConstraintError)
    }
}

impl TryFrom<PosixMs> for EpochUs {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: PosixMs) -> Result<Self, Self::Error> {
        val.to_epoch_us().ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn posix_ms_round_trip_and_duration() {
        let t = PosixMs::from_raw(2_500);
        assert_eq!(t.raw(), 2_500);
        assert_eq!(t.as_secs(), 2);
        assert_eq!(t.as_duration(), Duration::from_millis(2_500));
        assert_eq!(PosixMs::from_duration_saturating(Duration::from_millis(2_500)), t);
        assert_eq!(t.to_string(), "2500 ms");

        let av = AttributeValue::from(t);
        assert_eq!(PosixMs::try_from(&av), Ok(t));
        assert_eq!(
            PosixMs::try_from(&AttributeValue::U32(2_500)),
            Err(MatterStatus::InvalidDataType)
        );

        // Duration & cross-type conversions
        assert_eq!(Duration::from(t), Duration::from_millis(2_500));
        assert_eq!(PosixMs::try_from(Duration::from_millis(2_500)), Ok(t));
        assert_eq!(EpochS::try_from(t), Err(MatterStatus::ConstraintError));
        assert_eq!(EpochUs::try_from(t), Err(MatterStatus::ConstraintError));

        let after_matter = PosixMs::from_raw(EpochS::UNIX_EPOCH_OFFSET_SECS.saturating_add(42).saturating_mul(1_000));
        assert_eq!(EpochS::try_from(after_matter), Ok(EpochS::from_raw(42)));
        assert_eq!(EpochUs::try_from(after_matter), Ok(EpochUs::from_raw(42_000_000)));
    }

    #[cfg(feature = "std")]
    #[test]
    fn posix_ms_system_time_round_trip() {
        let t = PosixMs::from_raw(2_500);
        let st = SystemTime::try_from(t).expect("valid SystemTime");
        assert_eq!(PosixMs::try_from(st), Ok(t));
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn posix_ms_chrono_round_trip() {
        let t = PosixMs::from_raw(2_500);
        let dt = DateTime::<Utc>::try_from(t).expect("valid chrono DateTime");
        assert_eq!(PosixMs::try_from(dt), Ok(t));
    }

    #[cfg(feature = "time")]
    #[test]
    fn posix_ms_time_round_trip() {
        let t = PosixMs::from_raw(2_500);
        let dt = OffsetDateTime::try_from(t).expect("valid time OffsetDateTime");
        assert_eq!(PosixMs::try_from(dt), Ok(t));
    }
}
