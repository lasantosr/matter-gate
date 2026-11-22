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
pub struct DeviceTypeStruct {
    pub device_type: DeviceTypeId,
    pub revision: u16,
}

impl DeviceTypeStruct {
    pub const FIELD_DEVICE_TYPE: FieldId = FieldId::new(0x0);
    pub const FIELD_REVISION: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(device_type: impl Into<DeviceTypeId>, revision: impl Into<u16>) -> Self {
        Self {
            device_type: device_type.into(),
            revision: revision.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut device_type = None;
        let mut revision = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if device_type.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <DeviceTypeId>::try_from(fval)?;
                        device_type = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if revision.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <u16>::try_from(fval)?;
                        (decoded >= 1_u16).then_some(()).ok_or(MatterStatus::ConstraintError)?;
                        revision = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            device_type: device_type.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            revision: revision.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (Self::FIELD_DEVICE_TYPE, AttributeValue::from(self.device_type)),
            (Self::FIELD_REVISION, AttributeValue::from(self.revision)),
        ]
    }
}

impl From<DeviceTypeStruct> for AttributeValue {
    #[inline]
    fn from(val: DeviceTypeStruct) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for DeviceTypeStruct {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
