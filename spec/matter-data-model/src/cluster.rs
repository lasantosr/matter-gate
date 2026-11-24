//! Matter Cluster entity per CSA Matter Specification Section 7.7.

extern crate alloc;

use alloc::vec::Vec;
use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
};

use crate::{
    attribute::{Attribute, AttributeId},
    command::CommandId,
    optional::Optional,
    value::AttributeValue,
};

define_matter_id! {
    /// Strongly-typed Matter Cluster identifier (`u32`).
    pub struct ClusterId(u32);
    display = hex_32_or_16;
    mei = true;
}

impl ClusterId {
    /// Descriptor Cluster ID (`0x001D`) per CSA Matter Core Specification §9.5.
    /// Mandatory on every endpoint to expose device types, server clusters, client clusters, and parts list.
    pub const DESCRIPTOR: Self = Self(0x001D);
    /// Wildcard Cluster ID (`0xFFFF_FFFF`) per CSA Matter Interaction Model §10.5.
    pub const WILDCARD: Self = Self(0xFFFF_FFFF);

    /// Return `true` if this is the Wildcard cluster (`0xFFFF_FFFF`).
    #[must_use]
    pub const fn is_wildcard(self) -> bool {
        self.0 == 0xFFFF_FFFF
    }

    /// Return `true` if this cluster ID lies within CSA Matter Core Spec §7.18.2.37 / §2.5.7 allocation ranges
    /// (standard `0x0000..=0x7FFF`, or valid manufacturer vendor with `sub_id` in `0xFC00..=0xFFFE`).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        if self.is_standard() {
            self.sub_id() <= 0x7FFF
        } else {
            self.vendor_id().is_valid() && matches!(self.sub_id(), 0xFC00..=0xFFFE)
        }
    }
}

define_matter_id! {
    /// Strongly-typed Matter `DataVersion` identifier (`u32`).
    pub struct DataVersion(u32);
    display = decimal;
    mei = false;
}

/// Stateful wire representation of a Matter Cluster instance per Section 7.7.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct Cluster {
    /// Matter cluster identifier.
    pub id: ClusterId,
    /// Optional monotonic data version counter for change detection.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub data_version: Optional<DataVersion>,
    /// Attributes belonging to this cluster instance.
    pub attributes: Vec<Attribute>,
    /// Accepted command IDs supported by this cluster instance (CSA §7.13 `AcceptedCommandList`, `0xFFF9`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub accepted_commands: Vec<CommandId>,
    /// Generated command IDs emitted by this cluster instance (CSA §7.13 `GeneratedCommandList`, `0xFFF8`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub generated_commands: Vec<CommandId>,
}

impl Cluster {
    /// Construct a cluster with attributes, omitted data version, and empty command lists.
    ///
    /// Callers should provide attributes with unique [`AttributeId`]s; lookups and updates
    /// operate on the first matching ID.
    #[must_use]
    pub const fn new(id: ClusterId, attributes: Vec<Attribute>) -> Self {
        Self {
            id,
            data_version: Optional::Omitted,
            attributes,
            accepted_commands: Vec::new(),
            generated_commands: Vec::new(),
        }
    }

    /// Construct an empty cluster without attributes, omitted data version, and empty command lists.
    #[must_use]
    pub const fn empty(id: ClusterId) -> Self {
        Self {
            id,
            data_version: Optional::Omitted,
            attributes: Vec::new(),
            accepted_commands: Vec::new(),
            generated_commands: Vec::new(),
        }
    }

    /// Set or override the cluster identifier.
    #[must_use]
    pub const fn with_id(mut self, id: ClusterId) -> Self {
        self.id = id;
        self
    }

    /// Set the data version of this cluster.
    #[must_use]
    pub const fn with_data_version(mut self, version: DataVersion) -> Self {
        self.data_version = Optional::Present(version);
        self
    }

    /// Set or omit the optional data version of this cluster.
    #[must_use]
    pub fn with_optional_data_version(mut self, version: impl Into<Optional<DataVersion>>) -> Self {
        self.data_version = version.into();
        self
    }

    /// Add or update an attribute on this cluster.
    #[must_use]
    pub fn with_attribute(mut self, id: AttributeId, value: impl Into<AttributeValue>) -> Self {
        self.set_attribute(id, value);
        self
    }

    /// Add or update multiple attributes on this cluster.
    #[must_use]
    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = Attribute>) -> Self {
        for attr in attributes {
            self.set_attribute(attr.id, attr.value);
        }
        self
    }

    /// Add an accepted command ID to this cluster if not already present.
    #[must_use]
    pub fn with_accepted_command(mut self, id: CommandId) -> Self {
        self.set_accepted_command(id);
        self
    }

    /// Add multiple accepted command IDs to this cluster if not already present.
    #[must_use]
    pub fn with_accepted_commands(mut self, ids: impl IntoIterator<Item = CommandId>) -> Self {
        for id in ids {
            self.set_accepted_command(id);
        }
        self
    }

    /// Add a generated command ID to this cluster if not already present.
    #[must_use]
    pub fn with_generated_command(mut self, id: CommandId) -> Self {
        self.set_generated_command(id);
        self
    }

    /// Add multiple generated command IDs to this cluster if not already present.
    #[must_use]
    pub fn with_generated_commands(mut self, ids: impl IntoIterator<Item = CommandId>) -> Self {
        for id in ids {
            self.set_generated_command(id);
        }
        self
    }

    /// Return `true` if this cluster contains an attribute with the given ID.
    #[must_use]
    pub fn has_attribute(&self, id: AttributeId) -> bool {
        self.attributes.iter().any(|attr| attr.id == id)
    }

    /// Find an attribute's value by ID on this cluster.
    #[must_use]
    pub fn attribute(&self, id: AttributeId) -> Option<&AttributeValue> {
        self.attributes
            .iter()
            .find(|attr| attr.id == id)
            .map(|attr| &attr.value)
    }

    /// Return a mutable reference to an attribute's value by ID on this cluster.
    #[must_use]
    pub fn attribute_mut(&mut self, id: AttributeId) -> Option<&mut AttributeValue> {
        self.attributes
            .iter_mut()
            .find(|attr| attr.id == id)
            .map(|attr| &mut attr.value)
    }

    /// Insert or update an attribute on this cluster in place, returning the previous value if replaced.
    pub fn set_attribute(&mut self, id: AttributeId, value: impl Into<AttributeValue>) -> Option<AttributeValue> {
        let val = value.into();
        if let Some(existing) = self.attributes.iter_mut().find(|attr| attr.id == id) {
            Some(mem::replace(&mut existing.value, val))
        } else {
            self.attributes.push(Attribute::new(id, val));
            None
        }
    }

    /// Remove an attribute by ID from this cluster, returning its value if present.
    pub fn remove_attribute(&mut self, id: AttributeId) -> Option<AttributeValue> {
        self.attributes
            .iter()
            .position(|attr| attr.id == id)
            .map(|idx| self.attributes.remove(idx).value)
    }

    /// Return `true` if `accepted_commands` contains the given command ID.
    #[must_use]
    pub fn has_accepted_command(&self, id: CommandId) -> bool {
        self.accepted_commands.contains(&id)
    }

    /// Add an accepted command ID to this cluster in place if not already present, returning `true` if inserted.
    pub fn set_accepted_command(&mut self, id: CommandId) -> bool {
        if self.accepted_commands.contains(&id) {
            false
        } else {
            self.accepted_commands.push(id);
            true
        }
    }

    /// Remove an accepted command ID from this cluster, returning `true` if removed.
    pub fn remove_accepted_command(&mut self, id: CommandId) -> bool {
        if let Some(idx) = self.accepted_commands.iter().position(|&cmd_id| cmd_id == id) {
            self.accepted_commands.remove(idx);
            true
        } else {
            false
        }
    }

    /// Return `true` if `generated_commands` contains the given command ID.
    #[must_use]
    pub fn has_generated_command(&self, id: CommandId) -> bool {
        self.generated_commands.contains(&id)
    }

    /// Add a generated command ID to this cluster in place if not already present, returning `true` if inserted.
    pub fn set_generated_command(&mut self, id: CommandId) -> bool {
        if self.generated_commands.contains(&id) {
            false
        } else {
            self.generated_commands.push(id);
            true
        }
    }

    /// Remove a generated command ID from this cluster, returning `true` if removed.
    pub fn remove_generated_command(&mut self, id: CommandId) -> bool {
        if let Some(idx) = self.generated_commands.iter().position(|&cmd_id| cmd_id == id) {
            self.generated_commands.remove(idx);
            true
        } else {
            false
        }
    }
}

impl From<(ClusterId, Vec<Attribute>)> for Cluster {
    fn from((id, attributes): (ClusterId, Vec<Attribute>)) -> Self {
        Self {
            id,
            data_version: Optional::Omitted,
            attributes,
            accepted_commands: Vec::new(),
            generated_commands: Vec::new(),
        }
    }
}

impl Display for Cluster {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Cluster {} ({} attributes)", self.id, self.attributes.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VendorId;

    #[test]
    fn cluster_id_and_data_version() {
        assert_eq!(ClusterId::WILDCARD.raw(), 0xFFFF_FFFF);

        let cid = ClusterId::new(0x0006);
        assert_eq!(cid.raw(), 0x0006);
        assert_eq!(alloc::format!("{cid}"), "0x0006");
        assert_eq!(u32::from(cid), 0x0006);

        let dv = DataVersion::new(42);
        assert_eq!(dv.raw(), 42);
        assert_eq!(alloc::format!("{dv}"), "42");
        assert_eq!(u32::from(dv), 42);

        let val_cid = AttributeValue::from(cid);
        assert_eq!(ClusterId::try_from(&val_cid), Ok(cid));
        assert_eq!(ClusterId::try_from(val_cid), Ok(cid));

        let val_dv = AttributeValue::from(dv);
        assert_eq!(DataVersion::try_from(&val_dv), Ok(dv));
        assert_eq!(DataVersion::try_from(val_dv), Ok(dv));
    }

    #[test]
    fn cluster_construction_and_lookup() {
        const ON_OFF_CLUSTER_ID: ClusterId = ClusterId::new(0x0006);

        let cluster = Cluster::empty(ON_OFF_CLUSTER_ID)
            .with_data_version(DataVersion::new(1))
            .with_attribute(AttributeId::new(0), true);

        assert_eq!(cluster.id, ON_OFF_CLUSTER_ID);
        assert_eq!(cluster.data_version, Optional::Present(DataVersion::new(1)));
        assert_eq!(
            cluster.attribute(AttributeId::new(0)),
            Some(&AttributeValue::Bool(true))
        );
        assert_eq!(cluster.attribute(AttributeId::new(1)), None);

        // Update attribute
        let mut cluster = cluster.with_attribute(AttributeId::new(0), false);
        assert_eq!(
            cluster.attribute(AttributeId::new(0)),
            Some(&AttributeValue::Bool(false))
        );
        assert!(cluster.has_attribute(AttributeId::new(0)));

        // In-place mutation
        let prev = cluster.set_attribute(AttributeId::new(0), true);
        assert_eq!(prev, Some(AttributeValue::Bool(false)));
        if let Some(val) = cluster.attribute_mut(AttributeId::new(0)) {
            *val = AttributeValue::U8(99);
        }
        assert_eq!(cluster.attribute(AttributeId::new(0)), Some(&AttributeValue::U8(99)));
        assert_eq!(
            cluster.remove_attribute(AttributeId::new(0)),
            Some(AttributeValue::U8(99))
        );
        assert!(!cluster.has_attribute(AttributeId::new(0)));
    }

    #[test]
    fn mei_helpers() {
        use crate::types::VendorId;
        let mei = ClusterId::from_mei(VendorId::new(0xFFF1), 0x1234);
        assert_eq!(mei.vendor_id(), VendorId::new(0xFFF1));
        assert_eq!(mei.sub_id(), 0x1234);
        assert!(mei.is_manufacturer_specific());
        assert!(!mei.is_standard());

        let std_id = ClusterId::DESCRIPTOR;
        assert_eq!(std_id.vendor_id(), VendorId::new(0));
        assert!(std_id.is_standard());
        assert!(!std_id.is_manufacturer_specific());
        assert!(!std_id.is_wildcard());

        assert!(ClusterId::WILDCARD.is_wildcard());
        assert!(!ClusterId::WILDCARD.is_standard());
        assert!(!ClusterId::WILDCARD.is_manufacturer_specific());
    }

    #[test]
    fn remove_attribute_preserves_order_and_formatting() {
        let mut cluster = Cluster::empty(ClusterId::new(0x0006))
            .with_optional_data_version(Some(DataVersion::new(5)))
            .with_attribute(AttributeId::new(1), true)
            .with_attribute(AttributeId::new(2), 2_u8)
            .with_attribute(AttributeId::new(3), 3_u8);
        assert_eq!(cluster.data_version, Optional::Present(DataVersion::new(5)));
        assert_eq!(alloc::format!("{cluster}"), "Cluster 0x0006 (3 attributes)");

        assert_eq!(
            cluster.remove_attribute(AttributeId::new(2)),
            Some(AttributeValue::U8(2))
        );
        let ids: Vec<u32> = cluster.attributes.iter().map(|a| a.id.raw()).collect();
        assert_eq!(ids, alloc::vec![1, 3]);

        assert_eq!("0x0006".parse::<ClusterId>(), Ok(ClusterId::new(0x0006)));
        assert_eq!("42".parse::<DataVersion>(), Ok(DataVersion::new(42)));
        assert_eq!(alloc::format!("{:04X}", ClusterId::new(0x001D)), "001D");

        let renamed = cluster.with_id(ClusterId::DESCRIPTOR);
        assert_eq!(renamed.id, ClusterId::DESCRIPTOR);
        assert!(ClusterId::DESCRIPTOR.is_valid());
        assert!(!ClusterId::new(0x8000).is_valid());
        assert!(ClusterId::from_mei(VendorId::new(0xFFF1), 0xFC00).is_valid());
        assert!(!ClusterId::from_mei(VendorId::new(0xFFF1), 0x0001).is_valid());
        assert!(!ClusterId::WILDCARD.is_valid());
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn cluster_json_schema_marks_optional_fields() {
        let mut generator = schemars::SchemaGenerator::default();
        let schema = <Cluster as schemars::JsonSchema>::json_schema(&mut generator);
        let required = schema
            .as_object()
            .and_then(|o| o.get("required"))
            .and_then(|v| v.as_array());
        assert!(
            required.is_some_and(|req| {
                !req.iter().any(|v| {
                    matches!(
                        v.as_str(),
                        Some("data_version" | "accepted_commands" | "generated_commands")
                    )
                })
            }),
            "data_version, accepted_commands, and generated_commands must not be required in Cluster JSON schema"
        );
    }

    #[test]
    fn cluster_command_lists_default_empty() {
        let c1 = Cluster::new(ClusterId::new(0x0006), alloc::vec![]);
        assert_eq!(c1.accepted_commands, []);
        assert_eq!(c1.generated_commands, []);

        let c2 = Cluster::empty(ClusterId::new(0x0006));
        assert_eq!(c2.accepted_commands, []);
        assert_eq!(c2.generated_commands, []);

        let c3 = Cluster::default();
        assert_eq!(c3.accepted_commands, []);
        assert_eq!(c3.generated_commands, []);

        let c4 = Cluster::from((ClusterId::new(0x0006), alloc::vec![]));
        assert_eq!(c4.accepted_commands, []);
        assert_eq!(c4.generated_commands, []);
    }

    #[test]
    fn cluster_command_builders_deduplicate_and_preserve_order() {
        let cmd_off = CommandId::new(0x00);
        let cmd_on = CommandId::new(0x01);
        let cmd_toggle = CommandId::new(0x02);
        let gen_resp = CommandId::new(0x00);
        let gen_other = CommandId::new(0x01);

        let mut cluster = Cluster::empty(ClusterId::new(0x0006))
            .with_accepted_command(cmd_on)
            .with_accepted_commands([cmd_off, cmd_on, cmd_toggle, cmd_off])
            .with_generated_command(gen_other)
            .with_generated_commands([gen_resp, gen_other]);

        assert_eq!(cluster.accepted_commands, alloc::vec![cmd_on, cmd_off, cmd_toggle]);
        assert_eq!(cluster.generated_commands, alloc::vec![gen_other, gen_resp]);

        assert!(cluster.has_accepted_command(cmd_on));
        assert!(!cluster.has_accepted_command(CommandId::new(0x99)));
        assert!(!cluster.set_accepted_command(cmd_on));
        assert!(cluster.set_accepted_command(CommandId::new(0x03)));
        assert_eq!(
            cluster.accepted_commands,
            alloc::vec![cmd_on, cmd_off, cmd_toggle, CommandId::new(0x03)]
        );

        assert!(cluster.remove_accepted_command(cmd_off));
        assert!(!cluster.remove_accepted_command(cmd_off));
        assert_eq!(
            cluster.accepted_commands,
            alloc::vec![cmd_on, cmd_toggle, CommandId::new(0x03)]
        );

        assert!(cluster.has_generated_command(gen_resp));
        assert!(!cluster.has_generated_command(CommandId::new(0x99)));
        assert!(!cluster.set_generated_command(gen_resp));
        assert!(cluster.set_generated_command(CommandId::new(0x02)));
        assert!(cluster.remove_generated_command(gen_other));
        assert!(!cluster.remove_generated_command(gen_other));
        assert_eq!(cluster.generated_commands, alloc::vec![gen_resp, CommandId::new(0x02)]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn cluster_serde_omits_default_command_lists_on_deserialize() {
        use serde::de::{
            Deserialize as _, Deserializer, IntoDeserializer, Visitor,
            value::{Error as DeValueError, MapDeserializer},
        };

        // Simulate a legacy payload with only `id` and `attributes` (no `accepted_commands` or `generated_commands`).
        #[derive(Clone)]
        enum FieldVal {
            Id(u32),
            EmptySeq,
        }

        impl IntoDeserializer<'_, DeValueError> for FieldVal {
            type Deserializer = Self;

            fn into_deserializer(self) -> Self::Deserializer {
                self
            }
        }

        impl<'de> Deserializer<'de> for FieldVal {
            type Error = DeValueError;

            serde::forward_to_deserialize_any! {
                bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
                bytes byte_buf option unit unit_struct seq tuple
                tuple_struct map struct enum identifier ignored_any
            }

            fn deserialize_newtype_struct<V: Visitor<'de>>(
                self,
                _name: &'static str,
                visitor: V,
            ) -> Result<V::Value, Self::Error> {
                visitor.visit_newtype_struct(self)
            }

            fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
                match self {
                    Self::Id(v) => visitor.visit_u32(v),
                    Self::EmptySeq => {
                        let empty: Vec<u32> = alloc::vec![];
                        empty.into_deserializer().deserialize_seq(visitor)
                    }
                }
            }
        }

        let entries = [("id", FieldVal::Id(6)), ("attributes", FieldVal::EmptySeq)];
        let de = MapDeserializer::<'_, _, DeValueError>::new(entries.into_iter());
        let decoded = Cluster::deserialize(de).expect("legacy payload should deserialize");
        assert_eq!(decoded.id, ClusterId::new(6));
        assert_eq!(decoded.attributes, []);
        assert_eq!(decoded.accepted_commands, []);
        assert_eq!(decoded.generated_commands, []);
    }
}
