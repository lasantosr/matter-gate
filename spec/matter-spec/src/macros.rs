//! Internal declarative macros for `matter-spec` generated models.
//!
//! These macros collapse repetitive trait implementations, bitmap/enum boilerplate,
//! and wire/dispatch conversions across generated version modules (`v1_0`..`v1_6`)
//! to keep the crate compact and fast to parse.

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
                match val {
                    $crate::model::AttributeValue::$attr_variant(raw_val) => Ok(Self(*raw_val)),
                    _ => Err($crate::model::MatterStatus::InvalidDataType),
                }
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
                match val {
                    $crate::model::AttributeValue::$attr_variant(raw) => Self::try_from(*raw),
                    _ => Err($crate::model::MatterStatus::InvalidDataType),
                }
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

macro_rules! impl_cluster_member_variant {
    (attribute, $enum_ty:path, $member_ty:ty, $variant:ident) => {
        impl From<$member_ty> for $enum_ty {
            #[inline]
            fn from(attr: $member_ty) -> Self {
                Self::$variant(attr)
            }
        }

        impl TryFrom<$enum_ty> for $member_ty {
            type Error = $crate::error::AttributeError;

            #[inline]
            fn try_from(attr: $enum_ty) -> Result<Self, Self::Error> {
                use $enum_ty as __Enum;
                if let __Enum::$variant(inner) = attr {
                    Ok(inner)
                } else {
                    Err($crate::error::AttributeError::AttributeIdMismatch {
                        expected: Self::ATTRIBUTE_ID,
                        actual: attr.id(),
                    })
                }
            }
        }
    };
    (command, $enum_ty:path, $member_ty:ty, $variant:ident) => {
        impl From<$member_ty> for $enum_ty {
            #[inline]
            fn from(cmd: $member_ty) -> Self {
                Self::$variant(cmd)
            }
        }

        impl TryFrom<$enum_ty> for $member_ty {
            type Error = $crate::error::CommandError;

            #[inline]
            fn try_from(cmd: $enum_ty) -> Result<Self, Self::Error> {
                use $enum_ty as __Enum;
                if let __Enum::$variant(inner) = cmd {
                    Ok(inner)
                } else if cmd.id() == Self::COMMAND_ID {
                    Err($crate::error::CommandError::DirectionMismatch {
                        expected: Self::DIRECTION,
                        actual: cmd.direction(),
                    })
                } else {
                    Err($crate::error::CommandError::CommandIdMismatch {
                        expected: Self::COMMAND_ID,
                        actual: cmd.id(),
                    })
                }
            }
        }
    };
    (command_unit, $enum_ty:path, $member_ty:ty, $variant:ident) => {
        impl From<$member_ty> for $enum_ty {
            #[inline]
            fn from(_: $member_ty) -> Self {
                Self::$variant
            }
        }

        impl TryFrom<$enum_ty> for $member_ty {
            type Error = $crate::error::CommandError;

            #[inline]
            fn try_from(cmd: $enum_ty) -> Result<Self, Self::Error> {
                use $enum_ty as __Enum;
                if let __Enum::$variant = cmd {
                    Ok(<$member_ty>::default())
                } else if cmd.id() == Self::COMMAND_ID {
                    Err($crate::error::CommandError::DirectionMismatch {
                        expected: Self::DIRECTION,
                        actual: cmd.direction(),
                    })
                } else {
                    Err($crate::error::CommandError::CommandIdMismatch {
                        expected: Self::COMMAND_ID,
                        actual: cmd.id(),
                    })
                }
            }
        }
    };
    (event, $enum_ty:path, $member_ty:ty, $variant:ident) => {
        impl From<$member_ty> for $enum_ty {
            #[inline]
            fn from(ev: $member_ty) -> Self {
                Self::$variant(ev)
            }
        }

        impl TryFrom<$enum_ty> for $member_ty {
            type Error = $crate::error::EventError;

            #[inline]
            fn try_from(ev: $enum_ty) -> Result<Self, Self::Error> {
                use $enum_ty as __Enum;
                if let __Enum::$variant(inner) = ev {
                    Ok(inner)
                } else {
                    Err($crate::error::EventError::EventIdMismatch {
                        expected: Self::EVENT_ID,
                        actual: ev.id(),
                    })
                }
            }
        }
    };
    (event_unit, $enum_ty:path, $member_ty:ty, $variant:ident) => {
        impl From<$member_ty> for $enum_ty {
            #[inline]
            fn from(_: $member_ty) -> Self {
                Self::$variant
            }
        }

        impl TryFrom<$enum_ty> for $member_ty {
            type Error = $crate::error::EventError;

            #[inline]
            fn try_from(ev: $enum_ty) -> Result<Self, Self::Error> {
                use $enum_ty as __Enum;
                if let __Enum::$variant = ev {
                    Ok(<$member_ty>::default())
                } else {
                    Err($crate::error::EventError::EventIdMismatch {
                        expected: Self::EVENT_ID,
                        actual: ev.id(),
                    })
                }
            }
        }
    };
}

macro_rules! impl_matter_attribute {
    ($name:ident, $inner:ty, $cluster:ty, copy) => {
        impl $name {
            #[inline]
            #[must_use]
            pub const fn as_inner(&self) -> &$inner {
                &self.0
            }

            #[inline]
            #[must_use]
            pub const fn into_inner(self) -> $inner {
                self.0
            }
        }

        $crate::impl_matter_attribute!(@common $name, $inner, $cluster);
    };
    ($name:ident, $inner:ty, $cluster:ty, owned) => {
        impl $name {
            #[inline]
            #[must_use]
            pub const fn as_inner(&self) -> &$inner {
                &self.0
            }

            #[inline]
            #[must_use]
            pub fn into_inner(self) -> $inner {
                self.0
            }
        }

        $crate::impl_matter_attribute!(@common $name, $inner, $cluster);
    };
    (@common $name:ident, $inner:ty, $cluster:ty) => {
        impl From<$name> for $crate::model::Attribute {
            #[inline]
            fn from(attr: $name) -> Self {
                Self::new($name::ATTRIBUTE_ID, attr.into_value())
            }
        }

        impl TryFrom<$crate::model::Attribute> for $name {
            type Error = $crate::error::AttributeError;

            fn try_from(attr: $crate::model::Attribute) -> Result<Self, Self::Error> {
                if attr.id != Self::ATTRIBUTE_ID {
                    return Err($crate::error::AttributeError::AttributeIdMismatch {
                        expected: Self::ATTRIBUTE_ID,
                        actual: attr.id,
                    });
                }
                Self::try_from_value(attr.value).map_err(|status| $crate::error::AttributeError::InvalidValue {
                    id: Self::ATTRIBUTE_ID,
                    field_id: None,
                    status,
                })
            }
        }

        impl From<$inner> for $name {
            #[inline]
            fn from(val: $inner) -> Self {
                Self(val)
            }
        }

        impl From<$name> for $inner {
            #[inline]
            fn from(attr: $name) -> Self {
                attr.0
            }
        }

        impl core::ops::Deref for $name {
            type Target = $inner;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl AsRef<$inner> for $name {
            #[inline]
            fn as_ref(&self) -> &$inner {
                &self.0
            }
        }

        impl $crate::traits::AttributeSpec for $name {
            const CLUSTER_ID: $crate::model::ClusterId = Self::CLUSTER_ID;
            const CLUSTER_NAME: &'static str = <$cluster>::NAME;
            const ATTRIBUTE_ID: $crate::model::AttributeId = Self::ATTRIBUTE_ID;
            const ATTRIBUTE_NAME: &'static str = Self::NAME;
            const IS_NULLABLE: bool = Self::IS_NULLABLE;
            const IS_OPTIONAL: bool = Self::IS_OPTIONAL;
            const IS_WRITABLE: bool = Self::IS_WRITABLE;
        }

        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::MatterDescribe for $name {
            const STATIC_SCHEMA: $crate::dispatch::StaticEntitySchema = $crate::dispatch::StaticEntitySchema::concrete(
                $crate::dispatch::EntityDescriptor::Attribute {
                    cluster: <Self as $crate::traits::AttributeSpec>::CLUSTER_NAME,
                    cluster_id: Self::CLUSTER_ID.raw(),
                    attribute: Self::NAME,
                    attribute_id: Self::ATTRIBUTE_ID.raw(),
                },
                false,
            );

            #[inline]
            fn descriptor(&self) -> $crate::dispatch::EntityDescriptor {
                $crate::dispatch::EntityDescriptor::Attribute {
                    cluster: <Self as $crate::traits::AttributeSpec>::CLUSTER_NAME,
                    cluster_id: Self::CLUSTER_ID.raw(),
                    attribute: Self::NAME,
                    attribute_id: Self::ATTRIBUTE_ID.raw(),
                }
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                false
            }

            #[cfg(feature = "serde")]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_concrete_described(
                    deserializer,
                    $crate::dispatch::EntityDescriptor::Attribute {
                        cluster: <Self as $crate::traits::AttributeSpec>::CLUSTER_NAME,
                        cluster_id: Self::CLUSTER_ID.raw(),
                        attribute: Self::NAME,
                        attribute_id: Self::ATTRIBUTE_ID.raw(),
                    },
                    false,
                )
            }
        }
    };
}

macro_rules! impl_matter_command {
    ($name:ident, $cluster:ty, $is_empty:expr) => {
        impl From<$name> for $crate::model::Command {
            #[inline]
            fn from(cmd: $name) -> Self {
                Self::new(
                    $name::CLUSTER_ID,
                    $name::COMMAND_ID,
                    $name::DIRECTION,
                    cmd.into_value(),
                )
            }
        }

        impl TryFrom<$crate::model::Command> for $name {
            type Error = $crate::error::CommandError;

            fn try_from(cmd: $crate::model::Command) -> Result<Self, Self::Error> {
                if cmd.cluster_id != Self::CLUSTER_ID {
                    return Err($crate::error::CommandError::ClusterIdMismatch {
                        expected: Self::CLUSTER_ID,
                        actual: cmd.cluster_id,
                    });
                }
                if cmd.command_id != Self::COMMAND_ID {
                    return Err($crate::error::CommandError::CommandIdMismatch {
                        expected: Self::COMMAND_ID,
                        actual: cmd.command_id,
                    });
                }
                if cmd.direction != Self::DIRECTION {
                    return Err($crate::error::CommandError::DirectionMismatch {
                        expected: Self::DIRECTION,
                        actual: cmd.direction,
                    });
                }
                Self::try_from_value_with_field(cmd.payload).map_err(|(field_id, status)| {
                    $crate::error::CommandError::InvalidPayload {
                        id: Self::COMMAND_ID,
                        field_id,
                        status,
                    }
                })
            }
        }

        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::MatterDescribe for $name {
            const STATIC_SCHEMA: $crate::dispatch::StaticEntitySchema = $crate::dispatch::StaticEntitySchema::concrete(
                $crate::dispatch::EntityDescriptor::Command {
                    cluster: <Self as $crate::traits::CommandSpec>::CLUSTER_NAME,
                    cluster_id: Self::CLUSTER_ID.raw(),
                    command: Self::NAME,
                    command_id: Self::COMMAND_ID.raw(),
                },
                $is_empty,
            );

            #[inline]
            fn descriptor(&self) -> $crate::dispatch::EntityDescriptor {
                $crate::dispatch::EntityDescriptor::Command {
                    cluster: <Self as $crate::traits::CommandSpec>::CLUSTER_NAME,
                    cluster_id: Self::CLUSTER_ID.raw(),
                    command: Self::NAME,
                    command_id: Self::COMMAND_ID.raw(),
                }
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                $is_empty
            }

            #[cfg(feature = "serde")]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_concrete_described(
                    deserializer,
                    $crate::dispatch::EntityDescriptor::Command {
                        cluster: <Self as $crate::traits::CommandSpec>::CLUSTER_NAME,
                        cluster_id: Self::CLUSTER_ID.raw(),
                        command: Self::NAME,
                        command_id: Self::COMMAND_ID.raw(),
                    },
                    $is_empty,
                )
            }

            #[cfg(feature = "serde")]
            fn deserialize_invocation<'de, D>(deserializer: D) -> Result<($crate::model::EndpointId, Self), D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_concrete_invocation(
                    deserializer,
                    $crate::dispatch::EntityDescriptor::Command {
                        cluster: <Self as $crate::traits::CommandSpec>::CLUSTER_NAME,
                        cluster_id: Self::CLUSTER_ID.raw(),
                        command: Self::NAME,
                        command_id: Self::COMMAND_ID.raw(),
                    },
                    $is_empty,
                )
            }
        }
    };
}

macro_rules! impl_matter_event {
    ($name:ident, $cluster:ty, $is_empty:expr) => {
        impl From<$name> for $crate::model::Event {
            #[inline]
            fn from(ev: $name) -> Self {
                Self::new(
                    $name::CLUSTER_ID,
                    $name::EVENT_ID,
                    $name::PRIORITY,
                    ev.into_value(),
                )
            }
        }

        impl TryFrom<$crate::model::Event> for $name {
            type Error = $crate::error::EventError;

            fn try_from(ev: $crate::model::Event) -> Result<Self, Self::Error> {
                if ev.cluster_id != Self::CLUSTER_ID {
                    return Err($crate::error::EventError::ClusterIdMismatch {
                        expected: Self::CLUSTER_ID,
                        actual: ev.cluster_id,
                    });
                }
                if ev.event_id != Self::EVENT_ID {
                    return Err($crate::error::EventError::EventIdMismatch {
                        expected: Self::EVENT_ID,
                        actual: ev.event_id,
                    });
                }
                if ev.priority != Self::PRIORITY {
                    return Err($crate::error::EventError::PriorityMismatch {
                        expected: Self::PRIORITY,
                        actual: ev.priority,
                    });
                }
                Self::try_from_value_with_field(ev.payload).map_err(|(field_id, status)| {
                    $crate::error::EventError::InvalidPayload {
                        id: Self::EVENT_ID,
                        field_id,
                        status,
                    }
                })
            }
        }

        impl $crate::traits::EventSpec for $name {
            const CLUSTER_ID: $crate::model::ClusterId = Self::CLUSTER_ID;
            const CLUSTER_NAME: &'static str = <$cluster>::NAME;
            const EVENT_ID: $crate::model::EventId = Self::EVENT_ID;
            const EVENT_NAME: &'static str = Self::NAME;
            const IS_EMPTY: bool = $is_empty;
            const PRIORITY: $crate::model::EventPriority = Self::PRIORITY;
        }

        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::MatterDescribe for $name {
            const STATIC_SCHEMA: $crate::dispatch::StaticEntitySchema = $crate::dispatch::StaticEntitySchema::concrete(
                $crate::dispatch::EntityDescriptor::Event {
                    cluster: <Self as $crate::traits::EventSpec>::CLUSTER_NAME,
                    cluster_id: Self::CLUSTER_ID.raw(),
                    event: Self::NAME,
                    event_id: Self::EVENT_ID.raw(),
                },
                $is_empty,
            );

            #[inline]
            fn descriptor(&self) -> $crate::dispatch::EntityDescriptor {
                $crate::dispatch::EntityDescriptor::Event {
                    cluster: <Self as $crate::traits::EventSpec>::CLUSTER_NAME,
                    cluster_id: Self::CLUSTER_ID.raw(),
                    event: Self::NAME,
                    event_id: Self::EVENT_ID.raw(),
                }
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                $is_empty
            }

            #[cfg(feature = "serde")]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_concrete_described(
                    deserializer,
                    $crate::dispatch::EntityDescriptor::Event {
                        cluster: <Self as $crate::traits::EventSpec>::CLUSTER_NAME,
                        cluster_id: Self::CLUSTER_ID.raw(),
                        event: Self::NAME,
                        event_id: Self::EVENT_ID.raw(),
                    },
                    $is_empty,
                )
            }
        }
    };
}

macro_rules! impl_cluster_wire_conversions {
    ($cluster:ident) => {
        impl From<$cluster> for $crate::model::Cluster {
            fn from(val: $cluster) -> Self {
                let attrs = val
                    .into_attributes()
                    .into_iter()
                    .map(|(id, value)| $crate::model::Attribute::new(id, value))
                    .collect();
                Self::new($cluster::CLUSTER_ID, attrs)
            }
        }

        impl TryFrom<$crate::model::Cluster> for $cluster {
            type Error = $crate::error::ClusterError;

            fn try_from(cluster: $crate::model::Cluster) -> Result<Self, Self::Error> {
                if cluster.id != Self::CLUSTER_ID {
                    return Err($crate::error::ClusterError::ClusterIdMismatch {
                        expected: Self::CLUSTER_ID,
                        actual: cluster.id,
                    });
                }
                Self::decode_attributes_inner(&mut cluster.attributes.into_iter().map(|attr| (attr.id, attr.value)))
            }
        }
    };
}

macro_rules! impl_matter_describe {
    (cluster, $ty:ident, $is_empty:expr) => {
        $crate::impl_matter_describe!(
            @concrete $ty,
            $crate::dispatch::EntityDescriptor::Cluster {
                cluster: <$ty>::NAME,
                cluster_id: <$ty>::CLUSTER_ID.raw(),
            },
            $is_empty
        );
    };
    (device, $ty:ident, $is_empty:expr) => {
        $crate::impl_matter_describe!(
            @concrete $ty,
            $crate::dispatch::EntityDescriptor::Device {
                device_type: <$ty>::NAME,
                device_type_id: <$ty>::DEVICE_TYPE_ID.raw(),
            },
            $is_empty
        );
    };
    (@concrete $ty:ident, $desc:expr, $is_empty:expr) => {
        #[cfg(feature = "dispatch")]
        impl $crate::dispatch::MatterDescribe for $ty {
            const STATIC_SCHEMA: $crate::dispatch::StaticEntitySchema =
                $crate::dispatch::StaticEntitySchema::concrete($desc, $is_empty);

            #[inline]
            fn descriptor(&self) -> $crate::dispatch::EntityDescriptor {
                $desc
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                $is_empty
            }

            #[cfg(feature = "serde")]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::dispatch::deserialize_concrete_described(deserializer, $desc, $is_empty)
            }
        }
    };
}

macro_rules! impl_select_described {
    (attr, $kind:literal) => {
        $crate::impl_select_described!(@tuple3 $kind);
    };
    (ev, $kind:literal) => {
        $crate::impl_select_described!(@tuple3 $kind);
    };
    (cmd, $kind:literal) => {
        #[cfg(all(feature = "dispatch", feature = "serde"))]
        pub(crate) fn select_described(
            item_name: Option<&str>,
            item_id: Option<u32>,
            payload: $crate::dispatch::BufferedPayload,
        ) -> Result<Self, alloc::string::String> {
            let deserialize = match (item_name, item_id) {
                (Some(name), maybe_id) => {
                    let (_, id, _, f) = Self::DESCRIBED_VARIANTS
                        .iter()
                        .copied()
                        .find(|&(n, _, _, _)| n == name)
                        .ok_or_else(|| alloc::format!("unknown {} '{name}'", $kind))?;
                    if let Some(expected_id) = maybe_id {
                        if id != expected_id {
                            return Err(alloc::format!(
                                "conflicting {} '{name}' and {}_id {expected_id}",
                                $kind,
                                $kind
                            ));
                        }
                    }
                    f
                }
                (None, Some(id)) => Self::DESCRIBED_VARIANTS
                    .iter()
                    .copied()
                    .find(|&(_, vid, _, _)| vid == id)
                    .map(|(_, _, _, f)| f)
                    .ok_or_else(|| alloc::format!("unknown {}_id {id}", $kind))?,
                (None, None) => return Err(alloc::format!("missing {} discriminator", $kind)),
            };

            deserialize(payload)
        }
    };
    (@tuple3 $kind:literal) => {
        #[cfg(all(feature = "dispatch", feature = "serde"))]
        pub(crate) fn select_described(
            item_name: Option<&str>,
            item_id: Option<u32>,
            payload: $crate::dispatch::BufferedPayload,
        ) -> Result<Self, alloc::string::String> {
            let deserialize = match (item_name, item_id) {
                (Some(name), maybe_id) => {
                    let (_, id, f) = Self::DESCRIBED_VARIANTS
                        .iter()
                        .copied()
                        .find(|&(n, _, _)| n == name)
                        .ok_or_else(|| alloc::format!("unknown {} '{name}'", $kind))?;
                    if let Some(expected_id) = maybe_id {
                        if id != expected_id {
                            return Err(alloc::format!(
                                "conflicting {} '{name}' and {}_id {expected_id}",
                                $kind,
                                $kind
                            ));
                        }
                    }
                    f
                }
                (None, Some(id)) => Self::DESCRIBED_VARIANTS
                    .iter()
                    .copied()
                    .find(|&(_, vid, _)| vid == id)
                    .map(|(_, _, f)| f)
                    .ok_or_else(|| alloc::format!("unknown {}_id {id}", $kind))?,
                (None, None) => return Err(alloc::format!("missing {} discriminator", $kind)),
            };

            deserialize(payload)
        }
    };
}

pub(crate) use define_matter_bitmap;
pub(crate) use define_matter_enum;
pub(crate) use define_matter_semantic_bool;
pub(crate) use impl_cluster_member_variant;
pub(crate) use impl_cluster_wire_conversions;
pub(crate) use impl_matter_attribute;
pub(crate) use impl_matter_command;
pub(crate) use impl_matter_describe;
pub(crate) use impl_matter_event;
pub(crate) use impl_select_described;
