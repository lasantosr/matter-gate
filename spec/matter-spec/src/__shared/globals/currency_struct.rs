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
pub struct CurrencyStruct {
    pub currency: u16,
    pub decimal_points: u8,
}

impl CurrencyStruct {
    pub const FIELD_CURRENCY: FieldId = FieldId::new(0x0);
    pub const FIELD_DECIMAL_POINTS: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(currency: impl Into<u16>, decimal_points: impl Into<u8>) -> Self {
        Self {
            currency: currency.into(),
            decimal_points: decimal_points.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut currency = None;
        let mut decimal_points = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if currency.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u16>(&fval)?;
                        (decoded <= 999_u16)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        currency = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if decimal_points.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u8>(&fval)?;
                        decimal_points = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            currency: currency.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            decimal_points: decimal_points.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (Self::FIELD_CURRENCY, AttributeValue::from(self.currency)),
            (Self::FIELD_DECIMAL_POINTS, AttributeValue::from(self.decimal_points)),
        ]
    }
}

crate::impl_matter_struct_wire!(CurrencyStruct);
