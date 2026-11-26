//! Zero-allocation validation helpers for Descriptor (`0x001D`) attributes, device type requirements, and node
//! `PartsList` referential integrity.

use super::{
    IntegerMode, ValidationMode,
    conformance::{check_choice_group, eval_conformance, is_disallowed},
    descriptors::{ConformanceResult, DeviceTypeDescriptor, DeviceTypeIncludeItem},
    value::{decode_u8_with_mode, decode_u16_with_mode, decode_u32_with_mode},
};
use crate::{
    error::{AttributeError, ClusterError, EndpointError, NodeError},
    model::{AttributeId, AttributeValue, Cluster, ClusterId, DeviceTypeId, Endpoint, EndpointId, MatterStatus, Node},
};

/// Stack-buffered zero-alloc view of decoded `DeviceTypeList` (`0x0000`) entries, spilling to slice re-decode if >16.
pub(super) struct DecodedDeviceTypes<'a> {
    buf: [DeviceTypeId; 16],
    len: usize,
    overflow: Option<&'a [AttributeValue]>,
    int_mode: IntegerMode,
}

impl DecodedDeviceTypes<'_> {
    pub(super) fn try_for_each(
        &self,
        mut f: impl FnMut(DeviceTypeId) -> Result<(), EndpointError>,
    ) -> Result<(), EndpointError> {
        for &id in self.buf.get(..self.len).unwrap_or_default() {
            f(id)?;
        }
        if let Some(rest) = self.overflow {
            for item in rest {
                let dt_id = decode_device_type_entry_id(item, self.int_mode)
                    .map_err(|s| EndpointError::descriptor_attr_status(0x0000, s))?;
                f(dt_id)?;
            }
        }
        Ok(())
    }
}

/// Stack-buffered zero-alloc view of decoded `ClientList` (`0x0002`) entries, spilling to slice scan if >32.
pub(super) struct DecodedClientList<'a> {
    buf: [ClusterId; 32],
    len: usize,
    overflow: Option<&'a [AttributeValue]>,
    int_mode: IntegerMode,
}

impl DecodedClientList<'_> {
    pub(super) fn contains(&self, id: ClusterId) -> bool {
        if self.buf.get(..self.len).unwrap_or_default().contains(&id) {
            return true;
        }
        self.overflow.is_some_and(|rest| {
            rest.iter()
                .any(|item| decode_u32_with_mode(item, self.int_mode).map(ClusterId::new) == Ok(id))
        })
    }
}

/// Zero-copy borrowed key of a Matter `SemanticTagStruct` comparing `(mfg_code, namespace_id, tag, label)` (treating
/// omitted and `Null` labels equivalently).
#[derive(PartialEq, Eq)]
struct BorrowedSemanticTag<'a> {
    mfg_code: Option<u16>,
    namespace_id: u8,
    tag: u8,
    label: Option<&'a str>,
}

/// Returns `FeatureMap` (`0xFFFC`) on `cluster`, defaulting to `0` when absent or invalid for `int_mode`.
pub(super) fn extract_feature_map(cluster: &Cluster, int_mode: IntegerMode) -> u32 {
    cluster
        .attribute(AttributeId::FEATURE_MAP)
        .and_then(|val| decode_u32_with_mode(val, int_mode).ok())
        .unwrap_or(0)
}

/// Returns `true` for standard global attributes (`0xFFF8..=0xFFFD`).
pub(super) const fn is_global_attribute(id: AttributeId) -> bool {
    matches!(id.raw(), 0xFFF8..=0xFFFD)
}

fn decode_device_type_entry_id(value: &AttributeValue, int_mode: IntegerMode) -> Result<DeviceTypeId, MatterStatus> {
    let AttributeValue::Struct(fields) = value else {
        return Err(MatterStatus::InvalidDataType);
    };

    let mut id: Option<DeviceTypeId> = None;
    let mut revision: Option<u16> = None;

    for (idx, (field_id, val)) in fields.iter().enumerate() {
        match field_id.raw() {
            0 => {
                if id.is_some() {
                    return Err(MatterStatus::ConstraintError);
                }
                id = Some(DeviceTypeId::new(decode_u32_with_mode(val, int_mode)?));
            }
            1 => {
                if revision.is_some() {
                    return Err(MatterStatus::ConstraintError);
                }
                revision = Some(decode_u16_with_mode(val, int_mode)?);
            }
            _ => {
                if fields
                    .get(..idx)
                    .is_some_and(|prev| prev.iter().any(|(prev_id, _)| prev_id == field_id))
                {
                    return Err(MatterStatus::ConstraintError);
                }
            }
        }
    }

    let _rev = revision.ok_or(MatterStatus::ConstraintError)?;
    id.ok_or(MatterStatus::ConstraintError)
}

/// Decodes and deduplicates Descriptor `DeviceTypeList` (`0x0000`) in a single pass without heap allocation.
pub(super) fn decode_and_check_device_types(
    descriptor: &Cluster,
    int_mode: IntegerMode,
) -> Result<DecodedDeviceTypes<'_>, EndpointError> {
    let Some(val) = descriptor.attribute(AttributeId::new(0x0000)) else {
        return Err(EndpointError::EmptyDeviceTypes);
    };
    let AttributeValue::List(items) = val else {
        return Err(EndpointError::descriptor_attr_status(
            0x0000,
            MatterStatus::InvalidDataType,
        ));
    };
    if items.is_empty() {
        return Err(EndpointError::EmptyDeviceTypes);
    }

    let mut buf = [DeviceTypeId::new(0); 16];
    let mut len = 0;
    for (idx, item) in items.iter().enumerate() {
        let dt_id = decode_device_type_entry_id(item, int_mode)
            .map_err(|s| EndpointError::descriptor_attr_status(0x0000, s))?;
        if buf.get(..len).unwrap_or_default().contains(&dt_id) {
            return Err(EndpointError::DuplicateDeviceType(dt_id));
        }
        if let Some(slot) = buf.get_mut(len) {
            *slot = dt_id;
            len = len.saturating_add(1);
        } else if items
            .get(buf.len()..idx)
            .unwrap_or_default()
            .iter()
            .any(|prev| decode_device_type_entry_id(prev, int_mode).ok() == Some(dt_id))
        {
            return Err(EndpointError::DuplicateDeviceType(dt_id));
        } else {
            // Unique overflow entry verified against all prior entries.
        }
    }

    let overflow = items.get(buf.len()..).filter(|rest| !rest.is_empty());
    Ok(DecodedDeviceTypes {
        buf,
        len,
        overflow,
        int_mode,
    })
}

/// Decodes and deduplicates Descriptor `ClientList` (`0x0002`) without heap allocation.
pub(super) fn decode_and_check_client_list(
    descriptor: &Cluster,
    int_mode: IntegerMode,
) -> Result<DecodedClientList<'_>, EndpointError> {
    let mut buf = [ClusterId::new(0); 32];
    let Some(val) = descriptor.attribute(AttributeId::new(0x0002)) else {
        return Ok(DecodedClientList {
            buf,
            len: 0,
            overflow: None,
            int_mode,
        });
    };
    let AttributeValue::List(items) = val else {
        return Err(EndpointError::descriptor_attr_status(
            0x0002,
            MatterStatus::InvalidDataType,
        ));
    };

    let mut len = 0;
    for (idx, item) in items.iter().enumerate() {
        let raw = decode_u32_with_mode(item, int_mode).map_err(|s| EndpointError::descriptor_attr_status(0x0002, s))?;
        let cid = ClusterId::new(raw);
        if buf.get(..len).unwrap_or_default().contains(&cid) {
            return Err(EndpointError::DuplicateCluster(cid));
        }
        if let Some(slot) = buf.get_mut(len) {
            *slot = cid;
            len = len.saturating_add(1);
        } else if items
            .get(buf.len()..idx)
            .unwrap_or_default()
            .iter()
            .any(|prev| decode_u32_with_mode(prev, int_mode).map(ClusterId::new) == Ok(cid))
        {
            return Err(EndpointError::DuplicateCluster(cid));
        } else {
            // Unique overflow entry verified against all prior entries.
        }
    }

    let overflow = items.get(buf.len()..).filter(|rest| !rest.is_empty());
    Ok(DecodedClientList {
        buf,
        len,
        overflow,
        int_mode,
    })
}

/// Cross-checks Descriptor `ServerList` (`0x0001`), `PartsList` (`0x0003`), and `TagList` (`0x0004`).
pub(super) fn validate_descriptor_consistency(
    endpoint: &Endpoint,
    descriptor: &Cluster,
    int_mode: IntegerMode,
) -> Result<(), EndpointError> {
    if let Some(val) = descriptor.attribute(AttributeId::new(0x0001)) {
        let AttributeValue::List(srv_list) = val else {
            return Err(EndpointError::descriptor_attr_status(
                0x0001,
                MatterStatus::InvalidDataType,
            ));
        };
        for item in srv_list {
            if let Err(status) = decode_u32_with_mode(item, int_mode) {
                return Err(EndpointError::descriptor_attr_status(0x0001, status));
            }
        }
        if srv_list.len() != endpoint.clusters.len() {
            return Err(descriptor_attr_error(0x0001));
        }
        for srv in &endpoint.clusters {
            if !srv_list
                .iter()
                .any(|item| decode_u32_with_mode(item, int_mode) == Ok(srv.id.raw()))
            {
                return Err(descriptor_attr_error(0x0001));
            }
        }
    }

    validate_descriptor_parts_list(endpoint.id, descriptor, int_mode)?;
    validate_descriptor_tag_list(descriptor, int_mode)?;

    Ok(())
}

/// Verifies that every endpoint referenced by any `PartsList` (`0x0003`) exists on `node`.
pub(super) fn validate_node_parts_integrity(node: &Node, int_mode: IntegerMode) -> Result<(), NodeError> {
    for ep in &node.endpoints {
        let Some(descriptor) = ep.descriptor() else {
            continue;
        };
        let Some(AttributeValue::List(items)) = descriptor.attribute(AttributeId::new(0x0003)) else {
            continue;
        };
        for item in items {
            if let Ok(raw) = decode_u16_with_mode(item, int_mode) {
                let target_id = EndpointId::new(raw);
                if !node.endpoints.iter().any(|candidate| candidate.id == target_id) {
                    return Err(NodeError::InvalidEndpoint {
                        id: ep.id,
                        source: descriptor_attr_error(0x0003),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Enforces server/client cluster requirements and choice groups for `dt_desc` in `FullSnapshot` mode.
pub(super) fn validate_device_type_reqs(
    endpoint: &Endpoint,
    client_list: &DecodedClientList<'_>,
    dt_desc: &DeviceTypeDescriptor,
    mode: ValidationMode,
) -> Result<(), EndpointError> {
    if !matches!(mode, ValidationMode::FullSnapshot { .. }) {
        return Ok(());
    }

    for req in dt_desc.server_clusters {
        let conf = eval_conformance(req.conformance, 0, None);
        let present = endpoint.clusters.iter().any(|c| c.id == req.cluster_id);
        if conf == ConformanceResult::Mandatory && !present {
            return Err(EndpointError::MissingMandatoryServerCluster(req.cluster_id));
        }
        if is_disallowed(conf, mode) && present {
            return Err(EndpointError::DisallowedCluster(req.cluster_id));
        }
    }

    for req in dt_desc.client_clusters {
        let conf = eval_conformance(req.conformance, 0, None);
        let present = client_list.contains(req.cluster_id);
        if conf == ConformanceResult::Mandatory && !present {
            return Err(EndpointError::MissingMandatoryClientCluster(req.cluster_id));
        }
        if is_disallowed(conf, mode) && present {
            return Err(EndpointError::DisallowedCluster(req.cluster_id));
        }
    }

    for group in dt_desc.choice_groups {
        if !check_choice_group(group, |id| {
            endpoint.clusters.iter().any(|c| c.id.raw() == id) || client_list.contains(ClusterId::new(id))
        }) {
            let first_id = group.members.first().copied().unwrap_or(0);
            return Err(EndpointError::MissingMandatoryServerCluster(ClusterId::new(first_id)));
        }
    }

    Ok(())
}

/// Checks device-type-mandated attribute, command, and event inclusions on a server cluster.
pub(super) fn validate_include_item(
    endpoint: &Endpoint,
    inc: &DeviceTypeIncludeItem,
    mode: ValidationMode,
    int_mode: IntegerMode,
) -> Result<(), EndpointError> {
    if !inc.is_server {
        return Ok(());
    }
    let Some(cluster) = endpoint.clusters.iter().find(|c| c.id == inc.cluster_id) else {
        return Ok(());
    };

    if matches!(mode, ValidationMode::FullSnapshot { .. }) {
        for &req_attr in inc.required_attributes {
            if !cluster.attributes.iter().any(|a| a.id == req_attr) {
                return Err(EndpointError::InvalidCluster {
                    id: cluster.id,
                    source: ClusterError::MissingMandatoryAttribute(req_attr),
                });
            }
        }
    }

    if !inc.required_commands.is_empty()
        && let Some(accepted) = cluster.attribute(AttributeId::ACCEPTED_COMMAND_LIST)
    {
        validate_u32_id_list_contains_all(
            cluster.id,
            AttributeId::ACCEPTED_COMMAND_LIST,
            accepted,
            int_mode,
            inc.required_commands.iter().map(|id| id.raw()),
        )?;
    }

    if !inc.required_events.is_empty()
        && let Some(events) = cluster.attribute(AttributeId::EVENT_LIST)
    {
        validate_u32_id_list_contains_all(
            cluster.id,
            AttributeId::EVENT_LIST,
            events,
            int_mode,
            inc.required_events.iter().map(|id| id.raw()),
        )?;
    }

    Ok(())
}

pub(super) const fn descriptor_attr_error(attr_id: u32) -> EndpointError {
    EndpointError::descriptor_attr_status(attr_id, MatterStatus::ConstraintError)
}

fn validate_u32_id_list_contains_all(
    cluster_id: ClusterId,
    attr_id: AttributeId,
    val: &AttributeValue,
    int_mode: IntegerMode,
    mut required_raw_ids: impl Iterator<Item = u32>,
) -> Result<(), EndpointError> {
    let AttributeValue::List(items) = val else {
        return Err(EndpointError::InvalidCluster {
            id: cluster_id,
            source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                id: attr_id,
                field_id: None,
                status: MatterStatus::InvalidDataType,
            }),
        });
    };

    for item in items {
        if let Err(status) = decode_u32_with_mode(item, int_mode) {
            return Err(EndpointError::InvalidCluster {
                id: cluster_id,
                source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                    id: attr_id,
                    field_id: None,
                    status,
                }),
            });
        }
    }

    if required_raw_ids.any(|req| !items.iter().any(|item| decode_u32_with_mode(item, int_mode) == Ok(req))) {
        return Err(EndpointError::InvalidCluster {
            id: cluster_id,
            source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                id: attr_id,
                field_id: None,
                status: MatterStatus::ConstraintError,
            }),
        });
    }

    Ok(())
}

fn validate_descriptor_parts_list(
    endpoint_id: EndpointId,
    descriptor: &Cluster,
    int_mode: IntegerMode,
) -> Result<(), EndpointError> {
    let Some(val) = descriptor.attribute(AttributeId::new(0x0003)) else {
        return Ok(());
    };
    let AttributeValue::List(items) = val else {
        return Err(EndpointError::descriptor_attr_status(
            0x0003,
            MatterStatus::InvalidDataType,
        ));
    };

    let mut buf = [0_u16; 32];
    let mut len = 0;
    for (idx, item) in items.iter().enumerate() {
        let raw = decode_u16_with_mode(item, int_mode).map_err(|s| EndpointError::descriptor_attr_status(0x0003, s))?;
        let part_id = EndpointId::new(raw);
        if !part_id.is_valid() || part_id.is_root() || part_id == endpoint_id {
            return Err(descriptor_attr_error(0x0003));
        }
        if buf.get(..len).unwrap_or_default().contains(&raw) {
            return Err(descriptor_attr_error(0x0003));
        }
        if let Some(slot) = buf.get_mut(len) {
            *slot = raw;
            len = len.saturating_add(1);
        } else if items
            .get(buf.len()..idx)
            .unwrap_or_default()
            .iter()
            .any(|prev| decode_u16_with_mode(prev, int_mode) == Ok(raw))
        {
            return Err(descriptor_attr_error(0x0003));
        } else {
            // Unique overflow entry verified against all prior entries.
        }
    }

    Ok(())
}

fn decode_borrowed_semantic_tag(value: &AttributeValue, int_mode: IntegerMode) -> Option<BorrowedSemanticTag<'_>> {
    let AttributeValue::Struct(fields) = value else {
        return None;
    };
    let mut mfg_code = None;
    let mut has_mfg_code = false;
    let mut namespace_id = None;
    let mut tag = None;
    let mut label = None;

    for (fid, fval) in fields {
        match fid.raw() {
            0 => {
                has_mfg_code = true;
                match fval {
                    AttributeValue::Null => mfg_code = None,
                    other => mfg_code = Some(decode_u16_with_mode(other, int_mode).ok()?),
                }
            }
            1 => {
                namespace_id = Some(decode_u8_with_mode(fval, int_mode).ok()?);
            }
            2 => {
                tag = Some(decode_u8_with_mode(fval, int_mode).ok()?);
            }
            3 => match fval {
                AttributeValue::Null => label = None,
                AttributeValue::String(s) => label = Some(s.as_str()),
                _ => return None,
            },
            _ => return None,
        }
    }

    if !has_mfg_code {
        return None;
    }

    Some(BorrowedSemanticTag {
        mfg_code,
        namespace_id: namespace_id?,
        tag: tag?,
        label,
    })
}

fn validate_descriptor_tag_list(descriptor: &Cluster, int_mode: IntegerMode) -> Result<(), EndpointError> {
    let Some(AttributeValue::List(items)) = descriptor.attribute(AttributeId::new(0x0004)) else {
        return Ok(());
    };

    for (idx, item) in items.iter().enumerate() {
        let Some(tag) = decode_borrowed_semantic_tag(item, int_mode) else {
            continue;
        };
        if items
            .get(..idx)
            .unwrap_or_default()
            .iter()
            .filter_map(|prev| decode_borrowed_semantic_tag(prev, int_mode))
            .any(|prev| prev == tag)
        {
            return Err(descriptor_attr_error(0x0004));
        }
    }

    Ok(())
}
