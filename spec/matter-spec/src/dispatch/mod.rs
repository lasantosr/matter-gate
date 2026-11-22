//! Runtime Matter metadata traits and self-describing JSON wrappers.
//!
//! This module provides two layers of abstraction over [`matter-data-model`](crate::model)
//! and generated [`matter-spec`](crate) types:
//!
//! 1. **Runtime Metadata Traits:** [`ClusterMeta`], [`CommandMeta`], [`AttributeMeta`], [`EventMeta`], and
//!    [`DeviceMeta`] (automatically implemented for all [`ClusterSpec`](crate::traits::ClusterSpec),
//!    [`CommandSpec`](crate::traits::CommandSpec), [`AttributeSpec`](crate::traits::AttributeSpec),
//!    [`EventSpec`](crate::traits::EventSpec), and [`DeviceTypeSpec`](crate::traits::DeviceTypeSpec) types), unified
//!    under [`MatterDescribe`].
//! 2. **Self-Describing JSON Wrappers:** [`Described<T>`] and [`Invocation<T>`], which serialize Matter entities with
//!    both `PascalCase` names and numeric IDs, validate metadata on deserialization, and emit `"const"` constraints in
//!    `schemars` JSON schemas.

#[cfg(feature = "serde")]
mod buffer;
mod envelopes;
#[cfg(test)]
mod tests;
mod traits;

pub use envelopes::*;
pub use traits::*;

/// Implements `From<$local> for $global` and `TryFrom<$global> for $local` for a global dispatch enum variant.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_global_variant {
    (@tuple $local:ty, $global:path, $variant:ident, $err:ty, $on_other:expr, $on_none:expr) => {
        impl From<$local> for $global {
            #[inline]
            fn from(value: $local) -> Self {
                Self::$variant(value)
            }
        }

        impl TryFrom<$global> for $local {
            type Error = $err;

            #[inline]
            fn try_from(value: $global) -> Result<Self, Self::Error> {
                use $global as __Global;
                match Some(value) {
                    Some(__Global::$variant(inner)) => Ok(inner),
                    Some(other) => Err(($on_other)(other)),
                    None => Err($on_none),
                }
            }
        }
    };
    (@unit $local:ty, $global:path, $variant:ident, $err:ty, $on_other:expr, $on_none:expr) => {
        impl From<$local> for $global {
            #[inline]
            fn from(_: $local) -> Self {
                Self::$variant
            }
        }

        impl TryFrom<$global> for $local {
            type Error = $err;

            #[inline]
            fn try_from(value: $global) -> Result<Self, Self::Error> {
                use $global as __Global;
                match Some(value) {
                    Some(__Global::$variant) => Ok(Self),
                    Some(other) => Err(($on_other)(other)),
                    None => Err($on_none),
                }
            }
        }
    };
    (command, $local:ty, $global:path, $variant:ident) => {
        $crate::impl_global_variant!(
            @tuple
            $local,
            $global,
            $variant,
            $crate::error::CommandError,
            |other: $global| $crate::error::CommandError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::dispatch::CommandMeta::cluster_id(&other),
            },
            $crate::error::CommandError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::model::ClusterId::new(0),
            }
        );
    };
    (attribute, $local:ty, $global:path, $variant:ident) => {
        $crate::impl_global_variant!(
            @tuple
            $local,
            $global,
            $variant,
            $crate::error::AttributeError,
            |other: $global| $crate::error::AttributeError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::dispatch::AttributeMeta::cluster_id(&other),
            },
            $crate::error::AttributeError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::model::ClusterId::new(0),
            }
        );
    };
    (event, $local:ty, $global:path, $variant:ident) => {
        $crate::impl_global_variant!(
            @tuple
            $local,
            $global,
            $variant,
            $crate::error::EventError,
            |other: $global| $crate::error::EventError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::dispatch::EventMeta::cluster_id(&other),
            },
            $crate::error::EventError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::model::ClusterId::new(0),
            }
        );
    };
    (cluster, $local:ty, $global:path, $variant:ident) => {
        $crate::impl_global_variant!(
            @tuple
            $local,
            $global,
            $variant,
            $crate::error::ClusterError,
            |other: $global| $crate::error::ClusterError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::dispatch::ClusterMeta::cluster_id(&other),
            },
            $crate::error::ClusterError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::model::ClusterId::new(0),
            }
        );
    };
    (cluster_unit, $local:ty, $global:path, $variant:ident) => {
        $crate::impl_global_variant!(
            @unit
            $local,
            $global,
            $variant,
            $crate::error::ClusterError,
            |other: $global| $crate::error::ClusterError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::dispatch::ClusterMeta::cluster_id(&other),
            },
            $crate::error::ClusterError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: $crate::model::ClusterId::new(0),
            }
        );
    };
    (endpoint, $local:ty, $global:path, $variant:ident) => {
        $crate::impl_global_variant!(
            @tuple
            $local,
            $global,
            $variant,
            $crate::error::EndpointError,
            |other: $global| $crate::error::EndpointError::DeviceTypeMismatch {
                expected: Self::DEVICE_TYPE_ID,
                actual: $crate::dispatch::DeviceMeta::device_type_id(&other),
            },
            $crate::error::EndpointError::DeviceTypeMismatch {
                expected: Self::DEVICE_TYPE_ID,
                actual: $crate::model::DeviceTypeId::new(0),
            }
        );
    };
    (endpoint_unit, $local:ty, $global:path, $variant:ident) => {
        $crate::impl_global_variant!(
            @unit
            $local,
            $global,
            $variant,
            $crate::error::EndpointError,
            |other: $global| $crate::error::EndpointError::DeviceTypeMismatch {
                expected: Self::DEVICE_TYPE_ID,
                actual: $crate::dispatch::DeviceMeta::device_type_id(&other),
            },
            $crate::error::EndpointError::DeviceTypeMismatch {
                expected: Self::DEVICE_TYPE_ID,
                actual: $crate::model::DeviceTypeId::new(0),
            }
        );
    };
}
