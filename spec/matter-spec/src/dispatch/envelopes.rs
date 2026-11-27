//! Self-describing [`Described<T>`] and [`Invocation<T>`] envelopes.

#[cfg(feature = "serde")]
use alloc::{format, string::String, string::ToString as _};
use core::ops::{Deref, DerefMut};
#[cfg(feature = "serde")]
use core::{fmt, marker::PhantomData};

#[cfg(feature = "serde")]
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, IgnoredAny, MapAccess, Visitor},
    ser::SerializeMap,
};

#[cfg(feature = "serde")]
use super::buffer::BufferedValue;
use super::traits::CommandMeta;
#[cfg(any(feature = "serde", feature = "schemars"))]
use super::traits::EntityKind;
#[cfg(feature = "serde")]
use super::traits::{EntityDescriptor, MatterDescribe};
use crate::model::EndpointId;

/// Entry in a generated `DESCRIBED_VARIANTS` lookup table: `(name, id, deserializer)`.
#[cfg(feature = "serde")]
pub(crate) type DescribedVariant<T> = (&'static str, u32, fn(BufferedPayload) -> Result<T, String>);

/// Entry in a generated command `DESCRIBED_VARIANTS` lookup table: `(name, id, is_client, deserializer)`.
#[cfg(feature = "serde")]
pub(crate) type DescribedCommandVariant<T> = (&'static str, u32, bool, fn(BufferedPayload) -> Result<T, String>);

/// Self-describing JSON/wire envelope for any identifiable Matter entity.
///
/// Serializes `T` alongside its `PascalCase` name(s) and numeric ID(s), omitting `"payload"`
/// when `T` has an empty/unit payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Described<T>(pub T);

impl<T> Described<T> {
    /// Wrap a Matter entity in a [`Described`] envelope.
    #[inline]
    #[must_use]
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }

    /// Returns a shared reference to the inner Matter entity.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.0
    }

    /// Returns a mutable reference to the inner Matter entity.
    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Unwrap the inner Matter entity.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Described<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Described<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Described<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

/// Self-describing command invocation envelope targeting a specific Matter [`EndpointId`].
///
/// Bounded on [`CommandMeta`] so only concrete commands or polymorphic command enums
/// can be wrapped in an [`Invocation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Invocation<T: CommandMeta> {
    /// Target Matter endpoint identifier.
    pub endpoint_id: EndpointId,
    /// Typed command payload.
    pub payload: T,
}

impl<T: CommandMeta> Invocation<T> {
    /// Construct a new command [`Invocation`] targeting `endpoint_id`.
    #[inline]
    #[must_use]
    pub fn new(endpoint_id: impl Into<EndpointId>, payload: T) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            payload,
        }
    }

    /// Returns a shared reference to the command payload.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.payload
    }

    /// Returns a mutable reference to the command payload.
    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.payload
    }

    /// Consumes the invocation and returns the command payload.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.payload
    }
}

#[cfg(feature = "serde")]
fn serialize_descriptor_fields<M: SerializeMap>(map: &mut M, descriptor: EntityDescriptor) -> Result<(), M::Error> {
    match descriptor {
        EntityDescriptor::Cluster { cluster, cluster_id } => {
            map.serialize_entry("cluster", cluster)?;
            map.serialize_entry("cluster_id", &cluster_id)?;
        }
        EntityDescriptor::Command {
            cluster,
            cluster_id,
            command,
            command_id,
        } => {
            map.serialize_entry("cluster", cluster)?;
            map.serialize_entry("cluster_id", &cluster_id)?;
            map.serialize_entry("command", command)?;
            map.serialize_entry("command_id", &command_id)?;
        }
        EntityDescriptor::Attribute {
            cluster,
            cluster_id,
            attribute,
            attribute_id,
        } => {
            map.serialize_entry("cluster", cluster)?;
            map.serialize_entry("cluster_id", &cluster_id)?;
            map.serialize_entry("attribute", attribute)?;
            map.serialize_entry("attribute_id", &attribute_id)?;
        }
        EntityDescriptor::Event {
            cluster,
            cluster_id,
            event,
            event_id,
        } => {
            map.serialize_entry("cluster", cluster)?;
            map.serialize_entry("cluster_id", &cluster_id)?;
            map.serialize_entry("event", event)?;
            map.serialize_entry("event_id", &event_id)?;
        }
        EntityDescriptor::Device {
            device_type,
            device_type_id,
        } => {
            map.serialize_entry("device_type", device_type)?;
            map.serialize_entry("device_type_id", &device_type_id)?;
        }
    }
    Ok(())
}

#[cfg(feature = "serde")]
struct PayloadRef<'a, T: ?Sized>(&'a T);

#[cfg(feature = "serde")]
impl<T: MatterDescribe + Serialize + ?Sized> Serialize for PayloadRef<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize_payload(serializer)
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for Described<T>
where
    T: MatterDescribe + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let descriptor = self.0.descriptor();
        let is_empty = self.0.is_empty_payload();
        let base_len = match descriptor {
            EntityDescriptor::Cluster { .. } | EntityDescriptor::Device { .. } => 2_usize,
            EntityDescriptor::Command { .. } | EntityDescriptor::Attribute { .. } | EntityDescriptor::Event { .. } => {
                4_usize
            }
        };
        let total_len = if is_empty { base_len } else { base_len.saturating_add(1) };

        let mut map = serializer.serialize_map(Some(total_len))?;
        serialize_descriptor_fields(&mut map, descriptor)?;
        if !is_empty {
            map.serialize_entry("payload", &PayloadRef(&self.0))?;
        }
        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Described<T>
where
    T: MatterDescribe,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_described(deserializer).map(Self)
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for Invocation<T>
where
    T: MatterDescribe + CommandMeta + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let descriptor = self.payload.descriptor();
        let is_empty = MatterDescribe::is_empty_payload(&self.payload);
        let total_len = if is_empty { 5_usize } else { 6_usize };

        let mut map = serializer.serialize_map(Some(total_len))?;
        map.serialize_entry("endpoint_id", &self.endpoint_id)?;
        serialize_descriptor_fields(&mut map, descriptor)?;
        if !is_empty {
            map.serialize_entry("payload", &PayloadRef(&self.payload))?;
        }
        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Invocation<T>
where
    T: MatterDescribe + CommandMeta,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (endpoint_id, payload) = T::deserialize_invocation(deserializer)?;
        Ok(Self { endpoint_id, payload })
    }
}

#[cfg(feature = "serde")]
struct EmptyPayload;

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for EmptyPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EmptyVisitor;

        impl<'de> Visitor<'de> for EmptyVisitor {
            type Value = EmptyPayload;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("null or an empty object {}")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(EmptyPayload)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(EmptyPayload)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                if let Some(key) = map.next_key::<String>()? {
                    return Err(de::Error::custom(format!(
                        "expected empty payload object, found unexpected field '{key}'"
                    )));
                }
                Ok(EmptyPayload)
            }
        }

        deserializer.deserialize_any(EmptyVisitor)
    }
}

#[cfg(feature = "serde")]
const fn key_names(kind: EntityKind) -> (&'static str, &'static str, Option<(&'static str, &'static str)>) {
    match kind {
        EntityKind::Cluster => ("cluster", "cluster_id", None),
        EntityKind::Command => ("cluster", "cluster_id", Some(("command", "command_id"))),
        EntityKind::Attribute => ("cluster", "cluster_id", Some(("attribute", "attribute_id"))),
        EntityKind::Event => ("cluster", "cluster_id", Some(("event", "event_id"))),
        EntityKind::Device => ("device_type", "device_type_id", None),
    }
}

#[cfg(feature = "serde")]
fn validate_str_field<E: de::Error>(field: &str, expected: &str, actual: &str) -> Result<(), E> {
    if actual == expected {
        Ok(())
    } else {
        Err(de::Error::custom(format!(
            "metadata mismatch for '{field}': expected '{expected}', got '{actual}'"
        )))
    }
}

#[cfg(feature = "serde")]
fn validate_id_field<E: de::Error>(field: &str, expected: u32, actual: u32) -> Result<(), E> {
    if actual == expected {
        Ok(())
    } else {
        Err(de::Error::custom(format!(
            "metadata mismatch for '{field}': expected {expected}, got {actual}"
        )))
    }
}

/// Deserialize a concrete [`MatterDescribe`] type from a [`Described<T>`] JSON/wire envelope.
///
/// Metadata fields are optional; any provided metadata field is validated against `expected`.
#[cfg(feature = "serde")]
pub(crate) fn deserialize_concrete_described<'de, T, D>(
    deserializer: D,
    expected: EntityDescriptor,
    is_empty: bool,
) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let (_, payload) = deserialize_concrete_inner::<T, D>(deserializer, expected, is_empty, false)?;
    Ok(payload)
}

/// Deserialize a concrete [`MatterDescribe`] command type from an [`Invocation<T>`] JSON/wire envelope.
///
/// Requires `endpoint_id`; other metadata fields are optional and validated against `expected`.
#[cfg(feature = "serde")]
pub(crate) fn deserialize_concrete_invocation<'de, T, D>(
    deserializer: D,
    expected: EntityDescriptor,
    is_empty: bool,
) -> Result<(EndpointId, T), D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let (endpoint_id, payload) = deserialize_concrete_inner::<T, D>(deserializer, expected, is_empty, true)?;
    let ep = endpoint_id.ok_or_else(|| de::Error::missing_field("endpoint_id"))?;
    Ok((ep, payload))
}

#[cfg(feature = "serde")]
enum EnvelopeField {
    EndpointId,
    ScopeName,
    ScopeId,
    ItemName,
    ItemId,
    Payload,
    Unknown,
}

#[cfg(feature = "serde")]
fn classify_field(key: &str, kind: EntityKind, expect_endpoint_id: bool) -> EnvelopeField {
    if expect_endpoint_id && key == "endpoint_id" {
        return EnvelopeField::EndpointId;
    }
    if key == "payload" {
        return EnvelopeField::Payload;
    }
    let (scope_name_key, scope_id_key, item_keys) = key_names(kind);
    if key == scope_name_key {
        return EnvelopeField::ScopeName;
    }
    if key == scope_id_key {
        return EnvelopeField::ScopeId;
    }
    if let Some((item_name_key, item_id_key)) = item_keys {
        if key == item_name_key {
            return EnvelopeField::ItemName;
        }
        if key == item_id_key {
            return EnvelopeField::ItemId;
        }
    }
    EnvelopeField::Unknown
}

#[cfg(feature = "serde")]
struct ConcreteVisitor<T> {
    expected: EntityDescriptor,
    is_empty: bool,
    expect_endpoint_id: bool,
    _marker: PhantomData<T>,
}

#[cfg(feature = "serde")]
impl<'de, T> Visitor<'de> for ConcreteVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = (Option<EndpointId>, T);

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a self-describing Matter envelope object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let kind = self.expected.kind();
        let (scope_name_key, scope_id_key, item_keys) = key_names(kind);
        let (expected_scope_name, expected_scope_id) = self.expected.scope();
        let expected_item = self.expected.item();

        let mut endpoint_id: Option<EndpointId> = None;
        let mut payload: Option<T> = None;

        while let Some(key) = map.next_key::<String>()? {
            match classify_field(&key, kind, self.expect_endpoint_id) {
                EnvelopeField::EndpointId => {
                    if endpoint_id.is_some() {
                        return Err(de::Error::duplicate_field("endpoint_id"));
                    }
                    endpoint_id = Some(map.next_value::<EndpointId>()?);
                }
                EnvelopeField::ScopeName => {
                    let actual = map.next_value::<String>()?;
                    validate_str_field(scope_name_key, expected_scope_name, &actual)?;
                }
                EnvelopeField::ScopeId => {
                    let actual = map.next_value::<u32>()?;
                    validate_id_field(scope_id_key, expected_scope_id, actual)?;
                }
                EnvelopeField::ItemName => {
                    let actual = map.next_value::<String>()?;
                    if let Some(((item_name_key, _), (expected_name, _))) = item_keys.zip(expected_item) {
                        validate_str_field(item_name_key, expected_name, &actual)?;
                    }
                }
                EnvelopeField::ItemId => {
                    let actual = map.next_value::<u32>()?;
                    if let Some(((_, item_id_key), (_, expected_id))) = item_keys.zip(expected_item) {
                        validate_id_field(item_id_key, expected_id, actual)?;
                    }
                }
                EnvelopeField::Payload => {
                    if payload.is_some() {
                        return Err(de::Error::duplicate_field("payload"));
                    }
                    payload = Some(read_concrete_payload::<T, A>(&mut map, self.is_empty)?);
                }
                EnvelopeField::Unknown => {
                    if cfg!(feature = "deny-unknown-fields") {
                        return Err(de::Error::custom(format!("unknown field '{key}'")));
                    }
                    let _: IgnoredAny = map.next_value()?;
                }
            }
        }

        let resolved_payload = match payload {
            Some(val) => val,
            None if self.is_empty => T::deserialize(BufferedValue::Unit).map_err(de::Error::custom)?,
            None => return Err(de::Error::missing_field("payload")),
        };

        Ok((endpoint_id, resolved_payload))
    }
}

#[cfg(feature = "serde")]
fn deserialize_concrete_inner<'de, T, D>(
    deserializer: D,
    expected: EntityDescriptor,
    is_empty: bool,
    expect_endpoint_id: bool,
) -> Result<(Option<EndpointId>, T), D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    deserializer.deserialize_map(ConcreteVisitor {
        expected,
        is_empty,
        expect_endpoint_id,
        _marker: PhantomData,
    })
}

#[cfg(feature = "serde")]
fn read_concrete_payload<'de, T, A>(map: &mut A, is_empty: bool) -> Result<T, A::Error>
where
    T: Deserialize<'de>,
    A: MapAccess<'de>,
{
    if is_empty {
        let EmptyPayload = map.next_value::<EmptyPayload>()?;
        T::deserialize(BufferedValue::Unit).map_err(de::Error::custom)
    } else {
        map.next_value::<T>()
    }
}

/// Opaque buffered payload passed to polymorphic enum variant selectors during
/// [`deserialize_enum_described`] and [`deserialize_enum_invocation`].
#[cfg(feature = "serde")]
#[derive(Debug, Clone)]
pub(crate) struct BufferedPayload(BufferedValue);

#[cfg(feature = "serde")]
impl BufferedPayload {
    /// Deserialize the buffered payload into a concrete variant type `T`.
    pub(crate) fn deserialize_into<'de, T: Deserialize<'de>>(self) -> Result<T, String> {
        T::deserialize(self.0).map_err(|err| err.to_string())
    }
}

/// Deserialize a polymorphic enum [`MatterDescribe`] type from a [`Described<T>`] JSON/wire envelope.
///
/// At least one discriminator (name or numeric ID) is required; if both are provided, the `select`
/// callback is responsible for validating that they agree.
#[cfg(feature = "serde")]
pub(crate) fn deserialize_enum_described<'de, T, D, F>(
    deserializer: D,
    kind: EntityKind,
    expected_scope: Option<(&'static str, u32)>,
    select: F,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    F: FnOnce(Option<&str>, Option<u32>, Option<&str>, Option<u32>, BufferedPayload) -> Result<T, String>,
{
    let (_, payload) = deserialize_enum_inner(deserializer, kind, expected_scope, false, select)?;
    Ok(payload)
}

/// Deserialize a polymorphic enum command type from an [`Invocation<T>`] JSON/wire envelope.
#[cfg(feature = "serde")]
pub(crate) fn deserialize_enum_invocation<'de, T, D, F>(
    deserializer: D,
    kind: EntityKind,
    expected_scope: Option<(&'static str, u32)>,
    select: F,
) -> Result<(EndpointId, T), D::Error>
where
    D: Deserializer<'de>,
    F: FnOnce(Option<&str>, Option<u32>, Option<&str>, Option<u32>, BufferedPayload) -> Result<T, String>,
{
    let (endpoint_id, payload) = deserialize_enum_inner(deserializer, kind, expected_scope, true, select)?;
    let ep = endpoint_id.ok_or_else(|| de::Error::missing_field("endpoint_id"))?;
    Ok((ep, payload))
}

#[cfg(feature = "serde")]
struct EnumVisitor<F> {
    kind: EntityKind,
    expected_scope: Option<(&'static str, u32)>,
    expect_endpoint_id: bool,
    select: F,
}

#[cfg(feature = "serde")]
impl<'de, T, F> Visitor<'de> for EnumVisitor<F>
where
    F: FnOnce(Option<&str>, Option<u32>, Option<&str>, Option<u32>, BufferedPayload) -> Result<T, String>,
{
    type Value = (Option<EndpointId>, T);

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a self-describing polymorphic Matter envelope object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let (scope_name_key, scope_id_key, item_keys) = key_names(self.kind);

        let mut endpoint_id: Option<EndpointId> = None;
        let mut scope_name: Option<String> = None;
        let mut scope_id: Option<u32> = None;
        let mut item_name: Option<String> = None;
        let mut item_id: Option<u32> = None;
        let mut payload: Option<BufferedValue> = None;

        while let Some(key) = map.next_key::<String>()? {
            match classify_field(&key, self.kind, self.expect_endpoint_id) {
                EnvelopeField::EndpointId => {
                    if endpoint_id.is_some() {
                        return Err(de::Error::duplicate_field("endpoint_id"));
                    }
                    endpoint_id = Some(map.next_value::<EndpointId>()?);
                }
                EnvelopeField::ScopeName => {
                    let val = map.next_value::<String>()?;
                    if let Some((expected_name, _)) = self.expected_scope {
                        validate_str_field(scope_name_key, expected_name, &val)?;
                    }
                    scope_name = Some(val);
                }
                EnvelopeField::ScopeId => {
                    let val = map.next_value::<u32>()?;
                    if let Some((_, expected_id)) = self.expected_scope {
                        validate_id_field(scope_id_key, expected_id, val)?;
                    }
                    scope_id = Some(val);
                }
                EnvelopeField::ItemName => {
                    item_name = Some(map.next_value::<String>()?);
                }
                EnvelopeField::ItemId => {
                    item_id = Some(map.next_value::<u32>()?);
                }
                EnvelopeField::Payload => {
                    if payload.is_some() {
                        return Err(de::Error::duplicate_field("payload"));
                    }
                    payload = Some(map.next_value::<BufferedValue>()?);
                }
                EnvelopeField::Unknown => {
                    if cfg!(feature = "deny-unknown-fields") {
                        return Err(de::Error::custom(format!("unknown field '{key}'")));
                    }
                    let _: IgnoredAny = map.next_value()?;
                }
            }
        }

        ensure_enum_discriminator(
            item_keys,
            scope_name_key,
            scope_id_key,
            scope_name.as_deref(),
            scope_id,
            item_name.as_deref(),
            item_id,
        )?;

        let resolved = (self.select)(
            scope_name.as_deref(),
            scope_id,
            item_name.as_deref(),
            item_id,
            BufferedPayload(payload.unwrap_or(BufferedValue::Unit)),
        )
        .map_err(de::Error::custom)?;

        Ok((endpoint_id, resolved))
    }
}

#[cfg(feature = "serde")]
fn ensure_enum_discriminator<E: de::Error>(
    item_keys: Option<(&str, &str)>,
    scope_name_key: &str,
    scope_id_key: &str,
    scope_name: Option<&str>,
    scope_id: Option<u32>,
    item_name: Option<&str>,
    item_id: Option<u32>,
) -> Result<(), E> {
    if let Some((item_name_key, item_id_key)) = item_keys {
        if item_name.is_none() && item_id.is_none() {
            return Err(de::Error::custom(format!(
                "missing discriminator: expected '{item_name_key}' or '{item_id_key}'"
            )));
        }
        return Ok(());
    }
    if scope_name.is_none() && scope_id.is_none() {
        return Err(de::Error::custom(format!(
            "missing discriminator: expected '{scope_name_key}' or '{scope_id_key}'"
        )));
    }
    Ok(())
}

#[cfg(feature = "serde")]
fn deserialize_enum_inner<'de, T, D, F>(
    deserializer: D,
    kind: EntityKind,
    expected_scope: Option<(&'static str, u32)>,
    expect_endpoint_id: bool,
    select: F,
) -> Result<(Option<EndpointId>, T), D::Error>
where
    D: Deserializer<'de>,
    F: FnOnce(Option<&str>, Option<u32>, Option<&str>, Option<u32>, BufferedPayload) -> Result<T, String>,
{
    deserializer.deserialize_map(EnumVisitor {
        kind,
        expected_scope,
        expect_endpoint_id,
        select,
    })
}

#[cfg(feature = "schemars")]
mod schema_impls {
    use alloc::{
        borrow::{Cow, ToOwned as _},
        collections::BTreeSet,
        format,
        string::String,
        vec::Vec,
    };

    use schemars::{
        _private::serde_json::{Map, Value},
        JsonSchema, Schema, SchemaGenerator, json_schema,
    };

    use super::{Described, EntityKind, Invocation};
    use crate::{
        dispatch::traits::{CommandMeta, MatterDescribe, StaticEntitySchema},
        model::EndpointId,
        registry::{ClusterDescriptor, ConformanceResult, eval_conformance},
    };

    fn insert_const_or_type_str(props: &mut Map<String, Value>, key: &str, val: Option<&'static str>) {
        let schema = match val {
            Some(s) => json_schema!({
                "type": "string",
                "const": s,
            }),
            None => json_schema!({
                "type": "string",
            }),
        };
        props.insert(key.to_owned(), schema.into());
    }

    fn insert_const_or_type_u32(props: &mut Map<String, Value>, key: &str, val: Option<u32>) {
        let schema = match val {
            Some(n) => json_schema!({
                "type": "integer",
                "format": "uint32",
                "minimum": 0,
                "const": n,
            }),
            None => json_schema!({
                "type": "integer",
                "format": "uint32",
                "minimum": 0,
            }),
        };
        props.insert(key.to_owned(), schema.into());
    }

    fn build_envelope_schema<T: JsonSchema>(
        generator: &mut SchemaGenerator,
        static_schema: StaticEntitySchema,
        include_endpoint_id: bool,
    ) -> Schema {
        let mut props = Map::new();
        let mut required: Vec<String> = Vec::new();

        if include_endpoint_id {
            let ep_schema = generator.subschema_for::<EndpointId>();
            props.insert("endpoint_id".to_owned(), ep_schema.into());
            required.push("endpoint_id".to_owned());
        }

        let (scope_name, scope_id) = match static_schema.scope {
            Some((n, id)) => (Some(n), Some(id)),
            None => (None, None),
        };
        let (item_name, item_id) = match static_schema.item {
            Some((n, id)) => (Some(n), Some(id)),
            None => (None, None),
        };

        match static_schema.kind {
            EntityKind::Cluster => {
                insert_const_or_type_str(&mut props, "cluster", scope_name);
                insert_const_or_type_u32(&mut props, "cluster_id", scope_id);
            }
            EntityKind::Command => {
                insert_const_or_type_str(&mut props, "cluster", scope_name);
                insert_const_or_type_u32(&mut props, "cluster_id", scope_id);
                insert_const_or_type_str(&mut props, "command", item_name);
                insert_const_or_type_u32(&mut props, "command_id", item_id);
            }
            EntityKind::Attribute => {
                insert_const_or_type_str(&mut props, "cluster", scope_name);
                insert_const_or_type_u32(&mut props, "cluster_id", scope_id);
                insert_const_or_type_str(&mut props, "attribute", item_name);
                insert_const_or_type_u32(&mut props, "attribute_id", item_id);
            }
            EntityKind::Event => {
                insert_const_or_type_str(&mut props, "cluster", scope_name);
                insert_const_or_type_u32(&mut props, "cluster_id", scope_id);
                insert_const_or_type_str(&mut props, "event", item_name);
                insert_const_or_type_u32(&mut props, "event_id", item_id);
            }
            EntityKind::Device => {
                insert_const_or_type_str(&mut props, "device_type", scope_name);
                insert_const_or_type_u32(&mut props, "device_type_id", scope_id);
            }
        }

        if !static_schema.is_empty {
            let payload_schema = generator.subschema_for::<T>();
            props.insert("payload".to_owned(), payload_schema.into());
            required.push("payload".to_owned());
        }

        let mut root = json_schema!({
            "type": "object",
            "properties": props,
            "required": required,
        });
        if cfg!(feature = "deny-unknown-fields")
            && let Some(obj) = root.as_object_mut()
        {
            obj.insert("additionalProperties".to_owned(), Value::Bool(false));
        }
        root
    }

    impl<T> JsonSchema for Described<T>
    where
        T: MatterDescribe + JsonSchema,
    {
        fn schema_name() -> Cow<'static, str> {
            Cow::Owned(format!("Described_{}", T::schema_name()))
        }

        fn json_schema(generator: &mut SchemaGenerator) -> Schema {
            build_envelope_schema::<T>(generator, T::STATIC_SCHEMA, false)
        }
    }

    impl<T> JsonSchema for Invocation<T>
    where
        T: MatterDescribe + CommandMeta + JsonSchema,
    {
        fn schema_name() -> Cow<'static, str> {
            Cow::Owned(format!("Invocation_{}", T::schema_name()))
        }

        fn json_schema(generator: &mut SchemaGenerator) -> Schema {
            build_envelope_schema::<T>(generator, T::STATIC_SCHEMA, true)
        }
    }

    /// Generates a JSON Schema for cluster `T` dynamically pruned against `feature_map`
    /// using the static [`ClusterDescriptor`](crate::registry::ClusterDescriptor).
    pub fn feature_pruned_cluster_schema<T: JsonSchema>(
        generator: &mut SchemaGenerator,
        descriptor: &ClusterDescriptor,
        feature_map: u32,
    ) -> Schema {
        let mut schema = T::json_schema(generator);
        let Some(obj) = schema.as_object_mut() else {
            return schema;
        };

        let mut disallowed = BTreeSet::new();
        let mut mandatory = BTreeSet::new();
        for attr in descriptor.attributes {
            let outcome = eval_conformance(attr.conformance, feature_map, None);
            match outcome {
                ConformanceResult::Disallowed => {
                    disallowed.insert(attr.field_name);
                }
                ConformanceResult::Mandatory => {
                    mandatory.insert(attr.field_name);
                }
                _ => {}
            }
        }

        if let Some(Value::Object(props)) = obj.get_mut("properties") {
            props.retain(|k, _| !disallowed.contains(k.as_str()));
            mandatory.retain(|name| props.contains_key(*name));
        }
        if let Some(Value::Array(req)) = obj.get_mut("required") {
            req.retain(|v| v.as_str().is_some_and(|s| !disallowed.contains(s)));
            for name in mandatory {
                if !req.iter().any(|v| v.as_str() == Some(name)) {
                    req.push(Value::String(String::from(name)));
                }
            }
        } else if !mandatory.is_empty() {
            let req = mandatory
                .into_iter()
                .map(|name| Value::String(String::from(name)))
                .collect();
            obj.insert(String::from("required"), Value::Array(req));
        } else {
            // No mandatory fields to add.
        }

        schema
    }
}

#[cfg(feature = "schemars")]
pub use schema_impls::feature_pruned_cluster_schema;
