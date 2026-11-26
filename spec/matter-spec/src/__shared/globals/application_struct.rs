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
pub struct ApplicationStruct {
    pub catalog_vendor_id: u16,
    pub application_id: String,
}

impl ApplicationStruct {
    pub const FIELD_APPLICATION_ID: FieldId = FieldId::new(0x1);
    pub const FIELD_CATALOG_VENDOR_ID: FieldId = FieldId::new(0x0);

    #[must_use]
    pub fn new(catalog_vendor_id: impl Into<u16>, application_id: impl Into<String>) -> Self {
        Self {
            catalog_vendor_id: catalog_vendor_id.into(),
            application_id: application_id.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut catalog_vendor_id = None;
        let mut application_id = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if catalog_vendor_id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u16>(&fval)?;
                        catalog_vendor_id = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if application_id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <String>::try_from(fval)?;
                        application_id = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            catalog_vendor_id: catalog_vendor_id.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            application_id: application_id.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (
                Self::FIELD_CATALOG_VENDOR_ID,
                AttributeValue::from(self.catalog_vendor_id)
            ),
            (Self::FIELD_APPLICATION_ID, AttributeValue::from(self.application_id)),
        ]
    }
}

crate::impl_matter_struct_wire!(ApplicationStruct);
