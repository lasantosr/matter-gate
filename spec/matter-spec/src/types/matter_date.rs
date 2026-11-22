//! Matter `date` semantic type.

use core::fmt;

#[cfg(feature = "chrono")]
use chrono::{Datelike as _, NaiveDate};
#[cfg(feature = "time")]
use time::{Date as TimeDate, Month as TimeMonth};

use crate::model::{AttributeValue, MatterStatus};

/// Matter Date (`date`) encoded as a packed `u32` calendar bitfield.
///
/// Per CSA Matter Core Specification §7.19.2.15, the 4 octets pack:
/// - `year` (offset from `1900`, `0..=254`, `0xFF` = unspecified)
/// - `month` (`1..=12`, `0xFF` = unspecified)
/// - `day` (`1..=31`, `0xFF` = unspecified)
/// - `day_of_week` (`1..=7` for Monday..=Sunday, `0xFF` = unspecified)
///
/// Construct explicitly with [`MatterDate::from_raw`] (no `From<u32>` is implemented).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(transparent))]
#[repr(transparent)]
pub struct MatterDate(u32);

impl Default for MatterDate {
    fn default() -> Self {
        Self(0xFFFF_FFFF)
    }
}

impl MatterDate {
    /// Sentinel octet indicating an unspecified calendar field (`0xFF`).
    pub const UNSPECIFIED_OCTET: u8 = 0xFF;

    /// Construct from a raw packed `u32` date value.
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the raw packed `u32` date value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Construct a [`MatterDate`] from calendar components (`None` encodes [`Self::UNSPECIFIED_OCTET`]).
    ///
    /// Returns `None` if any provided component is outside its valid Matter range:
    /// - `year`: `1900..=2154`
    /// - `month`: `1..=12`
    /// - `day`: `1..=31`
    /// - `day_of_week`: `1..=7`
    #[inline]
    #[must_use]
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "checked 1900..=2154 fits in u8"
    )]
    pub const fn new(year: Option<u16>, month: Option<u8>, day: Option<u8>, day_of_week: Option<u8>) -> Option<Self> {
        let year_byte = match year {
            Some(y) if y >= 1900 && y <= 2154 => y.saturating_sub(1900) as u8,
            Some(_) => return None,
            None => Self::UNSPECIFIED_OCTET,
        };
        let month_byte = match month {
            Some(m) if m >= 1 && m <= 12 => m,
            Some(_) => return None,
            None => Self::UNSPECIFIED_OCTET,
        };
        let day_byte = match day {
            Some(d) if d >= 1 && d <= 31 => d,
            Some(_) => return None,
            None => Self::UNSPECIFIED_OCTET,
        };
        let dow_byte = match day_of_week {
            Some(dow) if dow >= 1 && dow <= 7 => dow,
            Some(_) => return None,
            None => Self::UNSPECIFIED_OCTET,
        };
        Some(Self(u32::from_be_bytes([year_byte, month_byte, day_byte, dow_byte])))
    }

    /// Return the calendar year (`1900..=2154`), or `None` if unspecified (`0xFF`).
    #[inline]
    #[must_use]
    #[expect(clippy::as_conversions, reason = "const widening from u8 to u16")]
    pub const fn year(self) -> Option<u16> {
        let [y, _, _, _] = self.0.to_be_bytes();
        if y == Self::UNSPECIFIED_OCTET {
            None
        } else {
            Some(1900_u16.saturating_add(y as u16))
        }
    }

    /// Return the calendar month (`1..=12`), or `None` if unspecified or out of range.
    #[inline]
    #[must_use]
    pub const fn month(self) -> Option<u8> {
        let [_, m, _, _] = self.0.to_be_bytes();
        if m >= 1 && m <= 12 { Some(m) } else { None }
    }

    /// Return the calendar day of the month (`1..=31`), or `None` if unspecified or out of range.
    #[inline]
    #[must_use]
    pub const fn day(self) -> Option<u8> {
        let [_, _, d, _] = self.0.to_be_bytes();
        if d >= 1 && d <= 31 { Some(d) } else { None }
    }

    /// Return the day of the week (`1..=7` for Monday..=Sunday), or `None` if unspecified or out of range.
    #[inline]
    #[must_use]
    pub const fn day_of_week(self) -> Option<u8> {
        let [_, _, _, dow] = self.0.to_be_bytes();
        if dow >= 1 && dow <= 7 { Some(dow) } else { None }
    }

    /// Convert to a [`chrono::NaiveDate`] if `year`, `month`, and `day` are all specified and form a valid calendar
    /// date.
    #[cfg(feature = "chrono")]
    #[inline]
    #[must_use]
    pub fn to_chrono_naive_date(self) -> Option<NaiveDate> {
        let year = i32::from(self.year()?);
        let month = u32::from(self.month()?);
        let day = u32::from(self.day()?);
        NaiveDate::from_ymd_opt(year, month, day)
    }

    /// Construct from a [`chrono::NaiveDate`], returning `None` if the year is outside `1900..=2154`.
    #[cfg(feature = "chrono")]
    #[inline]
    #[must_use]
    pub fn from_chrono_naive_date(date: NaiveDate) -> Option<Self> {
        let year = u16::try_from(date.year()).ok()?;
        let month = u8::try_from(date.month()).ok()?;
        let day = u8::try_from(date.day()).ok()?;
        let dow = u8::try_from(date.weekday().number_from_monday()).ok()?;
        Self::new(Some(year), Some(month), Some(day), Some(dow))
    }

    /// Convert to a [`time::Date`] if `year`, `month`, and `day` are all specified and form a valid calendar date.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn to_time_date(self) -> Option<TimeDate> {
        let year = i32::from(self.year()?);
        let month = TimeMonth::try_from(self.month()?).ok()?;
        let day = self.day()?;
        TimeDate::from_calendar_date(year, month, day).ok()
    }

    /// Construct from a [`time::Date`], returning `None` if the year is outside `1900..=2154`.
    #[cfg(feature = "time")]
    #[inline]
    #[must_use]
    pub fn from_time_date(date: TimeDate) -> Option<Self> {
        let year = u16::try_from(date.year()).ok()?;
        let month = u8::from(date.month());
        let day = date.day();
        let dow = date.weekday().number_from_monday();
        Self::new(Some(year), Some(month), Some(day), Some(dow))
    }
}

impl fmt::Display for MatterDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MatterDate({:#010x})", self.0)
    }
}

impl From<MatterDate> for AttributeValue {
    #[inline]
    fn from(val: MatterDate) -> Self {
        Self::U32(val.0)
    }
}

impl TryFrom<AttributeValue> for MatterDate {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&AttributeValue> for MatterDate {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: &AttributeValue) -> Result<Self, Self::Error> {
        match val {
            AttributeValue::U32(v) => Ok(Self(*v)),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<MatterDate> for NaiveDate {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: MatterDate) -> Result<Self, Self::Error> {
        val.to_chrono_naive_date().ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<NaiveDate> for MatterDate {
    type Error = MatterStatus;

    #[inline]
    fn try_from(date: NaiveDate) -> Result<Self, Self::Error> {
        Self::from_chrono_naive_date(date).ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(feature = "time")]
impl TryFrom<MatterDate> for TimeDate {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: MatterDate) -> Result<Self, Self::Error> {
        val.to_time_date().ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(feature = "time")]
impl TryFrom<TimeDate> for MatterDate {
    type Error = MatterStatus;

    #[inline]
    fn try_from(date: TimeDate) -> Result<Self, Self::Error> {
        Self::from_time_date(date).ok_or(MatterStatus::ConstraintError)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;

    #[test]
    fn matter_date_round_trip() {
        let d = MatterDate::from_raw(0x7e_0b_0d_05);
        assert_eq!(d.raw(), 0x7e_0b_0d_05);
        assert_eq!(d.to_string(), "MatterDate(0x7e0b0d05)");

        let av = AttributeValue::from(d);
        assert_eq!(MatterDate::try_from(&av), Ok(d));
        assert_eq!(MatterDate::try_from(av), Ok(d));
        assert_eq!(
            MatterDate::try_from(&AttributeValue::U16(0)),
            Err(MatterStatus::InvalidDataType)
        );
    }

    #[test]
    fn matter_date_new_and_accessors() {
        let d = MatterDate::new(Some(2026), Some(11), Some(14), Some(6)).expect("valid date");
        assert_eq!(d.year(), Some(2026));
        assert_eq!(d.month(), Some(11));
        assert_eq!(d.day(), Some(14));
        assert_eq!(d.day_of_week(), Some(6));
        assert_eq!(d.raw(), u32::from_be_bytes([126, 11, 14, 6]));

        let wildcard = MatterDate::new(None, None, None, None).expect("wildcard date");
        assert_eq!(wildcard.raw(), 0xFFFF_FFFF);
        assert_eq!(wildcard.year(), None);
        assert_eq!(wildcard.month(), None);
        assert_eq!(wildcard.day(), None);
        assert_eq!(wildcard.day_of_week(), None);

        // Out-of-range construction rejected
        assert_eq!(MatterDate::new(Some(1899), Some(1), Some(1), Some(1)), None);
        assert_eq!(MatterDate::new(Some(2155), Some(1), Some(1), Some(1)), None);
        assert_eq!(MatterDate::new(Some(2000), Some(0), Some(1), Some(1)), None);
        assert_eq!(MatterDate::new(Some(2000), Some(13), Some(1), Some(1)), None);
        assert_eq!(MatterDate::new(Some(2000), Some(1), Some(0), Some(1)), None);
        assert_eq!(MatterDate::new(Some(2000), Some(1), Some(32), Some(1)), None);
        assert_eq!(MatterDate::new(Some(2000), Some(1), Some(1), Some(0)), None);
        assert_eq!(MatterDate::new(Some(2000), Some(1), Some(1), Some(8)), None);

        // Raw out-of-range octets return None on getters
        let invalid_raw = MatterDate::from_raw(u32::from_be_bytes([0xFF, 0, 32, 8]));
        assert_eq!(invalid_raw.year(), None);
        assert_eq!(invalid_raw.month(), None);
        assert_eq!(invalid_raw.day(), None);
        assert_eq!(invalid_raw.day_of_week(), None);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn matter_date_chrono_round_trip() {
        let d = MatterDate::new(Some(2026), Some(11), Some(15), Some(7)).expect("valid date");
        let nd = NaiveDate::try_from(d).expect("valid NaiveDate");
        assert_eq!(nd, NaiveDate::from_ymd_opt(2026, 11, 15).expect("ymd"));
        assert_eq!(MatterDate::try_from(nd), Ok(d));

        let wildcard = MatterDate::new(None, Some(11), Some(15), None).expect("wildcard");
        assert_eq!(NaiveDate::try_from(wildcard), Err(MatterStatus::ConstraintError));
    }

    #[cfg(feature = "time")]
    #[test]
    fn matter_date_time_round_trip() {
        let d = MatterDate::new(Some(2026), Some(11), Some(15), Some(7)).expect("valid date");
        let td = TimeDate::try_from(d).expect("valid time::Date");
        assert_eq!(
            td,
            TimeDate::from_calendar_date(2026, TimeMonth::November, 15).expect("ymd")
        );
        assert_eq!(MatterDate::try_from(td), Ok(d));

        let wildcard = MatterDate::new(None, Some(11), Some(15), None).expect("wildcard");
        assert_eq!(TimeDate::try_from(wildcard), Err(MatterStatus::ConstraintError));
    }
}
