#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

use crate::__shared::globals::product_identifier_type_enum::*;
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
pub struct ReplacementProductStruct {
    pub product_identifier_type: ProductIdentifierTypeEnum,
    pub product_identifier_value: String,
}

impl ReplacementProductStruct {
    pub const FIELD_PRODUCT_IDENTIFIER_TYPE: FieldId = FieldId::new(0x0);
    pub const FIELD_PRODUCT_IDENTIFIER_VALUE: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(
        product_identifier_type: impl Into<ProductIdentifierTypeEnum>,
        product_identifier_value: impl Into<String>,
    ) -> Self {
        Self {
            product_identifier_type: product_identifier_type.into(),
            product_identifier_value: product_identifier_value.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut product_identifier_type = None;
        let mut product_identifier_value = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if product_identifier_type.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <ProductIdentifierTypeEnum>::try_from(fval)?;
                        product_identifier_type = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if product_identifier_value.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <String>::try_from(fval)?;
                        (decoded.len() <= 20)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        product_identifier_value = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            product_identifier_type: product_identifier_type
                .ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            product_identifier_value: product_identifier_value
                .ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (
                Self::FIELD_PRODUCT_IDENTIFIER_TYPE,
                AttributeValue::from(self.product_identifier_type)
            ),
            (
                Self::FIELD_PRODUCT_IDENTIFIER_VALUE,
                AttributeValue::from(self.product_identifier_value)
            ),
        ]
    }
}

impl From<ReplacementProductStruct> for AttributeValue {
    #[inline]
    fn from(val: ReplacementProductStruct) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for ReplacementProductStruct {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
