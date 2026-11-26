#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

use crate::__shared::globals::{test_global_bitmap::*, test_global_enum::*};
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
pub struct TestGlobalStruct {
    pub name: String,
    pub my_bitmap: Nullable<TestGlobalBitmap>,
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Optional::is_omitted",
            deserialize_with = "crate::model::serde::deserialize_optional_nullable"
        )
    )]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub my_enum: Optional<Nullable<TestGlobalEnum>>,
}

impl TestGlobalStruct {
    pub const FIELD_MY_BITMAP: FieldId = FieldId::new(0x1);
    pub const FIELD_MY_ENUM: FieldId = FieldId::new(0x2);
    pub const FIELD_NAME: FieldId = FieldId::new(0x0);

    #[must_use]
    pub fn new(name: impl Into<String>, my_bitmap: impl Into<Nullable<TestGlobalBitmap>>) -> Self {
        Self {
            name: name.into(),
            my_bitmap: my_bitmap.into(),
            my_enum: Optional::Omitted,
        }
    }

    #[must_use]
    pub fn with_my_enum(mut self, my_enum: impl Into<Nullable<TestGlobalEnum>>) -> Self {
        self.my_enum = Optional::Present(my_enum.into());
        self
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut name = None;
        let mut my_bitmap = None;
        let mut my_enum = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if name.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <String>::try_from(fval)?;
                        (decoded.len() <= 128)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        name = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if my_bitmap.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <TestGlobalBitmap>::try_from)?;
                        my_bitmap = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if my_enum.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <TestGlobalEnum>::try_from)?;
                        my_enum = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            name: name.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            my_bitmap: my_bitmap.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            my_enum: Optional::from(my_enum),
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        let mut fields = Vec::with_capacity(3);
        fields.push((Self::FIELD_NAME, AttributeValue::from(self.name)));
        match self.my_bitmap {
            Nullable::NonNull(v) => {
                fields.push((Self::FIELD_MY_BITMAP, AttributeValue::from(v)));
            }
            Nullable::Null => {
                fields.push((Self::FIELD_MY_BITMAP, AttributeValue::Null));
            }
        }
        if let Optional::Present(inner) = self.my_enum {
            match inner {
                Nullable::NonNull(v) => {
                    fields.push((Self::FIELD_MY_ENUM, AttributeValue::from(v)));
                }
                Nullable::Null => {
                    fields.push((Self::FIELD_MY_ENUM, AttributeValue::Null));
                }
            }
        }
        fields
    }
}

crate::impl_matter_struct_wire!(TestGlobalStruct);
