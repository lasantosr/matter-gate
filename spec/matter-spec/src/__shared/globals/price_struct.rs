#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

use crate::__shared::globals::currency_struct::*;
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
pub struct PriceStruct {
    pub amount: i64,
    pub currency: CurrencyStruct,
}

impl PriceStruct {
    pub const FIELD_AMOUNT: FieldId = FieldId::new(0x0);
    pub const FIELD_CURRENCY: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(amount: impl Into<i64>, currency: impl Into<CurrencyStruct>) -> Self {
        Self {
            amount: amount.into(),
            currency: currency.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut amount = None;
        let mut currency = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if amount.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <i64>::try_from(fval)?;
                        amount = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if currency.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <CurrencyStruct>::try_from(fval)?;
                        currency = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            amount: amount.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            currency: currency.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (Self::FIELD_AMOUNT, AttributeValue::from(self.amount)),
            (Self::FIELD_CURRENCY, AttributeValue::from(self.currency)),
        ]
    }
}

impl From<PriceStruct> for AttributeValue {
    #[inline]
    fn from(val: PriceStruct) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for PriceStruct {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
