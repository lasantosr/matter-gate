#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

#[allow(unused_imports, reason = "Generated global imports")]
use crate::{
    model::{
        ActionId, AttributeId, AttributeValue, ClusterId, CommandId, DataVersion, DeviceTypeId, EndpointId, EventId,
        EventNumber, FabricId, FabricIndex, FieldId, GroupId, MatterStatus, NodeId, Nullable, Optional, TransactionId,
        VendorId,
    },
    types,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub struct Target {
    pub cluster: Nullable<ClusterId>,
    pub endpoint: Nullable<EndpointId>,
    pub device_type: Nullable<DeviceTypeId>,
}

impl Target {
    pub const FIELD_CLUSTER: FieldId = FieldId::new(0x0);
    pub const FIELD_DEVICE_TYPE: FieldId = FieldId::new(0x2);
    pub const FIELD_ENDPOINT: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(
        cluster: impl Into<Nullable<ClusterId>>,
        endpoint: impl Into<Nullable<EndpointId>>,
        device_type: impl Into<Nullable<DeviceTypeId>>,
    ) -> Self {
        Self {
            cluster: cluster.into(),
            endpoint: endpoint.into(),
            device_type: device_type.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut cluster = None;
        let mut endpoint = None;
        let mut device_type = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if cluster.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <ClusterId>::try_from)?;
                        cluster = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if endpoint.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <EndpointId>::try_from)?;
                        endpoint = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if device_type.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <DeviceTypeId>::try_from)?;
                        device_type = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            cluster: cluster.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            endpoint: endpoint.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            device_type: device_type.ok_or((Some(FieldId::new(2)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (
                Self::FIELD_CLUSTER,
                match self.cluster {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                }
            ),
            (
                Self::FIELD_ENDPOINT,
                match self.endpoint {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                }
            ),
            (
                Self::FIELD_DEVICE_TYPE,
                match self.device_type {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                }
            ),
        ]
    }
}

impl From<Target> for AttributeValue {
    #[inline]
    fn from(val: Target) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for Target {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
