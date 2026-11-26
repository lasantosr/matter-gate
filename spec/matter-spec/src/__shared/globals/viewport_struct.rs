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
pub struct ViewportStruct {
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
}

impl ViewportStruct {
    pub const FIELD_X1: FieldId = FieldId::new(0x0);
    pub const FIELD_X2: FieldId = FieldId::new(0x2);
    pub const FIELD_Y1: FieldId = FieldId::new(0x1);
    pub const FIELD_Y2: FieldId = FieldId::new(0x3);

    #[must_use]
    pub fn new(x1: impl Into<u16>, y1: impl Into<u16>, x2: impl Into<u16>, y2: impl Into<u16>) -> Self {
        Self {
            x1: x1.into(),
            y1: y1.into(),
            x2: x2.into(),
            y2: y2.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut x1 = None;
        let mut y1 = None;
        let mut x2 = None;
        let mut y2 = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if x1.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u16>(&fval)?;
                        x1 = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if y1.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u16>(&fval)?;
                        y1 = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if x2.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u16>(&fval)?;
                        x2 = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                3 => {
                    if y2.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u16>(&fval)?;
                        y2 = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            x1: x1.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            y1: y1.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            x2: x2.ok_or((Some(FieldId::new(2)), MatterStatus::ConstraintError))?,
            y2: y2.ok_or((Some(FieldId::new(3)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (Self::FIELD_X1, AttributeValue::from(self.x1)),
            (Self::FIELD_Y1, AttributeValue::from(self.y1)),
            (Self::FIELD_X2, AttributeValue::from(self.x2)),
            (Self::FIELD_Y2, AttributeValue::from(self.y2)),
        ]
    }
}

crate::impl_matter_struct_wire!(ViewportStruct);
