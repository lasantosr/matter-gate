//! Lossless wire wrappers for Matter clusters and events.
//!
//! These types pair typed domain payloads with instance-level protocol metadata
//! (`data_version`, [`GlobalAttributes`], `event_number`, `timestamp`, and
//! manufacturer-specific `extra_attributes`) without bloating lean domain structs.

use alloc::vec::Vec;
use core::{
    mem,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};

#[cfg(all(feature = "dispatch", feature = "serde"))]
use crate::dispatch::deserialize_concrete_described;
#[cfg(feature = "dispatch")]
use crate::dispatch::{EntityDescriptor, MatterDescribe, StaticEntitySchema};
use crate::{
    error::{AttributeError, ClusterError, EventError},
    model::{
        Attribute as WireAttribute, AttributeId, AttributeValue, Cluster as ModelCluster, CommandId, DataVersion,
        Event as ModelEvent, EventId, EventNumber, EventPriority, EventTimestamp, MatterStatus, Optional,
    },
};

/// Global attributes (`0xFFF8..=0xFFFD`) common to Matter clusters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub struct GlobalAttributes<F = u32> {
    /// `GeneratedCommandList` (`0xFFF8`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub generated_command_list: Optional<Vec<CommandId>>,
    /// `AcceptedCommandList` (`0xFFF9`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub accepted_command_list: Optional<Vec<CommandId>>,
    /// `EventList` (`0xFFFA`, present in Matter `v1.1`–`v1.3`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub event_list: Optional<Vec<EventId>>,
    /// `AttributeList` (`0xFFFB`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub attribute_list: Optional<Vec<AttributeId>>,
    /// `FeatureMap` (`0xFFFC`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub feature_map: Optional<F>,
    /// `ClusterRevision` (`0xFFFD`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub cluster_revision: Optional<u16>,
}

impl<F> GlobalAttributes<F> {
    /// Constructs an empty [`GlobalAttributes`] with all fields omitted.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            generated_command_list: Optional::Omitted,
            accepted_command_list: Optional::Omitted,
            event_list: Optional::Omitted,
            attribute_list: Optional::Omitted,
            feature_map: Optional::Omitted,
            cluster_revision: Optional::Omitted,
        }
    }

    /// Sets `generated_command_list` (`0xFFF8`).
    #[must_use]
    pub fn with_generated_command_list(mut self, list: Vec<CommandId>) -> Self {
        self.generated_command_list = Optional::Present(list);
        self
    }

    /// Sets `accepted_command_list` (`0xFFF9`).
    #[must_use]
    pub fn with_accepted_command_list(mut self, list: Vec<CommandId>) -> Self {
        self.accepted_command_list = Optional::Present(list);
        self
    }

    /// Sets `event_list` (`0xFFFA`).
    #[must_use]
    pub fn with_event_list(mut self, list: Vec<EventId>) -> Self {
        self.event_list = Optional::Present(list);
        self
    }

    /// Sets `attribute_list` (`0xFFFB`).
    #[must_use]
    pub fn with_attribute_list(mut self, list: Vec<AttributeId>) -> Self {
        self.attribute_list = Optional::Present(list);
        self
    }

    /// Sets `feature_map` (`0xFFFC`).
    #[must_use]
    pub fn with_feature_map(mut self, feature_map: F) -> Self {
        self.feature_map = Optional::Present(feature_map);
        self
    }

    /// Sets `cluster_revision` (`0xFFFD`).
    #[must_use]
    pub const fn with_cluster_revision(mut self, revision: u16) -> Self {
        self.cluster_revision = Optional::Present(revision);
        self
    }

    /// Returns `true` if `id` is a Matter global attribute (`0xFFF8..=0xFFFD`).
    #[must_use]
    pub const fn is_global_id(id: AttributeId) -> bool {
        matches!(
            id,
            AttributeId::GENERATED_COMMAND_LIST
                | AttributeId::ACCEPTED_COMMAND_LIST
                | AttributeId::EVENT_LIST
                | AttributeId::ATTRIBUTE_LIST
                | AttributeId::FEATURE_MAP
                | AttributeId::CLUSTER_REVISION
        )
    }

    /// Attempts to apply a global attribute (`0xFFF8..=0xFFFD`).
    ///
    /// Returns `Ok(true)` if `id` was a global attribute and was applied,
    /// `Ok(false)` if `id` is not a global attribute, or `Err(MatterStatus)`
    /// if the wire value has an invalid type or duplicate/invalid data.
    pub fn apply_global_attribute(&mut self, id: AttributeId, value: AttributeValue) -> Result<bool, MatterStatus>
    where
        F: TryFrom<u32>,
    {
        match id {
            AttributeId::GENERATED_COMMAND_LIST => {
                let AttributeValue::List(items) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.generated_command_list.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    let AttributeValue::U32(raw) = item else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    list.push(CommandId::new(raw));
                }
                self.generated_command_list = Optional::Present(list);
                Ok(true)
            }
            AttributeId::ACCEPTED_COMMAND_LIST => {
                let AttributeValue::List(items) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.accepted_command_list.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    let AttributeValue::U32(raw) = item else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    list.push(CommandId::new(raw));
                }
                self.accepted_command_list = Optional::Present(list);
                Ok(true)
            }
            AttributeId::EVENT_LIST => {
                let AttributeValue::List(items) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.event_list.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    let AttributeValue::U32(raw) = item else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    list.push(EventId::new(raw));
                }
                self.event_list = Optional::Present(list);
                Ok(true)
            }
            AttributeId::ATTRIBUTE_LIST => {
                let AttributeValue::List(items) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.attribute_list.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    let AttributeValue::U32(raw) = item else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    list.push(AttributeId::new(raw));
                }
                self.attribute_list = Optional::Present(list);
                Ok(true)
            }
            AttributeId::FEATURE_MAP => {
                let AttributeValue::U32(raw) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.feature_map.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                self.feature_map = Optional::Present(F::try_from(raw).map_err(|_| MatterStatus::ConstraintError)?);
                Ok(true)
            }
            AttributeId::CLUSTER_REVISION => {
                let AttributeValue::U16(rev) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.cluster_revision.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                self.cluster_revision = Optional::Present(rev);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Appends all present global attributes as `(AttributeId, AttributeValue)` pairs.
    pub fn into_attributes(self, out: &mut Vec<(AttributeId, AttributeValue)>)
    where
        F: Into<u32>,
    {
        if let Optional::Present(list) = self.generated_command_list {
            out.push((
                AttributeId::GENERATED_COMMAND_LIST,
                AttributeValue::List(list.into_iter().map(|id| AttributeValue::U32(id.raw())).collect()),
            ));
        }
        if let Optional::Present(list) = self.accepted_command_list {
            out.push((
                AttributeId::ACCEPTED_COMMAND_LIST,
                AttributeValue::List(list.into_iter().map(|id| AttributeValue::U32(id.raw())).collect()),
            ));
        }
        if let Optional::Present(list) = self.event_list {
            out.push((
                AttributeId::EVENT_LIST,
                AttributeValue::List(list.into_iter().map(|id| AttributeValue::U32(id.raw())).collect()),
            ));
        }
        if let Optional::Present(list) = self.attribute_list {
            out.push((
                AttributeId::ATTRIBUTE_LIST,
                AttributeValue::List(list.into_iter().map(|id| AttributeValue::U32(id.raw())).collect()),
            ));
        }
        if let Optional::Present(fm) = self.feature_map {
            out.push((AttributeId::FEATURE_MAP, AttributeValue::U32(fm.into())));
        }
        if let Optional::Present(rev) = self.cluster_revision {
            out.push((AttributeId::CLUSTER_REVISION, AttributeValue::U16(rev)));
        }
    }
}

/// Thin wire-metadata envelope around a cluster domain model `T`.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "T: Deserialize<'de>, F: Deserialize<'de> + Default"))
)]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(bound = "T: JsonSchema, F: JsonSchema + Default"))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub struct WireCluster<T, F = u32> {
    /// Instance-level `DataVersion`.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub data_version: Optional<DataVersion>,
    /// Global attributes (`0xFFF8..=0xFFFD`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub globals: GlobalAttributes<F>,
    /// Typed cluster domain state or patch.
    pub domain: T,
    /// Manufacturer-specific extension attributes (valid MEI IDs).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub extra_attributes: Vec<WireAttribute>,
}

impl<T, F> WireCluster<T, F> {
    /// Wraps `domain` in an empty [`WireCluster`].
    #[must_use]
    pub const fn new(domain: T) -> Self {
        Self {
            data_version: Optional::Omitted,
            globals: GlobalAttributes::empty(),
            domain,
            extra_attributes: Vec::new(),
        }
    }

    /// Sets `data_version`.
    #[must_use]
    pub fn with_data_version(mut self, data_version: DataVersion) -> Self {
        self.data_version = Optional::Present(data_version);
        self
    }

    /// Sets `globals`.
    #[must_use]
    pub fn with_globals(mut self, globals: GlobalAttributes<F>) -> Self {
        self.globals = globals;
        self
    }

    /// Sets `globals.feature_map`.
    #[must_use]
    pub fn with_feature_map(mut self, feature_map: F) -> Self {
        self.globals.feature_map = Optional::Present(feature_map);
        self
    }

    /// Appends a manufacturer-specific attribute to `extra_attributes`.
    #[must_use]
    pub fn with_extra_attribute(mut self, attr: WireAttribute) -> Self {
        self.extra_attributes.push(attr);
        self
    }

    /// Returns a shared reference to the inner domain value.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.domain
    }

    /// Returns a mutable reference to the inner domain value.
    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.domain
    }

    /// Consumes the wrapper and returns the inner domain value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.domain
    }

    /// Routes an attribute ID that was not handled by the domain cluster.
    ///
    /// - Global attributes (`0xFFF8..=0xFFFD`) are applied to `self.globals`.
    /// - Valid MEI attributes (`id.is_manufacturer_specific() && id.is_valid()`) are stored in `self.extra_attributes`.
    /// - Unknown standard attributes (`0x0000..=0x4FFF`), reserved vendor prefixes (`0xFFF5..=0xFFFF`), and unallocated
    ///   ranges (`0x5000..=0xEFFF`, `0xF000..=0xFFF7`, `0xFFFE`) fail fast with [`MatterStatus::UnsupportedAttribute`].
    pub fn route_unknown_attribute(&mut self, id: AttributeId, value: AttributeValue) -> Result<(), MatterStatus>
    where
        F: TryFrom<u32>,
    {
        if GlobalAttributes::<F>::is_global_id(id) {
            let _applied: bool = self.globals.apply_global_attribute(id, value)?;
            return Ok(());
        }
        if id.is_manufacturer_specific() && id.is_valid() {
            self.extra_attributes.push(WireAttribute::new(id, value));
            return Ok(());
        }
        Err(MatterStatus::UnsupportedAttribute)
    }
}

impl<T, F> Deref for WireCluster<T, F> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.domain
    }
}

impl<T, F> DerefMut for WireCluster<T, F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.domain
    }
}

impl<T, F> From<WireCluster<T, F>> for ModelCluster
where
    T: Into<ModelCluster>,
    F: Into<u32>,
{
    fn from(wire: WireCluster<T, F>) -> Self {
        let mut cluster: Self = wire.domain.into();
        if wire.data_version.is_present() {
            cluster.data_version = wire.data_version;
        }
        let mut global_attrs = Vec::new();
        wire.globals.into_attributes(&mut global_attrs);
        for (id, value) in global_attrs {
            if !cluster.attributes.iter().any(|attr| attr.id == id) {
                cluster.attributes.push(WireAttribute::new(id, value));
            }
        }
        for attr in wire.extra_attributes {
            if !cluster.attributes.iter().any(|existing| existing.id == attr.id) {
                cluster.attributes.push(attr);
            }
        }
        cluster
    }
}

impl<T, F> TryFrom<ModelCluster> for WireCluster<T, F>
where
    T: TryFrom<ModelCluster, Error = ClusterError>,
    F: TryFrom<u32>,
{
    type Error = ClusterError;

    fn try_from(mut cluster: ModelCluster) -> Result<Self, Self::Error> {
        let data_version = cluster.data_version;
        let mut globals = GlobalAttributes::<F>::empty();
        let mut extra_attributes = Vec::new();
        let mut standard_attributes = Vec::with_capacity(cluster.attributes.len());

        for attr in mem::take(&mut cluster.attributes) {
            if GlobalAttributes::<F>::is_global_id(attr.id) {
                globals.apply_global_attribute(attr.id, attr.value).map_err(|status| {
                    ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                        id: attr.id,
                        field_id: None,
                        status,
                    })
                })?;
            } else if attr.id.is_manufacturer_specific() && attr.id.is_valid() {
                extra_attributes.push(attr);
            } else {
                standard_attributes.push(attr);
            }
        }

        cluster.attributes = standard_attributes;
        let domain = T::try_from(cluster)?;
        Ok(Self {
            data_version,
            globals,
            domain,
            extra_attributes,
        })
    }
}

/// Thin wire-metadata envelope around a Matter event `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub struct WireEvent<T> {
    /// Wire event priority.
    pub priority: EventPriority,
    /// Optional wire `EventNumber`.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub event_number: Optional<EventNumber>,
    /// Optional wire `EventTimestamp`.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub timestamp: Optional<EventTimestamp>,
    /// Typed event payload.
    pub event: T,
}

impl<T> WireEvent<T> {
    /// Wraps `event` with its `priority`.
    #[must_use]
    pub const fn new(priority: EventPriority, event: T) -> Self {
        Self {
            priority,
            event_number: Optional::Omitted,
            timestamp: Optional::Omitted,
            event,
        }
    }

    /// Verifies that `wire_priority == expected_priority` before constructing [`WireEvent`].
    pub fn try_new(
        wire_priority: EventPriority,
        expected_priority: EventPriority,
        event: T,
    ) -> Result<Self, MatterStatus> {
        if wire_priority != expected_priority {
            return Err(MatterStatus::InvalidDataType);
        }
        Ok(Self::new(wire_priority, event))
    }

    /// Sets `event_number`.
    #[must_use]
    pub const fn with_event_number(mut self, event_number: EventNumber) -> Self {
        self.event_number = Optional::Present(event_number);
        self
    }

    /// Sets `timestamp`.
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp: EventTimestamp) -> Self {
        self.timestamp = Optional::Present(timestamp);
        self
    }

    /// Returns a shared reference to the inner event payload.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.event
    }

    /// Returns a mutable reference to the inner event payload.
    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.event
    }

    /// Consumes the wrapper and returns the inner event payload.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.event
    }
}

impl<T> From<WireEvent<T>> for ModelEvent
where
    T: Into<ModelEvent>,
{
    fn from(wire: WireEvent<T>) -> Self {
        let mut event: Self = wire.event.into();
        event.priority = wire.priority;
        event.event_number = wire.event_number;
        event.timestamp = wire.timestamp;
        event
    }
}

impl<T> TryFrom<ModelEvent> for WireEvent<T>
where
    T: TryFrom<ModelEvent, Error = EventError>,
{
    type Error = EventError;

    fn try_from(wire: ModelEvent) -> Result<Self, Self::Error> {
        let priority = wire.priority;
        let event_number = wire.event_number;
        let timestamp = wire.timestamp;
        let event = T::try_from(wire)?;
        Ok(Self {
            priority,
            event_number,
            timestamp,
            event,
        })
    }
}

impl<T> Deref for WireEvent<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<T> DerefMut for WireEvent<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

#[cfg(all(feature = "dispatch", feature = "serde"))]
fn deserialize_envelope_described<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned + MatterDescribe,
    D: Deserializer<'de>,
{
    use alloc::string::String;
    use core::{fmt, marker::PhantomData};

    use serde::de::{self, MapAccess, Visitor};

    struct EnvelopeVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for EnvelopeVisitor<T>
    where
        T: DeserializeOwned + MatterDescribe,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a self-describing Matter envelope object")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut payload: Option<T> = None;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "payload" if payload.is_some() => {
                        return Err(de::Error::duplicate_field("payload"));
                    }
                    "payload" => {
                        payload = Some(map.next_value::<T>()?);
                    }
                    "cluster" | "cluster_id" | "event" | "event_id" => {
                        let _ignored: de::IgnoredAny = map.next_value::<de::IgnoredAny>()?;
                    }
                    #[cfg(feature = "deny-unknown-fields")]
                    other => {
                        return Err(de::Error::unknown_field(
                            other,
                            &["cluster", "cluster_id", "event", "event_id", "payload"],
                        ));
                    }
                    #[cfg(not(feature = "deny-unknown-fields"))]
                    _ => {
                        let _ignored_val: de::IgnoredAny = map.next_value::<de::IgnoredAny>()?;
                    }
                }
            }
            payload.ok_or_else(|| de::Error::missing_field("payload"))
        }
    }

    deserializer.deserialize_map(EnvelopeVisitor(PhantomData))
}

#[cfg(all(feature = "dispatch", not(feature = "serde")))]
impl<T, F> MatterDescribe for WireCluster<T, F>
where
    T: MatterDescribe,
{
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema {
        is_empty: false,
        ..T::STATIC_SCHEMA
    };

    #[inline]
    fn descriptor(&self) -> EntityDescriptor {
        self.domain.descriptor()
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        false
    }
}

#[cfg(all(feature = "dispatch", feature = "serde"))]
impl<T, F> MatterDescribe for WireCluster<T, F>
where
    T: MatterDescribe + DeserializeOwned,
    F: Default + DeserializeOwned,
{
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema {
        is_empty: false,
        ..T::STATIC_SCHEMA
    };

    #[inline]
    fn descriptor(&self) -> EntityDescriptor {
        self.domain.descriptor()
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        false
    }

    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some((cluster, cluster_id)) = T::STATIC_SCHEMA.scope {
            deserialize_concrete_described(deserializer, EntityDescriptor::Cluster { cluster, cluster_id }, false)
        } else {
            deserialize_envelope_described(deserializer)
        }
    }
}

#[cfg(all(feature = "dispatch", not(feature = "serde")))]
impl<T> MatterDescribe for WireEvent<T>
where
    T: MatterDescribe,
{
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema {
        is_empty: false,
        ..T::STATIC_SCHEMA
    };

    #[inline]
    fn descriptor(&self) -> EntityDescriptor {
        self.event.descriptor()
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        false
    }
}

#[cfg(all(feature = "dispatch", feature = "serde"))]
impl<T> MatterDescribe for WireEvent<T>
where
    T: MatterDescribe + DeserializeOwned,
{
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema {
        is_empty: false,
        ..T::STATIC_SCHEMA
    };

    #[inline]
    fn descriptor(&self) -> EntityDescriptor {
        self.event.descriptor()
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        false
    }

    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let (Some((cluster, cluster_id)), Some((event, event_id))) = (T::STATIC_SCHEMA.scope, T::STATIC_SCHEMA.item)
        {
            deserialize_concrete_described(
                deserializer,
                EntityDescriptor::Event {
                    cluster,
                    cluster_id,
                    event,
                    event_id,
                },
                false,
            )
        } else {
            deserialize_envelope_described(deserializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_attributes_roundtrip_and_duplicate_rejection() {
        let mut globals = GlobalAttributes::<u32>::default();
        assert!(
            globals
                .apply_global_attribute(
                    AttributeId::GENERATED_COMMAND_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U32(0x01)]),
                )
                .expect("generated_command_list")
        );
        assert!(
            globals
                .apply_global_attribute(
                    AttributeId::ACCEPTED_COMMAND_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U32(0x00)]),
                )
                .expect("accepted_command_list")
        );
        assert!(
            globals
                .apply_global_attribute(
                    AttributeId::EVENT_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U32(0x02)]),
                )
                .expect("event_list")
        );
        assert!(
            globals
                .apply_global_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(0x03))
                .expect("feature_map")
        );
        assert!(
            globals
                .apply_global_attribute(AttributeId::CLUSTER_REVISION, AttributeValue::U16(4))
                .expect("cluster_revision")
        );
        assert!(
            globals
                .apply_global_attribute(
                    AttributeId::ATTRIBUTE_LIST,
                    AttributeValue::List(alloc::vec![
                        AttributeValue::U32(0x0000),
                        AttributeValue::U32(AttributeId::FEATURE_MAP.raw()),
                    ]),
                )
                .expect("attribute_list")
        );

        let mut encoded = Vec::new();
        globals.clone().into_attributes(&mut encoded);
        assert_eq!(encoded.len(), 6);

        let mut decoded = GlobalAttributes::<u32>::default();
        for (id, val) in encoded {
            assert!(decoded.apply_global_attribute(id, val).expect("decode"));
        }
        assert_eq!(decoded, globals);

        // Duplicate global attributes must fail with ConstraintError across all 6 globals
        for (id, val) in [
            (AttributeId::GENERATED_COMMAND_LIST, AttributeValue::List(Vec::new())),
            (AttributeId::ACCEPTED_COMMAND_LIST, AttributeValue::List(Vec::new())),
            (AttributeId::EVENT_LIST, AttributeValue::List(Vec::new())),
            (AttributeId::ATTRIBUTE_LIST, AttributeValue::List(Vec::new())),
            (AttributeId::FEATURE_MAP, AttributeValue::U32(0)),
            (AttributeId::CLUSTER_REVISION, AttributeValue::U16(5)),
        ] {
            assert_eq!(
                decoded.apply_global_attribute(id, val),
                Err(MatterStatus::ConstraintError)
            );
        }
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn envelopes_schemars_and_deny_unknown_fields_compatible() {
        let g_schema = schemars::schema_for!(GlobalAttributes<u32>);
        let c_schema = schemars::schema_for!(WireCluster<(), u32>);
        let e_schema = schemars::schema_for!(WireEvent<()>);

        for schema in [g_schema, c_schema, e_schema] {
            let json = serde_json::to_value(&schema).expect("serialize envelope schema");
            let obj = json.as_object().expect("schema object");
            if cfg!(feature = "deny-unknown-fields") {
                assert_eq!(
                    obj.get("additionalProperties"),
                    Some(&serde_json::Value::Bool(false)),
                    "deny-unknown-fields must emit additionalProperties: false: {json}"
                );
            } else {
                assert_ne!(obj.get("additionalProperties"), Some(&serde_json::Value::Bool(false)));
            }
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn envelopes_serde_deny_unknown_fields_respects_feature() {
        let g_json = r#"{"extra_unknown": 1}"#;
        let g_res: Result<GlobalAttributes<u32>, _> = serde_json::from_str(g_json);
        let c_json = r#"{"domain": null, "extra_unknown": 1}"#;
        let c_res: Result<WireCluster<(), u32>, _> = serde_json::from_str(c_json);
        let e_json = r#"{"priority": "Info", "event": null, "extra_unknown": 1}"#;
        let e_res: Result<WireEvent<()>, _> = serde_json::from_str(e_json);

        #[cfg(feature = "deny-unknown-fields")]
        {
            g_res.expect_err("GlobalAttributes must reject unknown fields");
            c_res.expect_err("WireCluster must reject unknown fields");
            e_res.expect_err("WireEvent must reject unknown fields");
        }
        #[cfg(not(feature = "deny-unknown-fields"))]
        {
            g_res.expect("GlobalAttributes should ignore unknown fields");
            c_res.expect("WireCluster should ignore unknown fields");
            e_res.expect("WireEvent should ignore unknown fields");
        }
    }

    #[test]
    fn cluster_envelope_routes_mei_and_rejects_unknown_standard_and_reserved() {
        let mut env = WireCluster::<(), u32>::new(());

        // Valid MEI: vendor 0xFFF1, sub-ID 0x0001
        let valid_mei = AttributeId::new(0xFFF1_0001);
        env.route_unknown_attribute(valid_mei, AttributeValue::U8(42))
            .expect("valid MEI should route to extra_attributes");
        assert_eq!(env.extra_attributes.len(), 1);
        assert_eq!(env.extra_attributes[0].id, valid_mei);

        // Unknown standard attribute 0x0000..=0x4FFF -> UnsupportedAttribute
        assert_eq!(
            env.route_unknown_attribute(AttributeId::new(0x0005), AttributeValue::Bool(true)),
            Err(MatterStatus::UnsupportedAttribute)
        );

        // Reserved vendor 0xFFFF -> UnsupportedAttribute
        assert_eq!(
            env.route_unknown_attribute(AttributeId::new(0xFFFF_0001), AttributeValue::U8(1)),
            Err(MatterStatus::UnsupportedAttribute)
        );

        // Reserved vendor 0xFFF5 -> UnsupportedAttribute
        assert_eq!(
            env.route_unknown_attribute(AttributeId::new(0xFFF5_0001), AttributeValue::U8(1)),
            Err(MatterStatus::UnsupportedAttribute)
        );

        // Unallocated standard ranges -> UnsupportedAttribute
        for raw in [0x5000_u32, 0xEFFF, 0xF000, 0xFFF7, 0xFFFE] {
            assert_eq!(
                env.route_unknown_attribute(AttributeId::new(raw), AttributeValue::U8(1)),
                Err(MatterStatus::UnsupportedAttribute)
            );
        }
    }

    #[test]
    fn event_envelope_verifies_priority_and_preserves_metadata() {
        assert_eq!(
            WireEvent::try_new(EventPriority::Debug, EventPriority::Critical, ()),
            Err(MatterStatus::InvalidDataType)
        );

        let env = WireEvent::try_new(EventPriority::Info, EventPriority::Info, ())
            .expect("matching priority")
            .with_event_number(EventNumber::new(7))
            .with_timestamp(EventTimestamp::EpochMs(1_000_000));

        assert_eq!(env.priority, EventPriority::Info);
        assert_eq!(env.event_number, Optional::Present(EventNumber::new(7)));
        assert_eq!(env.timestamp, Optional::Present(EventTimestamp::EpochMs(1_000_000)));
    }

    #[test]
    fn fluent_builders_and_inner_accessors() {
        let globals = GlobalAttributes::<u32>::empty()
            .with_generated_command_list(alloc::vec![CommandId::new(1)])
            .with_accepted_command_list(alloc::vec![CommandId::new(2)])
            .with_event_list(alloc::vec![EventId::new(3)])
            .with_attribute_list(alloc::vec![AttributeId::new(4)])
            .with_feature_map(0x05)
            .with_cluster_revision(6);

        let mut cluster = WireCluster::new(10_u8)
            .with_globals(globals.clone())
            .with_feature_map(0x0A)
            .with_extra_attribute(WireAttribute::new(AttributeId::new(0xFFF1_0001), AttributeValue::U8(9)));
        assert_eq!(*cluster.inner(), 10);
        *cluster.inner_mut() = 20;
        assert_eq!(*cluster.inner(), 20);
        assert_eq!(cluster.globals.feature_map, Optional::Present(0x0A));
        assert_eq!(cluster.globals.cluster_revision, Optional::Present(6));
        assert_eq!(cluster.extra_attributes.len(), 1);
        assert_eq!(cluster.into_inner(), 20);

        let mut ev = WireEvent::new(EventPriority::Info, 30_u8);
        assert_eq!(*ev.inner(), 30);
        *ev.inner_mut() = 40;
        assert_eq!(ev.into_inner(), 40);
    }
}
