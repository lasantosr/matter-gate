//! Semantic marker and contract traits for Matter attributes.

use alloc::{string::String, vec, vec::Vec};
use core::fmt::Debug;

use crate::{
    error::{AttributeError, ClusterError, CommandError, EndpointError, EventError},
    model::{
        Attribute, AttributeId, AttributeValue, Cluster, ClusterId, Command, CommandDirection, CommandId,
        DeviceTypeEntry, DeviceTypeId, Endpoint, EndpointId, Event, EventId, EventPriority, MatterStatus, Nullable,
        Optional,
    },
    wire::{WireCluster, WireEvent},
};

/// Structural contract implemented by generated cluster state structs (`<Name>Cluster`).
pub trait ClusterSpec: Sized + TryFrom<Cluster, Error = ClusterError> + Into<Cluster> {
    /// Typed single-attribute enum (`Attribute`).
    type Attribute;
    /// Typed `ClientToServer` command enum (`Command`, or `()` if none).
    type Command;
    /// Typed `ServerToClient` command response enum (`CommandResponse`, or `()` if none).
    type CommandResponse;
    /// Typed event enum (`Event`, or `()` if none).
    type Event;
    /// Typed `FeatureMap` representation (`Features` or `u32`).
    type Features: Copy + Default + Into<u32> + TryFrom<u32>;
    /// Sparse/delta patch struct (`<Name>ClusterPatch`).
    type Patch: PartialCluster<Full = Self>;

    /// CSA cluster identifier.
    const CLUSTER_ID: ClusterId;
    /// CSA cluster revision for this specification release.
    const CLUSTER_REVISION: u16;
    /// Human-readable cluster name.
    const NAME: &'static str;
    /// Whether this cluster has no attributes (e.g. empty cluster struct).
    const IS_EMPTY: bool = false;

    /// Applies a sparse patch in place onto `self`.
    fn apply_patch(&mut self, patch: Self::Patch);

    /// Wraps `self` in a lossless [`WireCluster`] with typed [`Self::Features`](ClusterSpec::Features).
    #[inline]
    fn into_wire(self) -> WireCluster<Self, Self::Features> {
        WireCluster::new(self)
    }
}

/// Contract implemented by typed `ClientToServer` command request structs and `ServerToClient` response structs.
pub trait CommandSpec: Sized + TryFrom<Command, Error = CommandError> + Into<Command> {
    /// CSA cluster identifier.
    const CLUSTER_ID: ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    const CLUSTER_NAME: &'static str;
    /// CSA command identifier.
    const COMMAND_ID: CommandId;
    /// Canonical `PascalCase` Matter Command name.
    const COMMAND_NAME: &'static str;
    /// Interaction Model command direction.
    const DIRECTION: CommandDirection = CommandDirection::ClientToServer;
    /// Whether this command carries an empty/unit payload.
    const IS_EMPTY: bool = false;
    /// Expected response type (`()` for commands that return `DefaultResponse` and for `ServerToClient` responses).
    type Response;
}

/// Static identity and wire-conversion contract for a concrete Matter attribute.
pub trait AttributeSpec: Sized + TryFrom<Attribute, Error = AttributeError> + Into<Attribute> {
    /// Owning Matter Cluster ID.
    const CLUSTER_ID: ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    const CLUSTER_NAME: &'static str;
    /// Numeric Matter Attribute ID.
    const ATTRIBUTE_ID: AttributeId;
    /// Canonical `PascalCase` Matter Attribute name.
    const ATTRIBUTE_NAME: &'static str;
    /// Whether the attribute value is nullable per the Matter specification.
    const IS_NULLABLE: bool = false;
    /// Whether the attribute is optional on its cluster.
    const IS_OPTIONAL: bool = false;
    /// Whether the attribute is writable.
    const IS_WRITABLE: bool = false;
}

/// Static identity and wire-conversion contract for a concrete Matter event.
pub trait EventSpec: Sized + TryFrom<Event, Error = EventError> + Into<Event> {
    /// Owning Matter Cluster ID.
    const CLUSTER_ID: ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    const CLUSTER_NAME: &'static str;
    /// Numeric Matter Event ID.
    const EVENT_ID: EventId;
    /// Canonical `PascalCase` Matter Event name.
    const EVENT_NAME: &'static str;
    /// Interaction Model event priority.
    const PRIORITY: EventPriority = EventPriority::Info;
    /// Whether this event carries an empty/unit payload.
    const IS_EMPTY: bool = false;

    /// Wraps `self` in a lossless [`WireEvent`] with [`Self::PRIORITY`](EventSpec::PRIORITY).
    #[inline]
    fn into_wire(self) -> WireEvent<Self> {
        WireEvent::new(Self::PRIORITY, self)
    }
}

/// Contract implemented by sparse/delta cluster patch types (`<Name>ClusterPatch`).
pub trait PartialCluster: Sized + Default {
    /// Full cluster state type corresponding to this patch.
    type Full;

    /// Applies a single wire attribute `(AttributeId, AttributeValue)` to this patch.
    fn apply_attribute(&mut self, id: AttributeId, value: AttributeValue) -> Result<(), AttributeError>;

    /// Merges another patch into `self`, overwriting any fields present in `other`.
    fn merge(&mut self, other: Self);

    /// Attempts to materialize a full cluster state from this patch.
    fn try_into_full(self) -> Result<Self::Full, ClusterError>;

    /// Converts all present fields in this patch into `(AttributeId, AttributeValue)` pairs.
    fn into_attributes(self) -> Vec<(AttributeId, AttributeValue)>;
}

/// Structural contract implemented by generated device-type models (`v1_X::endpoints::<Name>`).
pub trait DeviceTypeSpec: Sized + TryFrom<Endpoint, Error = EndpointError> {
    /// Zero-allocation borrowed view over an [`Endpoint`] for this device type.
    type View<'a>;

    /// CSA device type identifier.
    const DEVICE_TYPE_ID: DeviceTypeId;
    /// CSA device type revision for this specification release.
    const REVISION: u16;
    /// Human-readable device type name.
    const NAME: &'static str;
    /// Whether this device type has an empty/unit payload.
    const IS_EMPTY: bool = false;

    /// Places this typed device on an [`Endpoint`] with the given `id`.
    #[must_use]
    fn into_endpoint(self, id: EndpointId) -> Endpoint;

    /// Constructs a zero-allocation borrowed view over `endpoint`.
    fn view(endpoint: &Endpoint) -> Result<Self::View<'_>, EndpointError>;
}

/// Extension trait for inspecting [`Endpoint`]s as typed Matter device types.
pub trait EndpointExt {
    /// Returns a zero-allocation borrowed view over `self` if `self` declares
    /// [`D::DEVICE_TYPE_ID`](DeviceTypeSpec::DEVICE_TYPE_ID).
    ///
    /// Returns `Ok(None)` when `D` is absent from Descriptor `DeviceTypeList` (`0x0000`), `Ok(Some(view))` when
    /// present and conformant, or `Err(e)` when `D` is listed on the endpoint but fails conformance checks.
    fn view_as<D: DeviceTypeSpec>(&self) -> Result<Option<D::View<'_>>, EndpointError>;
}

impl EndpointExt for Endpoint {
    #[inline]
    fn view_as<D: DeviceTypeSpec>(&self) -> Result<Option<D::View<'_>>, EndpointError> {
        if !self
            .has_device_type(D::DEVICE_TYPE_ID)
            .map_err(|status| descriptor_attr_invalid(0x0000, status))?
        {
            return Ok(None);
        }
        D::view(self).map(Some)
    }
}

fn descriptor_attr_invalid(attr_id: u32, status: MatterStatus) -> EndpointError {
    EndpointError::InvalidCluster {
        id: ClusterId::DESCRIPTOR,
        source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
            id: AttributeId::new(attr_id),
            field_id: None,
            status,
        }),
    }
}

/// Validates that `endpoint` declares `device_type_id` and satisfies its disallowed and mandatory cluster lists.
#[allow(
    clippy::allow_attributes,
    dead_code,
    reason = "Called conditionally by feature-gated generated endpoints"
)]
pub(crate) fn validate_device_view(
    endpoint: &Endpoint,
    device_type_id: DeviceTypeId,
    disallowed: &[ClusterId],
    mandatory_server: &[ClusterId],
    mandatory_client: &[ClusterId],
) -> Result<(), EndpointError> {
    let Optional::Present(dt_iter) = endpoint
        .device_type_list()
        .map_err(|status| descriptor_attr_invalid(0x0000, status))?
    else {
        return Err(EndpointError::EmptyDeviceTypes);
    };
    let mut first_id: Option<DeviceTypeId> = None;
    let mut found = false;
    for entry in dt_iter {
        let entry = entry.map_err(|status| descriptor_attr_invalid(0x0000, status))?;
        if first_id.is_none() {
            first_id = Some(entry.id);
        }
        if entry.id == device_type_id {
            found = true;
            break;
        }
    }
    let Some(actual) = first_id else {
        return Err(EndpointError::EmptyDeviceTypes);
    };
    if !found {
        return Err(EndpointError::DeviceTypeMismatch {
            expected: device_type_id,
            actual,
        });
    }
    for &cluster_id in disallowed {
        if endpoint.cluster(cluster_id).is_some() {
            return Err(EndpointError::DisallowedCluster(cluster_id));
        }
    }
    for &cluster_id in mandatory_server {
        if endpoint.cluster(cluster_id).is_none() {
            return Err(EndpointError::MissingMandatoryServerCluster(cluster_id));
        }
    }
    if !mandatory_client.is_empty() {
        let client_list = endpoint
            .client_list()
            .map_err(|status| descriptor_attr_invalid(0x0002, status))?;
        for &cluster_id in mandatory_client {
            if !client_list
                .contains(cluster_id)
                .map_err(|status| descriptor_attr_invalid(0x0002, status))?
            {
                return Err(EndpointError::MissingMandatoryClientCluster(cluster_id));
            }
        }
    }
    Ok(())
}

/// Marker and contract trait for clusters derived from the Matter Mode Base cluster schema.
pub trait ModeBaseCluster: Sized {
    /// Associated strongly-typed enum for mode tags.
    type ModeTag: Into<Self::ModeTagStruct> + Copy + PartialEq + Debug;
    /// Associated wire struct representation of a mode tag.
    type ModeTagStruct;
    /// Associated wire struct representation of a mode option.
    type ModeOptionStruct: ModeOptionBuilder<Self::ModeTagStruct>;

    /// Construct a new mode base cluster instance with supported modes and current mode.
    fn new(supported_modes: Vec<Self::ModeOptionStruct>, current_mode: u8) -> Self;
}

/// Builder contract implemented by generated `ModeOptionStruct` types.
pub trait ModeOptionBuilder<TTag> {
    /// Construct a new mode option struct with label, mode number, and associated tags.
    fn build(label: String, mode: u8, mode_tags: Vec<TTag>) -> Self;
}

/// Helper to decode an array field from an owned [`AttributeValue::List`] without cloning.
#[inline]
#[allow(
    clippy::allow_attributes,
    dead_code,
    reason = "Called conditionally by feature-gated generated clusters"
)]
pub(crate) fn decode_list<T, E>(
    val: AttributeValue,
    mut f: impl FnMut(AttributeValue) -> Result<T, E>,
) -> Result<Vec<T>, MatterStatus>
where
    MatterStatus: From<E>,
{
    match val {
        AttributeValue::List(items) => items
            .into_iter()
            .map(&mut f)
            .collect::<Result<Vec<T>, _>>()
            .map_err(Into::into),
        _ => Err(MatterStatus::InvalidDataType),
    }
}

/// Helper to decode a nullable field from an owned [`AttributeValue`] without cloning.
#[inline]
#[allow(
    clippy::allow_attributes,
    dead_code,
    reason = "Called conditionally by feature-gated generated clusters"
)]
pub(crate) fn decode_nullable<T, E>(
    val: AttributeValue,
    f: impl FnOnce(AttributeValue) -> Result<T, E>,
) -> Result<Nullable<T>, MatterStatus>
where
    MatterStatus: From<E>,
{
    match val {
        AttributeValue::Null => Ok(Nullable::Null),
        other => f(other).map(Nullable::NonNull).map_err(Into::into),
    }
}

/// Helper to decode a nullable array field from an owned [`AttributeValue`] without cloning.
#[inline]
#[allow(
    clippy::allow_attributes,
    dead_code,
    reason = "Called conditionally by feature-gated generated clusters"
)]
pub(crate) fn decode_nullable_list<T, E>(
    val: AttributeValue,
    mut f: impl FnMut(AttributeValue) -> Result<T, E>,
) -> Result<Nullable<Vec<T>>, MatterStatus>
where
    MatterStatus: From<E>,
{
    match val {
        AttributeValue::Null => Ok(Nullable::Null),
        AttributeValue::List(items) => {
            let res = items.into_iter().map(&mut f).collect::<Result<Vec<T>, _>>()?;
            Ok(Nullable::NonNull(res))
        }
        _ => Err(MatterStatus::InvalidDataType),
    }
}

/// Synthesizes a `Descriptor` [`Cluster`] (`0x001D`) while preserving `PartsList` (`0x0003`),
/// `TagList` (`0x0004`), `EndpointUniqueId` (`0x0005`), `FeatureMap` (`0xFFFC`), `ClusterRevision` (`0xFFFD`),
/// global lists, and `data_version` from an existing `Descriptor` cluster if present.
#[must_use]
#[allow(
    clippy::allow_attributes,
    dead_code,
    reason = "Called conditionally by feature-gated generated endpoints"
)]
pub(crate) fn update_descriptor_cluster(
    existing: Option<&Cluster>,
    device_types: &[DeviceTypeEntry],
    server_clusters: &[Cluster],
    client_clusters: &[ClusterId],
) -> Cluster {
    let device_type_list = AttributeValue::List(device_types.iter().copied().map(AttributeValue::from).collect());

    let mut server_ids = Vec::with_capacity(server_clusters.len().saturating_add(1));
    server_ids.push(AttributeValue::from(ClusterId::DESCRIPTOR));
    for cluster in server_clusters {
        if cluster.id != ClusterId::DESCRIPTOR {
            server_ids.push(AttributeValue::from(cluster.id));
        }
    }
    let server_list = AttributeValue::List(server_ids);
    let client_list = AttributeValue::List(client_clusters.iter().copied().map(AttributeValue::from).collect());
    let parts_list = AttributeValue::List(Vec::new());

    let mut descriptor = Cluster::new(
        ClusterId::DESCRIPTOR,
        vec![
            Attribute::new(AttributeId::new(0x0000), device_type_list),
            Attribute::new(AttributeId::new(0x0001), server_list),
            Attribute::new(AttributeId::new(0x0002), client_list),
            Attribute::new(AttributeId::new(0x0003), parts_list),
        ],
    );

    if let Some(prev) = existing {
        descriptor.data_version = prev.data_version;
        for attr_id in [
            AttributeId::new(0x0003), // PartsList
            AttributeId::new(0x0004), // TagList
            AttributeId::new(0x0005), // EndpointUniqueId
            AttributeId::GENERATED_COMMAND_LIST,
            AttributeId::ACCEPTED_COMMAND_LIST,
            AttributeId::EVENT_LIST,
            AttributeId::ATTRIBUTE_LIST,
            AttributeId::FEATURE_MAP,
            AttributeId::CLUSTER_REVISION,
        ] {
            if let Some(val) = prev.attribute(attr_id) {
                descriptor.set_attribute(attr_id, val.clone());
            }
        }
    }
    descriptor
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::model::{DataVersion, Optional};

    #[test]
    fn decode_list_success() {
        let val = AttributeValue::List(vec![AttributeValue::U8(1), AttributeValue::U8(2)]);
        let decoded = decode_list(val, |item| match item {
            AttributeValue::U8(v) => Ok(v),
            _ => Err(MatterStatus::InvalidDataType),
        })
        .unwrap();
        assert_eq!(decoded, vec![1, 2]);
    }

    #[test]
    fn decode_list_item_error() {
        let val = AttributeValue::List(vec![AttributeValue::U8(1), AttributeValue::U16(2)]);
        let res = decode_list(val, |item| match item {
            AttributeValue::U8(v) => Ok(v),
            _ => Err(MatterStatus::InvalidDataType),
        });
        assert_eq!(res, Err(MatterStatus::InvalidDataType));
    }

    #[test]
    fn decode_list_not_a_list() {
        let val = AttributeValue::Bool(true);
        let res = decode_list(val, Ok::<_, MatterStatus>);
        assert_eq!(res, Err(MatterStatus::InvalidDataType));
    }

    #[test]
    fn decode_nullable_works() {
        let res_null = decode_nullable(AttributeValue::Null, |_| Ok::<_, MatterStatus>(42_u32)).unwrap();
        assert_eq!(res_null, Nullable::Null);

        let res_some = decode_nullable(AttributeValue::U32(100), |item| match item {
            AttributeValue::U32(v) => Ok(v),
            _ => Err(MatterStatus::InvalidDataType),
        })
        .unwrap();
        assert_eq!(res_some, Nullable::NonNull(100));
    }

    #[test]
    fn decode_nullable_list_works() {
        let res_null: Nullable<Vec<u8>> =
            decode_nullable_list(AttributeValue::Null, |_| Ok::<_, MatterStatus>(0_u8)).unwrap();
        assert_eq!(res_null, Nullable::Null);

        let list_val = AttributeValue::List(vec![AttributeValue::U8(10), AttributeValue::U8(20)]);
        let res_list = decode_nullable_list(list_val, |item| match item {
            AttributeValue::U8(v) => Ok(v),
            _ => Err(MatterStatus::InvalidDataType),
        })
        .unwrap();
        assert_eq!(res_list, Nullable::NonNull(vec![10, 20]));

        let err = decode_nullable_list(AttributeValue::U32(42), |_| Ok::<_, MatterStatus>(0_u8));
        assert_eq!(err, Err(MatterStatus::InvalidDataType));
    }

    #[test]
    fn validate_device_view_checks_presence_and_conformance() {
        let dt_id = DeviceTypeId::new(0x0100);
        let disallowed = [ClusterId::new(0x0406)];
        let mandatory_srv = [ClusterId::new(0x0006)];
        let mandatory_cli = [ClusterId::new(0x0019)];

        let empty = Endpoint::empty(EndpointId::new(1));
        assert_eq!(
            validate_device_view(&empty, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
            Err(EndpointError::EmptyDeviceTypes)
        );

        let wrong_dt = Endpoint::single_device(EndpointId::new(1), DeviceTypeId::new(0x0101), 1, vec![]);
        assert_eq!(
            validate_device_view(&wrong_dt, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
            Err(EndpointError::DeviceTypeMismatch {
                expected: dt_id,
                actual: DeviceTypeId::new(0x0101),
            })
        );

        let with_disallowed = Endpoint::single_device(
            EndpointId::new(1),
            dt_id,
            1,
            vec![
                Cluster::empty(ClusterId::new(0x0406)),
                Cluster::empty(ClusterId::new(0x0006)),
            ],
        )
        .with_client_list([ClusterId::new(0x0019)]);
        assert_eq!(
            validate_device_view(&with_disallowed, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
            Err(EndpointError::DisallowedCluster(ClusterId::new(0x0406)))
        );

        let missing_srv =
            Endpoint::single_device(EndpointId::new(1), dt_id, 1, vec![]).with_client_list([ClusterId::new(0x0019)]);
        assert_eq!(
            validate_device_view(&missing_srv, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
            Err(EndpointError::MissingMandatoryServerCluster(ClusterId::new(0x0006)))
        );

        let missing_cli = Endpoint::single_device(
            EndpointId::new(1),
            dt_id,
            1,
            vec![Cluster::empty(ClusterId::new(0x0006))],
        );
        assert_eq!(
            validate_device_view(&missing_cli, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
            Err(EndpointError::MissingMandatoryClientCluster(ClusterId::new(0x0019)))
        );

        let valid = Endpoint::single_device(
            EndpointId::new(1),
            dt_id,
            1,
            vec![Cluster::empty(ClusterId::new(0x0006))],
        )
        .with_client_list([ClusterId::new(0x0019)]);
        validate_device_view(&valid, dt_id, &disallowed, &mandatory_srv, &mandatory_cli).unwrap();
    }

    #[test]
    fn update_descriptor_cluster_preserves_metadata() {
        let dt = DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 2);
        let server = Cluster::empty(ClusterId::new(0x0006));
        let client = ClusterId::new(0x0019);

        let mut prev = Cluster::empty(ClusterId::DESCRIPTOR).with_data_version(DataVersion::new(9));
        prev.set_attribute(
            AttributeId::new(0x0003),
            AttributeValue::List(vec![AttributeValue::U16(2)]),
        );
        prev.set_attribute(AttributeId::new(0x0005), AttributeValue::String("ep-1".into()));
        prev.set_attribute(AttributeId::CLUSTER_REVISION, AttributeValue::U16(3));

        let desc = update_descriptor_cluster(Some(&prev), &[dt], &[server], &[client]);
        assert_eq!(desc.id, ClusterId::DESCRIPTOR);
        assert_eq!(desc.data_version, Optional::Present(DataVersion::new(9)));
        assert_eq!(
            desc.attribute(AttributeId::new(0x0003)),
            Some(&AttributeValue::List(vec![AttributeValue::U16(2)]))
        );
        assert_eq!(
            desc.attribute(AttributeId::new(0x0005)),
            Some(&AttributeValue::String("ep-1".into()))
        );
        assert_eq!(
            desc.attribute(AttributeId::CLUSTER_REVISION),
            Some(&AttributeValue::U16(3))
        );
    }

    #[test]
    fn update_descriptor_cluster_deduplicates_descriptor_and_handles_composed_device_types() {
        let dts = [
            DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 3),
            DeviceTypeEntry::new(DeviceTypeId::new(0x0013), 2),
        ];
        let existing = Cluster::empty(ClusterId::DESCRIPTOR);
        let server_clusters = [
            existing.clone(),
            Cluster::empty(ClusterId::new(0x0006)),
            Cluster::empty(ClusterId::new(0x0039)),
        ];
        let client_clusters = [ClusterId::new(0x0019)];

        let mut prev = Cluster::empty(ClusterId::DESCRIPTOR).with_data_version(DataVersion::new(12));
        prev.set_attribute(AttributeId::new(0x0004), AttributeValue::List(vec![]));
        prev.set_attribute(AttributeId::ACCEPTED_COMMAND_LIST, AttributeValue::List(vec![]));
        prev.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(0x01));

        let desc = update_descriptor_cluster(Some(&prev), &dts, &server_clusters, &client_clusters);
        assert_eq!(desc.data_version, Optional::Present(DataVersion::new(12)));
        assert_eq!(
            desc.attribute(AttributeId::new(0x0004)),
            Some(&AttributeValue::List(vec![]))
        );
        assert_eq!(
            desc.attribute(AttributeId::ACCEPTED_COMMAND_LIST),
            Some(&AttributeValue::List(vec![]))
        );
        assert_eq!(
            desc.attribute(AttributeId::FEATURE_MAP),
            Some(&AttributeValue::U32(0x01))
        );

        let Some(AttributeValue::List(dt_list)) = desc.attribute(AttributeId::new(0x0000)) else {
            panic!("expected DeviceTypeList");
        };
        assert_eq!(dt_list.len(), 2);

        let Some(AttributeValue::List(srv_list)) = desc.attribute(AttributeId::new(0x0001)) else {
            panic!("expected ServerList");
        };
        // Descriptor (0x001D) appears only once even though it was present in `server_clusters`
        assert_eq!(
            srv_list,
            &vec![
                AttributeValue::from(ClusterId::DESCRIPTOR),
                AttributeValue::from(ClusterId::new(0x0006)),
                AttributeValue::from(ClusterId::new(0x0039)),
            ]
        );

        let Some(AttributeValue::List(cli_list)) = desc.attribute(AttributeId::new(0x0002)) else {
            panic!("expected ClientList");
        };
        assert_eq!(cli_list, &vec![AttributeValue::from(ClusterId::new(0x0019))]);
    }
}
