//! Declarative macros for cluster member variants, attributes, commands, events, and wire conversions.

/// Implements `From` and `TryFrom` conversions between a cluster member struct and its parent cluster enum.
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
    (attribute_single, $enum_ty:path, $member_ty:ty, $variant:ident) => {
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
                match attr {
                    __Enum::$variant(inner) => Ok(inner),
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
    (command_single, $enum_ty:path, $member_ty:ty, $variant:ident) => {
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
                match cmd {
                    __Enum::$variant(inner) => Ok(inner),
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
    (command_unit_single, $enum_ty:path, $member_ty:ty, $variant:ident) => {
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
                match cmd {
                    __Enum::$variant => Ok(<$member_ty>::default()),
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
    (event_single, $enum_ty:path, $member_ty:ty, $variant:ident) => {
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
                match ev {
                    __Enum::$variant(inner) => Ok(inner),
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
    (event_unit_single, $enum_ty:path, $member_ty:ty, $variant:ident) => {
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
                match ev {
                    __Enum::$variant => Ok(<$member_ty>::default()),
                }
            }
        }
    };
}

/// Implements wrapper helpers, `into_value`, `Display`, wire conversions, `AttributeSpec`, and `MatterDescribe` for an
/// attribute struct.
macro_rules! impl_matter_attribute {
    ($name:ident, $base:ty, $cluster:ty, $copy_mode:ident, $val_mode:ident, $disp_mode:ident) => {
        $crate::impl_matter_attribute!(@inner_accessors $name, $base, $copy_mode);
        $crate::impl_matter_attribute!(@into_value required, $name, $val_mode);
        $crate::impl_matter_attribute!(@display required, $name, $disp_mode);
        $crate::impl_matter_attribute!(@common $name, $base, $cluster);
    };
    (nullable, $name:ident, $base:ty, $cluster:ty, $copy_mode:ident, $val_mode:ident, $disp_mode:ident) => {
        $crate::impl_matter_attribute!(@inner_accessors $name, $crate::model::Nullable<$base>, $copy_mode);
        $crate::impl_matter_attribute!(@into_value nullable, $name, $val_mode);
        $crate::impl_matter_attribute!(@display nullable, $name, $disp_mode);

        impl From<$base> for $name {
            #[inline]
            fn from(val: $base) -> Self {
                Self($crate::model::Nullable::NonNull(val))
            }
        }

        impl From<Option<$base>> for $name {
            #[inline]
            fn from(val: Option<$base>) -> Self {
                Self($crate::model::Nullable::from(val))
            }
        }

        $crate::impl_matter_attribute!(@common $name, $crate::model::Nullable<$base>, $cluster);
    };
    (@inner_accessors $name:ident, $inner:ty, copy) => {
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
    };
    (@inner_accessors $name:ident, $inner:ty, owned) => {
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
    };
    (@into_value required, $name:ident, scalar) => {
        impl $name {
            #[must_use]
            pub fn into_value(self) -> $crate::model::AttributeValue {
                $crate::model::AttributeValue::from(self.0)
            }
        }
    };
    (@into_value required, $name:ident, list) => {
        impl $name {
            #[must_use]
            pub fn into_value(self) -> $crate::model::AttributeValue {
                let items = self.0.into_iter().map($crate::model::AttributeValue::from).collect();
                $crate::model::AttributeValue::List(items)
            }
        }
    };
    (@into_value nullable, $name:ident, scalar) => {
        impl $name {
            #[must_use]
            pub fn into_value(self) -> $crate::model::AttributeValue {
                match self.0 {
                    $crate::model::Nullable::NonNull(val) => $crate::model::AttributeValue::from(val),
                    $crate::model::Nullable::Null => $crate::model::AttributeValue::Null,
                }
            }
        }
    };
    (@into_value nullable, $name:ident, list) => {
        impl $name {
            #[must_use]
            pub fn into_value(self) -> $crate::model::AttributeValue {
                match self.0 {
                    $crate::model::Nullable::NonNull(list) => {
                        let items = list.into_iter().map($crate::model::AttributeValue::from).collect();
                        $crate::model::AttributeValue::List(items)
                    }
                    $crate::model::Nullable::Null => $crate::model::AttributeValue::Null,
                }
            }
        }
    };
    (@display required, $name:ident, display) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
    (@display required, $name:ident, debug) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
    };
    (@display required, $name:ident, list) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "<list of {} items>", self.0.len())
            }
        }
    };
    (@display required, $name:ident, bytes) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "<{} bytes>", self.0.len())
            }
        }
    };
    (@display nullable, $name:ident, display) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match &self.0 {
                    $crate::model::Nullable::NonNull(val) => write!(f, "{val}"),
                    $crate::model::Nullable::Null => write!(f, "null"),
                }
            }
        }
    };
    (@display nullable, $name:ident, debug) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match &self.0 {
                    $crate::model::Nullable::NonNull(struct_val) => write!(f, "{struct_val:?}"),
                    $crate::model::Nullable::Null => write!(f, "null"),
                }
            }
        }
    };
    (@display nullable, $name:ident, list) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match &self.0 {
                    $crate::model::Nullable::NonNull(list) => write!(f, "<list of {} items>", list.len()),
                    $crate::model::Nullable::Null => write!(f, "null"),
                }
            }
        }
    };
    (@display nullable, $name:ident, bytes) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match &self.0 {
                    $crate::model::Nullable::NonNull(bytes_val) => write!(f, "<{} bytes>", bytes_val.len()),
                    $crate::model::Nullable::Null => write!(f, "null"),
                }
            }
        }
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

/// Implements [`CommandSpec`](crate::traits::CommandSpec), wire conversions, field/value helpers, and `MatterDescribe` for a command type.
macro_rules! impl_matter_command {
    ($name:ident, $cluster:ty, $response:ty, true) => {
        impl $name {
            #[inline]
            #[must_use]
            pub fn into_fields(self) -> alloc::vec::Vec<($crate::model::FieldId, $crate::model::AttributeValue)> {
                alloc::vec::Vec::new()
            }

            #[inline]
            pub(crate) fn decode_fields(
                fields: alloc::vec::Vec<($crate::model::FieldId, $crate::model::AttributeValue)>,
            ) -> Result<Self, (Option<$crate::model::FieldId>, $crate::model::MatterStatus)> {
                if let Some((fid, _)) = fields.first() {
                    Err((Some(*fid), $crate::model::MatterStatus::ConstraintError))
                } else {
                    Ok(Self::default())
                }
            }

            #[inline]
            pub fn try_from_value(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                match val {
                    $crate::model::AttributeValue::Null => Ok(Self::default()),
                    $crate::model::AttributeValue::Struct(fields) => Self::try_from_fields(fields),
                    _ => Err($crate::model::MatterStatus::InvalidDataType),
                }
            }
        }

        $crate::impl_matter_command!(@common $name, $cluster, $response, true);
    };
    ($name:ident, $cluster:ty, $response:ty, false) => {
        impl $name {
            #[inline]
            pub fn try_from_value(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                let $crate::model::AttributeValue::Struct(fields) = val else {
                    return Err($crate::model::MatterStatus::InvalidDataType);
                };
                Self::try_from_fields(fields)
            }
        }

        $crate::impl_matter_command!(@common $name, $cluster, $response, false);
    };
    (@common $name:ident, $cluster:ty, $response:ty, $is_empty:expr) => {
        impl $name {
            #[inline]
            #[must_use]
            pub fn into_value(self) -> $crate::model::AttributeValue {
                $crate::model::AttributeValue::Struct(self.into_fields())
            }

            #[inline]
            pub fn try_from_fields(
                fields: alloc::vec::Vec<($crate::model::FieldId, $crate::model::AttributeValue)>,
            ) -> Result<Self, $crate::model::MatterStatus> {
                Self::decode_fields(fields).map_err(|(_, status)| status)
            }
        }

        impl $crate::traits::CommandSpec for $name {
            const CLUSTER_ID: $crate::model::ClusterId = Self::CLUSTER_ID;
            const CLUSTER_NAME: &'static str = <$cluster>::NAME;
            const COMMAND_ID: $crate::model::CommandId = Self::COMMAND_ID;
            const COMMAND_NAME: &'static str = Self::NAME;
            const DIRECTION: $crate::model::CommandDirection = Self::DIRECTION;
            const IS_EMPTY: bool = $is_empty;
            type Response = $response;
        }

        impl From<$name> for $crate::model::Command {
            #[inline]
            fn from(cmd: $name) -> Self {
                Self::new(
                    $name::CLUSTER_ID,
                    $name::COMMAND_ID,
                    $name::DIRECTION,
                    cmd.into_fields(),
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
                Self::decode_fields(cmd.payload).map_err(|(field_id, status)| {
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

/// Implements [`EventSpec`](crate::traits::EventSpec), wire conversions, field/value helpers, and `MatterDescribe` for an event type.
macro_rules! impl_matter_event {
    ($name:ident, $cluster:ty, true) => {
        impl $name {
            #[inline]
            #[must_use]
            pub fn into_fields(self) -> alloc::vec::Vec<($crate::model::FieldId, $crate::model::AttributeValue)> {
                alloc::vec::Vec::new()
            }

            #[inline]
            pub(crate) fn decode_fields(
                fields: alloc::vec::Vec<($crate::model::FieldId, $crate::model::AttributeValue)>,
            ) -> Result<Self, (Option<$crate::model::FieldId>, $crate::model::MatterStatus)> {
                if let Some((fid, _)) = fields.first() {
                    Err((Some(*fid), $crate::model::MatterStatus::ConstraintError))
                } else {
                    Ok(Self::default())
                }
            }

            #[inline]
            pub fn try_from_value(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                match val {
                    $crate::model::AttributeValue::Null => Ok(Self::default()),
                    $crate::model::AttributeValue::Struct(fields) => Self::try_from_fields(fields),
                    _ => Err($crate::model::MatterStatus::InvalidDataType),
                }
            }
        }

        $crate::impl_matter_event!(@common $name, $cluster, true);
    };
    ($name:ident, $cluster:ty, false) => {
        impl $name {
            #[inline]
            pub fn try_from_value(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                let $crate::model::AttributeValue::Struct(fields) = val else {
                    return Err($crate::model::MatterStatus::InvalidDataType);
                };
                Self::try_from_fields(fields)
            }
        }

        $crate::impl_matter_event!(@common $name, $cluster, false);
    };
    (@common $name:ident, $cluster:ty, $is_empty:expr) => {
        impl $name {
            #[inline]
            #[must_use]
            pub fn into_value(self) -> $crate::model::AttributeValue {
                $crate::model::AttributeValue::Struct(self.into_fields())
            }

            #[inline]
            pub fn try_from_fields(
                fields: alloc::vec::Vec<($crate::model::FieldId, $crate::model::AttributeValue)>,
            ) -> Result<Self, $crate::model::MatterStatus> {
                Self::decode_fields(fields).map_err(|(_, status)| status)
            }
        }

        impl From<$name> for $crate::model::Event {
            #[inline]
            fn from(ev: $name) -> Self {
                Self::new(
                    $name::CLUSTER_ID,
                    $name::EVENT_ID,
                    $name::PRIORITY,
                    ev.into_fields(),
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
                Self::decode_fields(ev.payload).map_err(|(field_id, status)| {
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

/// Implements `From<Struct> for AttributeValue` and `TryFrom<AttributeValue> for Struct` via `encode_fields_inner` /
/// `decode_fields_inner`.
macro_rules! impl_matter_struct_wire {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        impl From<$name> for $crate::model::AttributeValue {
            #[inline]
            fn from(val: $name) -> Self {
                Self::Struct(val.encode_fields_inner())
            }
        }

        $(#[$meta])*
        impl TryFrom<$crate::model::AttributeValue> for $name {
            type Error = $crate::model::MatterStatus;

            #[inline]
            fn try_from(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                let $crate::model::AttributeValue::Struct(fields) = val else {
                    return Err($crate::model::MatterStatus::InvalidDataType);
                };
                Self::decode_fields_inner(fields).map_err(|(_, status)| status)
            }
        }
    };
}

/// Implements `From<Cluster> for model::Cluster` and `TryFrom<model::Cluster> for Cluster`.
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

pub(crate) use impl_cluster_member_variant;
pub(crate) use impl_cluster_wire_conversions;
pub(crate) use impl_matter_attribute;
pub(crate) use impl_matter_command;
pub(crate) use impl_matter_event;
pub(crate) use impl_matter_struct_wire;
