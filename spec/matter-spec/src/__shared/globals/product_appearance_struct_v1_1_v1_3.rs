#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

use crate::__shared::globals::{color_enum::*, product_finish_enum::*};
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
pub struct ProductAppearanceStruct {
    pub finish: ProductFinishEnum,
    pub primary_color: Nullable<ColorEnum>,
}

impl ProductAppearanceStruct {
    pub const FIELD_FINISH: FieldId = FieldId::new(0x0);
    pub const FIELD_PRIMARY_COLOR: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(finish: impl Into<ProductFinishEnum>, primary_color: impl Into<Nullable<ColorEnum>>) -> Self {
        Self {
            finish: finish.into(),
            primary_color: primary_color.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut finish = None;
        let mut primary_color = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if finish.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <ProductFinishEnum>::try_from(fval)?;
                        finish = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if primary_color.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <ColorEnum>::try_from)?;
                        primary_color = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            finish: finish.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            primary_color: primary_color.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (Self::FIELD_FINISH, AttributeValue::from(self.finish)),
            (
                Self::FIELD_PRIMARY_COLOR,
                match self.primary_color {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                }
            ),
        ]
    }
}

crate::impl_matter_struct_wire!(ProductAppearanceStruct);
