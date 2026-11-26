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
pub struct AtomicAttributeStatusStruct {
    pub attribute_id: AttributeId,
    pub status_code: u8,
}

impl AtomicAttributeStatusStruct {
    pub const FIELD_ATTRIBUTE_ID: FieldId = FieldId::new(0x0);
    pub const FIELD_STATUS_CODE: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(attribute_id: impl Into<AttributeId>, status_code: impl Into<u8>) -> Self {
        Self {
            attribute_id: attribute_id.into(),
            status_code: status_code.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut attribute_id = None;
        let mut status_code = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if attribute_id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <AttributeId>::try_from(fval)?;
                        attribute_id = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if status_code.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u8>(&fval)?;
                        status_code = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            attribute_id: attribute_id.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            status_code: status_code.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (Self::FIELD_ATTRIBUTE_ID, AttributeValue::from(self.attribute_id)),
            (Self::FIELD_STATUS_CODE, AttributeValue::from(self.status_code)),
        ]
    }
}

crate::impl_matter_struct_wire!(AtomicAttributeStatusStruct);
