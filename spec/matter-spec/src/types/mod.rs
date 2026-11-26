//! Semantic newtypes for Matter specification types.
//!
//! These types wrap raw Matter wire primitives and byte buffers (`bytes::Bytes`) to provide
//! compile-time unit, time, and address safety across all Matter versions.
//!
//! # Construction & Access Pattern
//!
//! - **Explicit raw conversion:** Semantic types use [`from_raw`](Temperature::from_raw) and [`raw`](Temperature::raw)
//!   (never `From<raw>`) so raw wire integers are never silently confused with scaled physical units.
//! - **Cluster attribute `Deref`:** Generated attribute newtypes wrapping a semantic type implement
//!   [`core::ops::Deref`] to the inner semantic type, exposing unit helpers directly on the attribute.
//! - **Nullable attributes:** Nullable attributes wrap `Nullable<T>` and can be transformed ergonomically with
//!   [`Nullable::map`](crate::model::Nullable::map).
//!
//! ```rust
//! use matter_spec::{
//!     model::Nullable,
//!     types::{Percent100ths, Temperature},
//! };
//!
//! let temp = Temperature::from_celsius(21.5);
//! assert_eq!(temp.raw(), 2150);
//!
//! let nullable_temp = Nullable::NonNull(temp);
//! let celsius = nullable_temp.map(Temperature::to_celsius);
//! assert_eq!(celsius, Nullable::NonNull(21.5));
//!
//! let pct = Percent100ths::from_percent(50.25);
//! assert_eq!(pct.raw(), 5025);
//! ```

macro_rules! define_matter_scalar_type {
    (
        $(#[$meta:meta])*
        pub struct $name:ident($inner:ty);
        attr_variant = $variant:ident;
        display_fmt = $display_fmt:literal;
    ) => {
        define_matter_scalar_type! {
            $(#[$meta])*
            pub struct $name($inner);
            attr_variant = $variant;
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, $display_fmt, self.0)
            }
        }
    };
    (
        $(#[$meta:meta])*
        pub struct $name:ident($inner:ty);
        attr_variant = $variant:ident;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(transparent))]
        #[repr(transparent)]
        pub struct $name($inner);

        impl $name {
            #[doc = concat!("Construct from a raw `", stringify!($inner), "` value.")]
            #[inline]
            #[must_use]
            pub const fn from_raw(raw: $inner) -> Self {
                Self(raw)
            }

            #[doc = concat!("Return the raw `", stringify!($inner), "` value.")]
            #[inline]
            #[must_use]
            pub const fn raw(self) -> $inner {
                self.0
            }
        }

        impl From<$name> for crate::model::AttributeValue {
            #[inline]
            fn from(val: $name) -> Self {
                Self::$variant(val.0)
            }
        }

        impl TryFrom<crate::model::AttributeValue> for $name {
            type Error = crate::model::MatterStatus;

            #[inline]
            fn try_from(val: crate::model::AttributeValue) -> Result<Self, Self::Error> {
                Self::try_from(&val)
            }
        }

        impl TryFrom<&crate::model::AttributeValue> for $name {
            type Error = crate::model::MatterStatus;

            #[inline]
            fn try_from(val: &crate::model::AttributeValue) -> Result<Self, Self::Error> {
                crate::traits::decode_scalar::<$inner>(val).map(Self)
            }
        }
    };
}

macro_rules! define_matter_milli_unit {
    (
        $(#[$meta:meta])*
        pub struct $name:ident(i64);
        display_fmt = $display_fmt:literal;
        from_unit = $from_fn:ident;
        to_unit = $to_fn:ident;
        unit_doc = $unit_doc:literal;
        milli_doc = $milli_doc:literal;
    ) => {
        define_matter_scalar_type! {
            $(#[$meta])*
            pub struct $name(i64);
            attr_variant = I64;
            display_fmt = $display_fmt;
        }

        impl $name {
            #[doc = concat!("Construct from ", $unit_doc, ", rounding to the nearest ", $milli_doc, ".")]
            #[inline]
            #[expect(
                clippy::as_conversions,
                clippy::cast_possible_truncation,
                reason = "no_std float scaling"
            )]
            #[must_use]
            pub fn $from_fn(val: f64) -> Self {
                if val.is_nan() {
                    return Self(0);
                }
                let scaled = val * 1000.0;
                let rounded = if scaled >= 0.0 { scaled + 0.5 } else { scaled - 0.5 };
                Self(rounded.clamp(-9_223_372_036_854_776_000.0, 9_223_372_036_854_775_000.0) as i64)
            }

            #[doc = concat!("Convert to ", $unit_doc, ".")]
            #[inline]
            #[expect(clippy::as_conversions, clippy::cast_precision_loss, reason = "no_std float scaling")]
            #[must_use]
            pub fn $to_fn(self) -> f64 {
                self.0 as f64 / 1000.0
            }
        }
    };
}

macro_rules! define_matter_ip_addr_type {
    (
        $(#[$meta:meta])*
        pub struct $name:ident([u8; $len:literal]);
        net_addr = $net_ty:ty;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(transparent))]
        #[repr(transparent)]
        pub struct $name([u8; $len]);

        impl $name {
            #[doc = concat!("Construct from a raw ", stringify!($len), "-byte array.")]
            #[inline]
            #[must_use]
            pub const fn from_raw(raw: [u8; $len]) -> Self {
                Self(raw)
            }

            #[doc = concat!("Return the raw ", stringify!($len), "-byte array.")]
            #[inline]
            #[must_use]
            pub const fn raw(self) -> [u8; $len] {
                self.0
            }

            #[doc = concat!("Borrow the underlying ", stringify!($len), "-byte slice.")]
            #[inline]
            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; $len] {
                &self.0
            }
        }

        impl AsRef<[u8]> for $name {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl From<$net_ty> for $name {
            #[inline]
            fn from(addr: $net_ty) -> Self {
                Self(addr.octets())
            }
        }

        impl From<$name> for $net_ty {
            #[inline]
            fn from(addr: $name) -> Self {
                Self::from(addr.0)
            }
        }

        impl TryFrom<&[u8]> for $name {
            type Error = crate::model::MatterStatus;

            #[inline]
            fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                let arr: [u8; $len] = bytes
                    .try_into()
                    .map_err(|_| crate::model::MatterStatus::ConstraintError)?;
                Ok(Self(arr))
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", <$net_ty>::from(self.0))
            }
        }

        impl From<$name> for crate::model::AttributeValue {
            #[inline]
            fn from(val: $name) -> Self {
                Self::Bytes(crate::Bytes::copy_from_slice(&val.0))
            }
        }

        impl TryFrom<crate::model::AttributeValue> for $name {
            type Error = crate::model::MatterStatus;

            #[inline]
            fn try_from(val: crate::model::AttributeValue) -> Result<Self, Self::Error> {
                Self::try_from(&val)
            }
        }

        impl TryFrom<&crate::model::AttributeValue> for $name {
            type Error = crate::model::MatterStatus;

            #[inline]
            fn try_from(val: &crate::model::AttributeValue) -> Result<Self, Self::Error> {
                match val {
                    crate::model::AttributeValue::Bytes(bytes) => Self::try_from(&**bytes),
                    _ => Err(crate::model::MatterStatus::InvalidDataType),
                }
            }
        }
    };
}

pub(crate) use define_matter_ip_addr_type;
pub(crate) use define_matter_milli_unit;
pub(crate) use define_matter_scalar_type;

macro_rules! impl_matter_uom_unit {
    ($name:ident, $uom_ty:ty, $uom_unit:ty, $from_fn:ident, $to_fn:ident) => {
        #[cfg(feature = "uom")]
        impl From<$name> for $uom_ty {
            #[inline]
            fn from(val: $name) -> Self {
                Self::new::<$uom_unit>(val.$to_fn())
            }
        }

        #[cfg(feature = "uom")]
        impl From<$uom_ty> for $name {
            #[inline]
            fn from(val: $uom_ty) -> Self {
                Self::$from_fn(val.get::<$uom_unit>())
            }
        }
    };
}

macro_rules! impl_matter_epoch_conversions {
    ($name:ident, $raw:ty, $dur_method:ident) => {
        impl From<$name> for Duration {
            #[inline]
            fn from(val: $name) -> Self {
                val.as_duration()
            }
        }

        impl TryFrom<Duration> for $name {
            type Error = MatterStatus;

            #[inline]
            fn try_from(duration: Duration) -> Result<Self, Self::Error> {
                <$raw>::try_from(duration.$dur_method())
                    .map(Self)
                    .map_err(|_| MatterStatus::ConstraintError)
            }
        }

        #[cfg(feature = "std")]
        impl TryFrom<$name> for SystemTime {
            type Error = MatterStatus;

            #[inline]
            fn try_from(val: $name) -> Result<Self, Self::Error> {
                val.to_system_time().ok_or(MatterStatus::ConstraintError)
            }
        }

        #[cfg(feature = "std")]
        impl TryFrom<SystemTime> for $name {
            type Error = MatterStatus;

            #[inline]
            fn try_from(time: SystemTime) -> Result<Self, Self::Error> {
                Self::from_system_time(time).ok_or(MatterStatus::ConstraintError)
            }
        }

        #[cfg(feature = "jiff")]
        impl TryFrom<$name> for Timestamp {
            type Error = MatterStatus;

            #[inline]
            fn try_from(val: $name) -> Result<Self, Self::Error> {
                val.to_jiff_timestamp().ok_or(MatterStatus::ConstraintError)
            }
        }

        #[cfg(feature = "jiff")]
        impl TryFrom<Timestamp> for $name {
            type Error = MatterStatus;

            #[inline]
            fn try_from(ts: Timestamp) -> Result<Self, Self::Error> {
                Self::from_jiff_timestamp(ts).ok_or(MatterStatus::ConstraintError)
            }
        }

        #[cfg(feature = "time")]
        impl TryFrom<$name> for OffsetDateTime {
            type Error = MatterStatus;

            #[inline]
            fn try_from(val: $name) -> Result<Self, Self::Error> {
                val.to_time_offset_datetime().ok_or(MatterStatus::ConstraintError)
            }
        }

        #[cfg(feature = "time")]
        impl TryFrom<OffsetDateTime> for $name {
            type Error = MatterStatus;

            #[inline]
            fn try_from(dt: OffsetDateTime) -> Result<Self, Self::Error> {
                Self::from_time_offset_datetime(dt).ok_or(MatterStatus::ConstraintError)
            }
        }
    };
}

pub(crate) use impl_matter_epoch_conversions;
pub(crate) use impl_matter_uom_unit;

// Units
pub mod amperage_ma;
pub mod energy_mvah;
pub mod energy_mvarh;
pub mod energy_mwh;
pub mod percent;
pub mod percent100ths;
pub mod power_mva;
pub mod power_mvar;
pub mod power_mw;
pub mod temperature;
pub mod voltage_mv;

// Time & Date
pub mod elapsed_s;
pub mod epoch_s;
pub mod epoch_us;
pub mod matter_date;
pub mod posix_ms;
pub mod systime_ms;
pub mod systime_us;
pub mod time_of_day;

// Network & Hardware Addresses
pub mod hardware_address;
pub mod ip_address;
pub mod ipv4_address;
pub mod ipv6_address;
pub mod ipv6_prefix;

pub use amperage_ma::AmperageMa;
pub use elapsed_s::ElapsedS;
pub use energy_mvah::EnergyMvah;
pub use energy_mvarh::EnergyMvarh;
pub use energy_mwh::EnergyMwh;
pub use epoch_s::EpochS;
pub use epoch_us::EpochUs;
pub use hardware_address::HardwareAddress;
pub use ip_address::IpAddress;
pub use ipv4_address::Ipv4Address;
pub use ipv6_address::Ipv6Address;
pub use ipv6_prefix::Ipv6Prefix;
pub use matter_date::MatterDate;
pub use percent::Percent;
pub use percent100ths::Percent100ths;
pub use posix_ms::PosixMs;
pub use power_mva::PowerMva;
pub use power_mvar::PowerMvar;
pub use power_mw::PowerMw;
pub use systime_ms::SystimeMs;
pub use systime_us::SystimeUs;
pub use temperature::Temperature;
pub use time_of_day::TimeOfDay;
pub use voltage_mv::VoltageMv;
