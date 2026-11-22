//! Matter `epoch_us` semantic type (microseconds since Matter epoch).

use core::time::Duration;
#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};
#[cfg(feature = "time")]
use time::OffsetDateTime;

use super::{EpochS, PosixMs};
use crate::model::MatterStatus;

super::define_matter_scalar_type! {
    /// Microseconds since the Matter epoch (`2000-01-01T00:00:00Z`, wire type `u64`).
    ///
    /// Unlike [`PosixMs`] (Unix epoch `1970-01-01T00:00:00Z`), Matter epoch
    /// timestamps start at midnight UTC on January 1, 2000. Construct explicitly with
    /// [`EpochUs::from_raw`] (no `From<u64>` is implemented).
    ///
    /// ```rust
    /// use matter_spec::types::EpochUs;
    ///
    /// let t = EpochUs::from_raw(2_500_000);
    /// assert_eq!(t.as_millis(), 2_500);
    /// assert_eq!(t.as_secs(), 2);
    /// ```
    pub struct EpochUs(u64);
    attr_variant = U64;
    display_fmt = "{} us";
}

impl EpochUs {
    /// Offset in microseconds between the Unix epoch (`1970-01-01T00:00:00Z`) and the Matter epoch
    /// (`2000-01-01T00:00:00Z`).
    pub const UNIX_EPOCH_OFFSET_MICROS: u64 = 946_684_800_000_000;

    /// Convert to microseconds since the Unix epoch (`1970-01-01T00:00:00Z`), or `None` on `u64` overflow.
    #[inline]
    #[must_use]
    pub const fn to_unix_micros(self) -> Option<u64> {
        self.0.checked_add(Self::UNIX_EPOCH_OFFSET_MICROS)
    }

    /// Construct from microseconds since the Unix epoch (`1970-01-01T00:00:00Z`), returning `None` if before the Matter
    /// epoch.
    #[inline]
    #[must_use]
    pub const fn from_unix_micros(unix_micros: u64) -> Option<Self> {
        match unix_micros.checked_sub(Self::UNIX_EPOCH_OFFSET_MICROS) {
            Some(us) => Some(Self(us)),
            None => None,
        }
    }

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

    /// Convert to a [`core::time::Duration`] since the Matter epoch.
    #[inline]
    #[must_use]
    pub const fn as_duration(self) -> Duration {
        Duration::from_micros(self.0)
    }

    /// Construct from a [`core::time::Duration`] since the Matter epoch, saturating at `u64::MAX` us.
    #[inline]
    #[must_use]
    pub fn from_duration_saturating(duration: Duration) -> Self {
        Self(u64::try_from(duration.as_micros()).unwrap_or(u64::MAX))
    }

    /// Convert to [`EpochS`] (truncating sub-second microseconds), returning `None` if `> u32::MAX` s.
    #[inline]
    #[must_use]
    pub fn to_epoch_s(self) -> Option<EpochS> {
        u32::try_from(self.as_secs()).ok().map(EpochS::from_raw)
    }

    /// Convert to [`PosixMs`] (truncating sub-millisecond microseconds), returning `None` on overflow.
    #[inline]
    #[must_use]
    pub const fn to_posix_ms(self) -> Option<PosixMs> {
        match self.to_unix_micros() {
            Some(us) => Some(PosixMs::from_raw(us.wrapping_div(1_000))),
            None => None,
        }
    }

    /// Convert to [`std::time::SystemTime`], returning `None` if the timestamp overflows.
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn to_system_time(self) -> Option<SystemTime> {
        UNIX_EPOCH.checked_add(Duration::from_micros(self.to_unix_micros()?))
    }

    /// Construct from [`std::time::SystemTime`], returning `None` if before the Matter epoch or overflowing `u64`.
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn from_system_time(time: SystemTime) -> Option<Self> {
        let elapsed = time.duration_since(UNIX_EPOCH).ok()?;
        let unix_us = u64::try_from(elapsed.as_micros()).ok()?;
        Self::from_unix_micros(unix_us)
    }

    /// Convert to a UTC [`chrono::DateTime<chrono::Utc>`], returning `None` on overflow.
    #[cfg(feature = "chrono")]
    #[inline]
    #[must_use]
    pub fn to_chrono_utc(self) -> Option<DateTime<Utc>> {
        let unix_us = i64::try_from(self.to_unix_micros()?).ok()?;
        DateTime::from_timestamp_micros(unix_us)
    }

    /// Construct from a UTC [`chrono::DateTime<chrono::Utc>`], returning `None` if out of range.
    #[cfg(feature = "chrono")]
    #[inline]
    #[must_use]
    pub fn from_chrono_utc(dt: DateTime<Utc>) -> Option<Self> {
        let unix_us = u64::try_from(dt.timestamp_micros()).ok()?;
        Self::from_unix_micros(unix_us)
    }

    /// Convert to a [`time::OffsetDateTime`] in UTC, returning `None` on overflow.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn to_time_offset_datetime(self) -> Option<OffsetDateTime> {
        let unix_ns = i128::from(self.to_unix_micros()?).checked_mul(1_000)?;
        OffsetDateTime::from_unix_timestamp_nanos(unix_ns).ok()
    }

    /// Construct from a [`time::OffsetDateTime`], returning `None` if out of range.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn from_time_offset_datetime(dt: OffsetDateTime) -> Option<Self> {
        let unix_us = u64::try_from(dt.unix_timestamp_nanos().div_euclid(1_000)).ok()?;
        Self::from_unix_micros(unix_us)
    }
}

super::impl_matter_epoch_conversions!(EpochUs, u64, as_micros);

impl TryFrom<EpochUs> for EpochS {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: EpochUs) -> Result<Self, Self::Error> {
        val.to_epoch_s().ok_or(MatterStatus::ConstraintError)
    }
}

impl TryFrom<EpochUs> for PosixMs {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: EpochUs) -> Result<Self, Self::Error> {
        val.to_posix_ms().ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn epoch_us_round_trip() {
        let t = EpochUs::from_raw(2_500_000);
        assert_eq!(t.as_millis(), 2_500);
        assert_eq!(t.as_secs(), 2);
        let av = AttributeValue::from(t);
        assert_eq!(EpochUs::try_from(&av), Ok(t));
        assert_eq!(EpochUs::try_from(av), Ok(t));
        assert_eq!(
            EpochUs::try_from(&AttributeValue::U32(1)),
            Err(MatterStatus::InvalidDataType)
        );

        // Unix epoch conversions
        let unix_us = t.to_unix_micros().expect("to_unix_micros");
        assert_eq!(unix_us, EpochUs::UNIX_EPOCH_OFFSET_MICROS.saturating_add(2_500_000));
        assert_eq!(EpochUs::from_unix_micros(unix_us), Some(t));
        assert_eq!(
            EpochUs::from_unix_micros(EpochUs::UNIX_EPOCH_OFFSET_MICROS.saturating_sub(1)),
            None
        );
        assert_eq!(EpochUs::from_raw(u64::MAX).to_unix_micros(), None);

        // Duration & cross-type conversions
        assert_eq!(Duration::from(t), Duration::from_millis(2_500));
        assert_eq!(EpochUs::try_from(Duration::from_millis(2_500)), Ok(t));
        assert_eq!(EpochS::try_from(t), Ok(EpochS::from_raw(2)));
        let posix = PosixMs::try_from(t).expect("valid PosixMs");
        assert_eq!(
            posix,
            PosixMs::from_raw(
                EpochS::UNIX_EPOCH_OFFSET_SECS
                    .saturating_mul(1_000)
                    .saturating_add(2_500)
            )
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn epoch_us_system_time_round_trip() {
        let t = EpochUs::from_raw(2_500_000);
        let st = SystemTime::try_from(t).expect("valid SystemTime");
        assert_eq!(EpochUs::try_from(st), Ok(t));
        assert_eq!(EpochUs::try_from(UNIX_EPOCH), Err(MatterStatus::ConstraintError));
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn epoch_us_chrono_round_trip() {
        let t = EpochUs::from_raw(2_500_000);
        let dt = DateTime::<Utc>::try_from(t).expect("valid chrono DateTime");
        assert_eq!(EpochUs::try_from(dt), Ok(t));
        assert_eq!(
            EpochUs::try_from(DateTime::<Utc>::UNIX_EPOCH),
            Err(MatterStatus::ConstraintError)
        );
    }

    #[cfg(feature = "time")]
    #[test]
    fn epoch_us_time_round_trip() {
        let t = EpochUs::from_raw(2_500_000);
        let dt = OffsetDateTime::try_from(t).expect("valid time OffsetDateTime");
        assert_eq!(EpochUs::try_from(dt), Ok(t));
        assert_eq!(
            EpochUs::try_from(OffsetDateTime::UNIX_EPOCH),
            Err(MatterStatus::ConstraintError)
        );
    }
}
