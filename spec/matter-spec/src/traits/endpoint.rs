use alloc::{vec, vec::Vec};

use crate::{
    error::EndpointError,
    model::{
        Attribute, AttributeId, AttributeValue, Cluster, ClusterId, DeviceTypeEntry, DeviceTypeId, Endpoint, Optional,
    },
};

/// Validates that `endpoint` declares `device_type_id` and satisfies its disallowed and mandatory cluster lists.
pub(crate) fn validate_device_view(
    endpoint: &Endpoint,
    device_type_id: DeviceTypeId,
    disallowed: &[ClusterId],
    mandatory_server: &[ClusterId],
    mandatory_client: &[ClusterId],
) -> Result<(), EndpointError> {
    let Optional::Present(dt_iter) = endpoint
        .device_type_list()
        .map_err(|status| EndpointError::descriptor_attr_status(0x0000, status))?
    else {
        return Err(EndpointError::EmptyDeviceTypes);
    };
    let mut first_id: Option<DeviceTypeId> = None;
    let mut found = false;
    for entry in dt_iter {
        let entry = entry.map_err(|status| EndpointError::descriptor_attr_status(0x0000, status))?;
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
            .map_err(|status| EndpointError::descriptor_attr_status(0x0002, status))?;
        for &cluster_id in mandatory_client {
            if !client_list
                .contains(cluster_id)
                .map_err(|status| EndpointError::descriptor_attr_status(0x0002, status))?
            {
                return Err(EndpointError::MissingMandatoryClientCluster(cluster_id));
            }
        }
    }
    Ok(())
}

/// Synthesizes a `Descriptor` [`Cluster`] (`0x001D`) while preserving `PartsList` (`0x0003`),
/// `TagList` (`0x0004`), `EndpointUniqueId` (`0x0005`), `FeatureMap` (`0xFFFC`), `ClusterRevision` (`0xFFFD`),
/// global lists, and `data_version` from an existing `Descriptor` cluster if present.
#[must_use]
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
