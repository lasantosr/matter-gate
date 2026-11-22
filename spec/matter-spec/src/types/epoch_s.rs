//! Matter `epoch_s` semantic type (seconds since Matter epoch).

use core::time::Duration;
#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};
#[cfg(feature = "time")]
use time::OffsetDateTime;

use super::{EpochUs, PosixMs};
use crate::model::MatterStatus;

super::define_matter_scalar_type! {
    /// Seconds since the Matter epoch (`2000-01-01T00:00:00Z`, wire type `u32`).
    ///
    /// Unlike [`PosixMs`] (Unix epoch `1970-01-01T00:00:00Z`), Matter epoch
    /// timestamps start at midnight UTC on January 1, 2000. Construct explicitly with
    /// [`EpochS::from_raw`] (no `From<u32>` is implemented).
    pub struct EpochS(u32);
    attr_variant = U32;
    display_fmt = "{} s";
}

impl EpochS {
    /// Offset in seconds between the Unix epoch (`1970-01-01T00:00:00Z`) and the Matter epoch (`2000-01-01T00:00:00Z`).
    pub const UNIX_EPOCH_OFFSET_SECS: u64 = 946_684_800;

    /// Convert to seconds since the Unix epoch (`1970-01-01T00:00:00Z`).
    #[inline]
    #[must_use]
    #[expect(clippy::as_conversions, reason = "const widening from u32 to u64")]
    pub const fn to_unix_secs(self) -> u64 {
        (self.0 as u64).saturating_add(Self::UNIX_EPOCH_OFFSET_SECS)
    }

    /// Construct from seconds since the Unix epoch (`1970-01-01T00:00:00Z`), returning `None` if before the Matter
    /// epoch or overflowing `u32::MAX`.
    #[inline]
    #[must_use]
    pub fn from_unix_secs(unix_secs: u64) -> Option<Self> {
        let matter_secs = unix_secs.checked_sub(Self::UNIX_EPOCH_OFFSET_SECS)?;
        u32::try_from(matter_secs).ok().map(Self)
    }

    /// Convert to milliseconds.
    #[inline]
    #[must_use]
    pub fn as_millis(self) -> u64 {
        u64::from(self.0).saturating_mul(1_000)
    }

    /// Convert to a [`core::time::Duration`] since the Matter epoch.
    #[inline]
    #[must_use]
    pub fn as_duration(self) -> Duration {
        Duration::from_secs(u64::from(self.0))
    }

    /// Construct from a [`core::time::Duration`] since the Matter epoch, saturating at `u32::MAX` seconds.
    #[inline]
    #[must_use]
    pub fn from_duration_saturating(duration: Duration) -> Self {
        Self(u32::try_from(duration.as_secs()).unwrap_or(u32::MAX))
    }

    /// Convert losslessly to [`EpochUs`].
    #[inline]
    #[must_use]
    #[expect(clippy::as_conversions, reason = "const widening from u32 to u64")]
    pub const fn to_epoch_us(self) -> EpochUs {
        EpochUs::from_raw((self.0 as u64).saturating_mul(1_000_000))
    }

    /// Convert losslessly to [`PosixMs`].
    #[inline]
    #[must_use]
    pub const fn to_posix_ms(self) -> PosixMs {
        PosixMs::from_raw(self.to_unix_secs().saturating_mul(1_000))
    }

    /// Convert to [`std::time::SystemTime`], returning `None` if the target platform cannot represent the timestamp.
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn to_system_time(self) -> Option<SystemTime> {
        UNIX_EPOCH.checked_add(Duration::from_secs(self.to_unix_secs()))
    }

    /// Construct from [`std::time::SystemTime`], returning `None` if before the Matter epoch or overflowing `u32::MAX`.
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn from_system_time(time: SystemTime) -> Option<Self> {
        let elapsed = time.duration_since(UNIX_EPOCH).ok()?;
        Self::from_unix_secs(elapsed.as_secs())
    }

    /// Convert to a UTC [`chrono::DateTime<chrono::Utc>`].
    #[cfg(feature = "chrono")]
    #[inline]
    #[must_use]
    pub fn to_chrono_utc(self) -> Option<DateTime<Utc>> {
        let secs = i64::try_from(self.to_unix_secs()).ok()?;
        DateTime::from_timestamp(secs, 0)
    }

    /// Construct from a UTC [`chrono::DateTime<chrono::Utc>`], returning `None` if out of range.
    #[cfg(feature = "chrono")]
    #[inline]
    #[must_use]
    pub fn from_chrono_utc(dt: DateTime<Utc>) -> Option<Self> {
        let unix_secs = u64::try_from(dt.timestamp()).ok()?;
        Self::from_unix_secs(unix_secs)
    }

    /// Convert to a [`time::OffsetDateTime`] in UTC.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn to_time_offset_datetime(self) -> Option<OffsetDateTime> {
        let secs = i64::try_from(self.to_unix_secs()).ok()?;
        OffsetDateTime::from_unix_timestamp(secs).ok()
    }

    /// Construct from a [`time::OffsetDateTime`], returning `None` if out of range.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn from_time_offset_datetime(dt: OffsetDateTime) -> Option<Self> {
        let unix_secs = u64::try_from(dt.unix_timestamp()).ok()?;
        Self::from_unix_secs(unix_secs)
    }
}

super::impl_matter_epoch_conversions!(EpochS, u32, as_secs);

impl From<EpochS> for EpochUs {
    #[inline]
    fn from(val: EpochS) -> Self {
        val.to_epoch_us()
    }
}

impl From<EpochS> for PosixMs {
    #[inline]
    fn from(val: EpochS) -> Self {
        val.to_posix_ms()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn epoch_s_round_trip_and_duration() {
        let e = EpochS::from_raw(1_000);
        assert_eq!(e.raw(), 1_000);
        assert_eq!(e.as_millis(), 1_000_000);
        assert_eq!(e.as_duration(), Duration::from_secs(1_000));
        assert_eq!(EpochS::from_duration_saturating(Duration::from_secs(1_000)), e);
        assert_eq!(e.to_string(), "1000 s");

        let av = AttributeValue::from(e);
        assert_eq!(EpochS::try_from(&av), Ok(e));
        assert_eq!(EpochS::try_from(av), Ok(e));
        assert_eq!(
            EpochS::try_from(&AttributeValue::U64(1_000)),
            Err(MatterStatus::InvalidDataType)
        );

        // Unix epoch conversions
        assert_eq!(e.to_unix_secs(), EpochS::UNIX_EPOCH_OFFSET_SECS.saturating_add(1_000));
        assert_eq!(EpochS::from_unix_secs(e.to_unix_secs()), Some(e));
        assert_eq!(
            EpochS::from_unix_secs(EpochS::UNIX_EPOCH_OFFSET_SECS.saturating_sub(1)),
            None
        );
        assert_eq!(
            EpochS::from_unix_secs(
                EpochS::UNIX_EPOCH_OFFSET_SECS
                    .saturating_add(u64::from(u32::MAX))
                    .saturating_add(1)
            ),
            None
        );

        // Duration From / TryFrom & cross-type conversions
        assert_eq!(Duration::from(e), Duration::from_secs(1_000));
        assert_eq!(EpochS::try_from(Duration::from_secs(1_000)), Ok(e));
        assert_eq!(
            EpochS::try_from(Duration::from_secs(u64::MAX)),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(EpochUs::from(e), EpochUs::from_raw(1_000_000_000));
        assert_eq!(
            PosixMs::from(e),
            PosixMs::from_raw(
                EpochS::UNIX_EPOCH_OFFSET_SECS
                    .saturating_add(1_000)
                    .saturating_mul(1_000)
            )
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn epoch_s_system_time_round_trip() {
        let e = EpochS::from_raw(1_000);
        let st = SystemTime::try_from(e).expect("valid SystemTime");
        assert_eq!(EpochS::try_from(st), Ok(e));
        assert_eq!(EpochS::try_from(UNIX_EPOCH), Err(MatterStatus::ConstraintError));
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn epoch_s_chrono_round_trip() {
        let e = EpochS::from_raw(1_000);
        let dt = DateTime::<Utc>::try_from(e).expect("valid chrono DateTime");
        assert_eq!(EpochS::try_from(dt), Ok(e));
        assert_eq!(
            EpochS::try_from(DateTime::<Utc>::UNIX_EPOCH),
            Err(MatterStatus::ConstraintError)
        );
    }

    #[cfg(feature = "time")]
    #[test]
    fn epoch_s_time_round_trip() {
        let e = EpochS::from_raw(1_000);
        let dt = OffsetDateTime::try_from(e).expect("valid time OffsetDateTime");
        assert_eq!(EpochS::try_from(dt), Ok(e));
        assert_eq!(
            EpochS::try_from(OffsetDateTime::UNIX_EPOCH),
            Err(MatterStatus::ConstraintError)
        );
    }
}
