//! Declarative macros for generated bitmaps, exhaustive enums, and semantic booleans.

macro_rules! define_matter_bitmap {
    (
        $(#[$meta:meta])*
        pub struct $name:ident($width:ty);
        attr_variant = $attr_variant:ident;
        display_fmt = $display_fmt:literal;
        flags {
            $(
                $(#[$flag_meta:meta])*
                $const_name:ident : $is_fn:ident = $mask:expr;
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(transparent))]
        #[repr(transparent)]
        pub struct $name($width);

        #[rustfmt::skip]
        impl $name {
            /// Empty bitmap with no flags set (`0`).
            pub const EMPTY: Self = Self(0);
            $(
                $(#[$flag_meta])*
                pub const $const_name: Self = Self($mask);
            )*
        }

        impl $name {
            /// Returns an empty bitmap with no flags set (`0`).
            #[inline]
            #[must_use]
            pub const fn empty() -> Self {
                Self(0)
            }

            /// Constructs a bitmap directly from raw bits, retaining any unknown bits.
            #[inline]
            #[must_use]
            pub const fn from_bits_retain(bits: $width) -> Self {
                Self(bits)
            }

            /// Returns the raw underlying integer bits.
            #[inline]
            #[must_use]
            pub const fn bits(self) -> $width {
                self.0
            }

            /// Returns `true` if no flags are set (`0`).
            #[inline]
            #[must_use]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            /// Returns `true` if all flags in `other` are set in `self`.
            #[inline]
            #[must_use]
            pub const fn contains(self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }

            /// Inserts all flags in `other` into `self`.
            #[inline]
            pub const fn insert(&mut self, other: Self) {
                self.0 |= other.0;
            }

            /// Removes all flags in `other` from `self`.
            #[inline]
            pub const fn remove(&mut self, other: Self) {
                self.0 &= !other.0;
            }

            /// Returns the bitwise union (`self | other`) of two bitmaps.
            #[inline]
            #[must_use]
            pub const fn union(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }

            /// Returns the bitwise intersection (`self & other`) of two bitmaps.
            #[inline]
            #[must_use]
            pub const fn intersection(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }

            $(
                #[doc = concat!("Returns `true` if [`Self::", stringify!($const_name), "`] is set.")]
                #[inline]
                #[must_use]
                pub const fn $is_fn(self) -> bool {
                    (self.0 & $mask) != 0
                }
            )*
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, $display_fmt, self.0)
            }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl core::ops::BitAnd for $name {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl core::ops::BitAndAssign for $name {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl core::ops::BitXor for $name {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }

        impl core::ops::BitXorAssign for $name {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }

        impl core::ops::Not for $name {
            type Output = Self;

            #[inline]
            fn not(self) -> Self {
                Self(!self.0)
            }
        }

        impl From<$name> for $width {
            #[inline]
            fn from(bm: $name) -> Self {
                bm.0
            }
        }

        impl From<$width> for $name {
            #[inline]
            fn from(val: $width) -> Self {
                Self(val)
            }
        }

        impl From<$name> for $crate::model::AttributeValue {
            #[inline]
            fn from(bm: $name) -> Self {
                Self::$attr_variant(bm.0)
            }
        }

        impl TryFrom<$crate::model::AttributeValue> for $name {
            type Error = $crate::model::MatterStatus;

            #[inline]
            fn try_from(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                Self::try_from(&val)
            }
        }

        impl TryFrom<&$crate::model::AttributeValue> for $name {
            type Error = $crate::model::MatterStatus;

            #[inline]
            fn try_from(val: &$crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                $crate::traits::decode_unsigned::<$width>(val).map(Self)
            }
        }
    };
}

macro_rules! define_matter_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($variant:ident = $val:expr),* $(,)?
        }
        repr = $repr_ty:ident;
        attr_variant = $attr_variant:ident;
        default = $default_variant:ident;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[repr($repr_ty)]
        pub enum $name {
            $(
                #[doc = concat!("`", stringify!($variant), "` variant.")]
                $variant = $val,
            )*
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(Self::$variant => write!(f, stringify!($variant)),)*
                }
            }
        }

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                Self::$default_variant
            }
        }

        impl TryFrom<$repr_ty> for $name {
            type Error = $crate::model::MatterStatus;

            fn try_from(val: $repr_ty) -> Result<Self, $crate::model::MatterStatus> {
                match val {
                    $($val => Ok(Self::$variant),)*
                    _ => Err($crate::model::MatterStatus::ConstraintError),
                }
            }
        }

        impl From<$name> for $repr_ty {
            #[inline]
            fn from(val: $name) -> Self {
                match val {
                    $($name::$variant => $val,)*
                }
            }
        }

        impl From<$name> for $crate::model::AttributeValue {
            #[inline]
            fn from(val: $name) -> Self {
                Self::$attr_variant($repr_ty::from(val))
            }
        }

        impl TryFrom<$crate::model::AttributeValue> for $name {
            type Error = $crate::model::MatterStatus;

            #[inline]
            fn try_from(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                Self::try_from(&val)
            }
        }

        impl TryFrom<&$crate::model::AttributeValue> for $name {
            type Error = $crate::model::MatterStatus;

            fn try_from(val: &$crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                let raw = $crate::traits::decode_unsigned::<$repr_ty>(val)?;
                Self::try_from(raw)
            }
        }
    };
}

macro_rules! define_matter_semantic_bool {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $false_variant:ident = 0,
            $true_variant:ident = 1 $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[repr(u8)]
        pub enum $name {
            #[doc = concat!("`", stringify!($false_variant), "` (`false` / `0`).")]
            #[default]
            $false_variant = 0,
            #[doc = concat!("`", stringify!($true_variant), "` (`true` / `1`).")]
            $true_variant = 1,
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Self::$false_variant => write!(f, stringify!($false_variant)),
                    Self::$true_variant => write!(f, stringify!($true_variant)),
                }
            }
        }

        impl From<bool> for $name {
            #[inline]
            fn from(val: bool) -> Self {
                if val { Self::$true_variant } else { Self::$false_variant }
            }
        }

        impl From<$name> for bool {
            #[inline]
            fn from(val: $name) -> Self {
                matches!(val, $name::$true_variant)
            }
        }

        impl From<$name> for $crate::model::AttributeValue {
            #[inline]
            fn from(val: $name) -> Self {
                Self::Bool(bool::from(val))
            }
        }

        impl TryFrom<$crate::model::AttributeValue> for $name {
            type Error = $crate::model::MatterStatus;

            #[inline]
            fn try_from(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                Self::try_from(&val)
            }
        }

        impl TryFrom<&$crate::model::AttributeValue> for $name {
            type Error = $crate::model::MatterStatus;

            #[inline]
            fn try_from(val: &$crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                match val {
                    $crate::model::AttributeValue::Bool(bool_val) => Ok(Self::from(*bool_val)),
                    _ => Err($crate::model::MatterStatus::InvalidDataType),
                }
            }
        }
    };
}

pub(crate) use define_matter_bitmap;
pub(crate) use define_matter_enum;
pub(crate) use define_matter_semantic_bool;
