//! **The universal, zero-bloat wire vocabulary for Matter in Rust.**
//!
//! `matter-data-model` models the core [Matter](https://csa-iot.org/all-solutions/matter/) hierarchy (nodes,
//! endpoints, clusters, attributes, commands, events, typed identifiers, and Interaction Model status codes) as
//! clean, `#![no_std]` data structures.
//!
//! # Overview
//!
//! Protocol stacks, bridges, and drivers need a shared language to exchange Matter state without pulling in an
//! entire network engine or breaking whenever a new cluster specification lands.
//!
//! `matter-data-model` is that foundation. It represents the Connectivity Standards Alliance (CSA) Data Model and
//! Interaction Model status codes as plain data: no custom traits, no cluster or schema conformance validation, no
//! Interaction Model message/path Information Blocks, and zero dependencies on async runtimes or transport stacks.
//!
//! Pair it with [`matter-spec`](https://docs.rs/matter-spec) for strongly typed, versioned cluster and endpoint
//! models that convert to and from these wire types via standard [`From`] and [`TryFrom`] implementations.
//!
//! # Features
//!
//! - **Complete wire hierarchy.** Clean data structures, fluent builders, and in-place mutation helpers representing
//!   Matter nodes, endpoints (including `parts` and semantic `tags`), clusters, attributes, commands, and events.
//! - **Zero-cost typed identifiers.** Transparent newtypes for node, endpoint, cluster, and attribute IDs (with
//!   Manufacturer Extension Identifier helpers) that prevent mixing up raw integers at compile time.
//! - **Universal wire value.** [`AttributeValue`] covers every Matter primitive and collection, paired with
//!   [`Optional<T>`](optional::Optional) and [`Nullable<T>`](optional::Nullable).
//! - **Normative status codes.** Direct byte-accurate mappings for Interaction Model status codes ([`MatterStatus`]).
//! - **`#![no_std]` ready.** Uses `alloc` and compiles identically on bare-metal microcontrollers and Linux hosts.
//!
//! # Cargo Features
//!
//! | Feature | Default | Description |
//! | :--- | :---: | :--- |
//! | `std` | Yes | Standard library support. |
//! | `serde` | No | `Serialize` and `Deserialize` implementations for all types (targets self-describing keyed formats). |
//! | `schemars` | No | `JsonSchema` generation via `schemars` (enables `std`). |
//! | `deny-unknown-fields` | No | Disallow unrecognized fields in Serde deserialization and Schemars JSON schemas (takes effect when `serde` or `schemars` is enabled). **Library crates must NOT enable this feature**, leave it for leaf binary crates to opt in. |
//!
//! # Examples
//!
//! ```rust
//! use matter_data_model::*;
//!
//! // Application-specific identifiers (from Matter Core Spec / Application Cluster Spec / Device Library)
//! const ROOT_NODE_DEVICE_TYPE: DeviceTypeId = DeviceTypeId::new(0x0016);
//! const ON_OFF_LIGHT_DEVICE_TYPE: DeviceTypeId = DeviceTypeId::new(0x0100);
//! const BASIC_INFORMATION_CLUSTER: ClusterId = ClusterId::new(0x0028);
//! const ON_OFF_CLUSTER: ClusterId = ClusterId::new(0x0006);
//!
//! // Assemble a Matter node with a Root Node endpoint (0) and an On/Off Light endpoint (1).
//! let node = Node::new(NodeId::new(0x0000_0000_1234_5678), Vec::new())
//!     .with_endpoint(Endpoint::single_device(
//!         EndpointId::ROOT,
//!         ROOT_NODE_DEVICE_TYPE,
//!         1,
//!         vec![Cluster::empty(BASIC_INFORMATION_CLUSTER)],
//!         Vec::new(),
//!     ))
//!     .with_endpoint(Endpoint::single_device(
//!         EndpointId::new(1),
//!         ON_OFF_LIGHT_DEVICE_TYPE,
//!         2,
//!         vec![
//!             Cluster::empty(ON_OFF_CLUSTER)
//!                 .with_data_version(DataVersion::new(1))
//!                 .with_attribute(AttributeId::new(0), true),
//!         ],
//!         Vec::new(),
//!     ));
//!
//! assert_eq!(node.endpoints.len(), 2);
//! assert!(node.root_endpoint().is_some());
//!
//! let light = node.endpoint(EndpointId::new(1)).unwrap();
//! let on_off = light.server_cluster(ON_OFF_CLUSTER).unwrap();
//! assert_eq!(
//!     on_off.attribute(AttributeId::new(0)),
//!     Some(&AttributeValue::Bool(true))
//! );
//! ```

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

macro_rules! define_matter_id {
    (
        $(#[$meta:meta])*
        pub struct $name:ident($inner:ident);
        display = $display:ident;
        mei = $mei:tt;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[repr(transparent)]
        pub struct $name($inner);

        impl $name {
            #[doc = concat!("Construct a new [`", stringify!($name), "`] from a raw `", stringify!($inner), "` value.")]
            #[must_use]
            pub const fn new(raw: $inner) -> Self {
                Self(raw)
            }

            #[doc = concat!("Return the raw `", stringify!($inner), "` value.")]
            #[must_use]
            pub const fn raw(self) -> $inner {
                self.0
            }
        }

        define_matter_id!(@mei $name, $mei);
        define_matter_id!(@display $name, $inner, $display);

        impl From<$inner> for $name {
            fn from(raw: $inner) -> Self {
                Self(raw)
            }
        }

        impl From<$name> for $inner {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        define_matter_id!(@attr_value $name, $inner);

        impl core::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::LowerHex::fmt(&self.0, f)
            }
        }

        impl core::fmt::UpperHex for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::UpperHex::fmt(&self.0, f)
            }
        }

        impl core::str::FromStr for $name {
            type Err = core::num::ParseIntError;

            fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
                let raw = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    if hex.starts_with('+') {
                        "".parse::<$inner>()?;
                    }
                    <$inner>::from_str_radix(hex, 16)?
                } else {
                    if s.starts_with('+') {
                        "".parse::<$inner>()?;
                    }
                    s.parse::<$inner>()?
                };
                Ok(Self(raw))
            }
        }
    };

    (@mei $name:ident, true) => {
        impl $name {
            /// Construct a 32-bit Matter Extended Identifier (MEI) from a
            /// [`VendorId`](crate::types::VendorId) and 16-bit sub-ID.
            #[must_use]
            pub const fn from_mei(vendor_id: crate::types::VendorId, sub_id: u16) -> Self {
                let [s0, s1] = sub_id.to_le_bytes();
                let [v0, v1] = vendor_id.raw().to_le_bytes();
                Self(u32::from_le_bytes([s0, s1, v0, v1]))
            }

            /// Extract the 16-bit [`VendorId`](crate::types::VendorId) prefix from bits `31..16`.
            #[must_use]
            pub const fn vendor_id(self) -> crate::types::VendorId {
                let [b0, b1, _, _] = (self.0 >> 16).to_le_bytes();
                crate::types::VendorId::new(u16::from_le_bytes([b0, b1]))
            }

            /// Extract the 16-bit scoped sub-ID from bits `15..0`.
            #[must_use]
            pub const fn sub_id(self) -> u16 {
                let [b0, b1, _, _] = self.0.to_le_bytes();
                u16::from_le_bytes([b0, b1])
            }

            /// Return `true` if this is a standard (non-manufacturer-specific) identifier (`vendor_id == 0`).
            #[must_use]
            pub const fn is_standard(self) -> bool {
                (self.0 >> 16) == 0
            }

            /// Return `true` if this is a manufacturer-specific identifier (`vendor_id != 0` and not `0xFFFF`).
            #[must_use]
            pub const fn is_manufacturer_specific(self) -> bool {
                let vendor = self.0 >> 16;
                vendor != 0 && vendor != 0xFFFF
            }
        }
    };
    (@mei $name:ident, false) => {};

    (@display $name:ident, $inner:ident, decimal) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
    (@display $name:ident, $inner:ident, hex) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    f,
                    "{:#0width$x}",
                    self.0,
                    width = core::mem::size_of::<$inner>().saturating_mul(2).saturating_add(2)
                )
            }
        }
    };
    (@display $name:ident, $inner:ident, hex_32_or_16) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                if self.0 > 0xFFFF {
                    write!(f, "{:#010x}", self.0)
                } else {
                    write!(f, "{:#06x}", self.0)
                }
            }
        }
    };
    (@display $name:ident, $inner:ident, hex_32_16_or_8) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                if self.0 > 0xFFFF {
                    write!(f, "{:#010x}", self.0)
                } else if self.0 > 0xFF {
                    write!(f, "{:#06x}", self.0)
                } else {
                    write!(f, "{:#04x}", self.0)
                }
            }
        }
    };

    (@attr_value $name:ident, u8) => {
        define_matter_id!(@attr_value_impl $name, U8);
    };
    (@attr_value $name:ident, u16) => {
        define_matter_id!(@attr_value_impl $name, U16);
    };
    (@attr_value $name:ident, u32) => {
        define_matter_id!(@attr_value_impl $name, U32);
    };
    (@attr_value $name:ident, u64) => {
        define_matter_id!(@attr_value_impl $name, U64);
    };
    (@attr_value_impl $name:ident, $variant:ident) => {
        impl From<$name> for crate::value::AttributeValue {
            fn from(val: $name) -> Self {
                Self::$variant(val.raw())
            }
        }

        impl TryFrom<&crate::value::AttributeValue> for $name {
            type Error = crate::status::MatterStatus;

            fn try_from(
                val: &crate::value::AttributeValue,
            ) -> core::result::Result<Self, crate::status::MatterStatus> {
                match val {
                    crate::value::AttributeValue::$variant(v) => Ok(Self::new(*v)),
                    _ => Err(crate::status::MatterStatus::InvalidDataType),
                }
            }
        }

        impl TryFrom<crate::value::AttributeValue> for $name {
            type Error = crate::status::MatterStatus;

            fn try_from(
                val: crate::value::AttributeValue,
            ) -> core::result::Result<Self, crate::status::MatterStatus> {
                (&val).try_into()
            }
        }
    };

}

mod attribute;
mod cluster;
mod command;
mod endpoint;
mod event;
mod node;
mod optional;
mod status;
mod tag;
mod types;
mod value;

/// Serde deserialization helpers.
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod serde;

pub use self::{
    attribute::*,
    cluster::*,
    command::*,
    endpoint::*,
    event::*,
    node::*,
    optional::{
        Nullable::{NonNull, Null},
        Optional::{Omitted, Present},
        *,
    },
    status::*,
    tag::*,
    types::*,
    value::*,
};
