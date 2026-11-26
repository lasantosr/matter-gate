//! Matter `tod` semantic type (time of day).

use core::fmt;

#[cfg(feature = "jiff")]
use jiff::civil::Time as JiffTime;
#[cfg(feature = "time")]
use time::Time as TimeLibTime;

use crate::{
    model::{AttributeValue, MatterStatus},
    traits::decode_unsigned,
};

/// Matter Time of Day (`tod`) encoded as a packed `u32` wall-clock bitfield.
///
/// Per CSA Matter Core Specification §7.19.2.14, the 4 octets pack:
/// - `hours` (`0..=23`, `0xFF` = unspecified)
/// - `minutes` (`0..=59`, `0xFF` = unspecified)
/// - `seconds` (`0..=59`, `0xFF` = unspecified)
/// - `hundredths` (`0..=99`, `0xFF` = unspecified)
///
/// Construct explicitly with [`TimeOfDay::from_raw`] (no `From<u32>` is implemented).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(transparent))]
#[repr(transparent)]
pub struct TimeOfDay(u32);

impl Default for TimeOfDay {
    fn default() -> Self {
        Self(0xFFFF_FFFF)
    }
}

impl TimeOfDay {
    /// Sentinel octet indicating an unspecified time-of-day field (`0xFF`).
    pub const UNSPECIFIED_OCTET: u8 = 0xFF;

    /// Construct from a raw packed `u32` time-of-day value.
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the raw packed `u32` time-of-day value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Construct a [`TimeOfDay`] from wall-clock components (`None` encodes [`Self::UNSPECIFIED_OCTET`]).
    ///
    /// Returns `None` if any provided component is outside its valid Matter range:
    /// - `hours`: `0..=23`
    /// - `minutes`: `0..=59`
    /// - `seconds`: `0..=59`
    /// - `hundredths`: `0..=99`
    #[inline]
    #[must_use]
    pub const fn new(
        hours: Option<u8>,
        minutes: Option<u8>,
        seconds: Option<u8>,
        hundredths: Option<u8>,
    ) -> Option<Self> {
        let h_byte = match hours {
            Some(h) if h <= 23 => h,
            Some(_) => return None,
            None => Self::UNSPECIFIED_OCTET,
        };
        let m_byte = match minutes {
            Some(m) if m <= 59 => m,
            Some(_) => return None,
            None => Self::UNSPECIFIED_OCTET,
        };
        let s_byte = match seconds {
            Some(s) if s <= 59 => s,
            Some(_) => return None,
            None => Self::UNSPECIFIED_OCTET,
        };
        let hs_byte = match hundredths {
            Some(hs) if hs <= 99 => hs,
            Some(_) => return None,
            None => Self::UNSPECIFIED_OCTET,
        };
        Some(Self(u32::from_be_bytes([h_byte, m_byte, s_byte, hs_byte])))
    }

    /// Return the hours component (`0..=23`), or `None` if unspecified or out of range.
    #[inline]
    #[must_use]
    pub const fn hours(self) -> Option<u8> {
        let [h, _, _, _] = self.0.to_be_bytes();
        if h <= 23 { Some(h) } else { None }
    }

    /// Return the minutes component (`0..=59`), or `None` if unspecified or out of range.
    #[inline]
    #[must_use]
    pub const fn minutes(self) -> Option<u8> {
        let [_, m, _, _] = self.0.to_be_bytes();
        if m <= 59 { Some(m) } else { None }
    }

    /// Return the seconds component (`0..=59`), or `None` if unspecified or out of range.
    #[inline]
    #[must_use]
    pub const fn seconds(self) -> Option<u8> {
        let [_, _, s, _] = self.0.to_be_bytes();
        if s <= 59 { Some(s) } else { None }
    }

    /// Return the hundredths-of-a-second component (`0..=99`), or `None` if unspecified or out of range.
    #[inline]
    #[must_use]
    pub const fn hundredths(self) -> Option<u8> {
        let [_, _, _, hs] = self.0.to_be_bytes();
        if hs <= 99 { Some(hs) } else { None }
    }

    /// Convert to a [`jiff::civil::Time`] if all four components (`hours`, `minutes`, `seconds`, `hundredths`) are
    /// specified.
    #[cfg(feature = "jiff")]
    #[inline]
    #[must_use]
    pub fn to_jiff_time(self) -> Option<JiffTime> {
        let h = i8::try_from(self.hours()?).ok()?;
        let m = i8::try_from(self.minutes()?).ok()?;
        let s = i8::try_from(self.seconds()?).ok()?;
        let ns = i32::from(self.hundredths()?).saturating_mul(10_000_000);
        JiffTime::new(h, m, s, ns).ok()
    }

    /// Construct from a [`jiff::civil::Time`], truncating sub-hundredth-second nanoseconds.
    #[cfg(feature = "jiff")]
    #[inline]
    #[must_use]
    pub fn from_jiff_time(time: JiffTime) -> Self {
        let h = u8::try_from(time.hour()).unwrap_or(0).min(23);
        let m = u8::try_from(time.minute()).unwrap_or(0).min(59);
        let s = u8::try_from(time.second()).unwrap_or(0).min(59);
        let hs = u8::try_from(time.subsec_nanosecond().max(0).wrapping_div(10_000_000).min(99)).unwrap_or(99);
        Self(u32::from_be_bytes([h, m, s, hs]))
    }

    /// Convert to a [`time::Time`] if all four components (`hours`, `minutes`, `seconds`, `hundredths`) are specified.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn to_time_time(self) -> Option<TimeLibTime> {
        let h = self.hours()?;
        let m = self.minutes()?;
        let s = self.seconds()?;
        let hs = u16::from(self.hundredths()?);
        TimeLibTime::from_hms_milli(h, m, s, hs.saturating_mul(10)).ok()
    }

    /// Construct from a [`time::Time`], truncating sub-hundredth-second milliseconds.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn from_time_time(time: TimeLibTime) -> Self {
        let (h, m, s, ms) = time.as_hms_milli();
        let hs = u8::try_from(ms.wrapping_div(10).min(99)).unwrap_or(99);
        Self(u32::from_be_bytes([h, m, s, hs]))
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TimeOfDay({:#010x})", self.0)
    }
}

impl From<TimeOfDay> for AttributeValue {
    #[inline]
    fn from(val: TimeOfDay) -> Self {
        Self::U32(val.0)
    }
}

impl TryFrom<AttributeValue> for TimeOfDay {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&AttributeValue> for TimeOfDay {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: &AttributeValue) -> Result<Self, Self::Error> {
        decode_unsigned::<u32>(val).map(Self)
    }
}

#[cfg(feature = "jiff")]
impl TryFrom<TimeOfDay> for JiffTime {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: TimeOfDay) -> Result<Self, Self::Error> {
        val.to_jiff_time().ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(feature = "jiff")]
impl From<JiffTime> for TimeOfDay {
    #[inline]
    fn from(time: JiffTime) -> Self {
        Self::from_jiff_time(time)
    }
}

#[cfg(feature = "time")]
impl TryFrom<TimeOfDay> for TimeLibTime {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: TimeOfDay) -> Result<Self, Self::Error> {
        val.to_time_time().ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(feature = "time")]
impl From<TimeLibTime> for TimeOfDay {
    #[inline]
    fn from(time: TimeLibTime) -> Self {
        Self::from_time_time(time)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;

    #[test]
    fn time_of_day_round_trip() {
        let t = TimeOfDay::from_raw(0x0c_1e_00_00);
        assert_eq!(t.raw(), 0x0c_1e_00_00);
        assert_eq!(t.to_string(), "TimeOfDay(0x0c1e0000)");

        let av = AttributeValue::from(t);
        assert_eq!(TimeOfDay::try_from(&av), Ok(t));
        assert_eq!(TimeOfDay::try_from(av), Ok(t));
        assert_eq!(TimeOfDay::try_from(&AttributeValue::U8(0)), Ok(TimeOfDay::from_raw(0)));
        assert_eq!(
            TimeOfDay::try_from(&AttributeValue::U16(42)),
            Ok(TimeOfDay::from_raw(42))
        );
        assert_eq!(
            TimeOfDay::try_from(&AttributeValue::U64(42)),
            Ok(TimeOfDay::from_raw(42))
        );
        assert_eq!(
            TimeOfDay::try_from(&AttributeValue::U64(u64::MAX)),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(
            TimeOfDay::try_from(&AttributeValue::I32(0)),
            Err(MatterStatus::InvalidDataType)
        );
    }

    #[test]
    fn time_of_day_new_and_accessors() {
        let t = TimeOfDay::new(Some(12), Some(30), Some(45), Some(99)).expect("valid time of day");
        assert_eq!(t.hours(), Some(12));
        assert_eq!(t.minutes(), Some(30));
        assert_eq!(t.seconds(), Some(45));
        assert_eq!(t.hundredths(), Some(99));
        assert_eq!(t.raw(), u32::from_be_bytes([12, 30, 45, 99]));

        let wildcard = TimeOfDay::new(None, None, None, None).expect("wildcard time of day");
        assert_eq!(wildcard.raw(), 0xFFFF_FFFF);
        assert_eq!(wildcard.hours(), None);
        assert_eq!(wildcard.minutes(), None);
        assert_eq!(wildcard.seconds(), None);
        assert_eq!(wildcard.hundredths(), None);

        // Out-of-range construction rejected
        assert_eq!(TimeOfDay::new(Some(24), Some(0), Some(0), Some(0)), None);
        assert_eq!(TimeOfDay::new(Some(0), Some(60), Some(0), Some(0)), None);
        assert_eq!(TimeOfDay::new(Some(0), Some(0), Some(60), Some(0)), None);
        assert_eq!(TimeOfDay::new(Some(0), Some(0), Some(0), Some(100)), None);

        // Raw out-of-range octets return None on getters
        let invalid_raw = TimeOfDay::from_raw(u32::from_be_bytes([24, 60, 60, 100]));
        assert_eq!(invalid_raw.hours(), None);
        assert_eq!(invalid_raw.minutes(), None);
        assert_eq!(invalid_raw.seconds(), None);
        assert_eq!(invalid_raw.hundredths(), None);
    }

    #[cfg(feature = "jiff")]
    #[test]
    fn time_of_day_jiff_round_trip() {
        let t = TimeOfDay::new(Some(12), Some(30), Some(45), Some(99)).expect("valid time");
        let jt = JiffTime::try_from(t).expect("valid jiff::civil::Time");
        assert_eq!(jt, JiffTime::new(12, 30, 45, 990_000_000).expect("hms"));
        assert_eq!(TimeOfDay::from(jt), t);

        let wildcard = TimeOfDay::new(Some(12), None, Some(0), Some(0)).expect("wildcard");
        assert_eq!(JiffTime::try_from(wildcard), Err(MatterStatus::ConstraintError));
    }

    #[cfg(feature = "time")]
    #[test]
    fn time_of_day_time_round_trip() {
        let t = TimeOfDay::new(Some(12), Some(30), Some(45), Some(99)).expect("valid time");
        let tt = TimeLibTime::try_from(t).expect("valid time::Time");
        assert_eq!(tt, TimeLibTime::from_hms_milli(12, 30, 45, 990).expect("hms"));
        assert_eq!(TimeOfDay::from(tt), t);

        let wildcard = TimeOfDay::new(Some(12), None, Some(0), Some(0)).expect("wildcard");
        assert_eq!(TimeLibTime::try_from(wildcard), Err(MatterStatus::ConstraintError));
    }
}
