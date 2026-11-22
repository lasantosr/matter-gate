#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

use crate::__shared::globals::area_type_tag::*;
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
pub struct LocationDescriptorStruct {
    pub location_name: String,
    pub floor_number: Nullable<i16>,
    pub area_type: Nullable<AreaTypeTag>,
}

impl LocationDescriptorStruct {
    pub const FIELD_AREA_TYPE: FieldId = FieldId::new(0x2);
    pub const FIELD_FLOOR_NUMBER: FieldId = FieldId::new(0x1);
    pub const FIELD_LOCATION_NAME: FieldId = FieldId::new(0x0);

    #[must_use]
    pub fn new(
        location_name: impl Into<String>,
        floor_number: impl Into<Nullable<i16>>,
        area_type: impl Into<Nullable<AreaTypeTag>>,
    ) -> Self {
        Self {
            location_name: location_name.into(),
            floor_number: floor_number.into(),
            area_type: area_type.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut location_name = None;
        let mut floor_number = None;
        let mut area_type = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if location_name.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <String>::try_from(fval)?;
                        (decoded.len() <= 128)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        location_name = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if floor_number.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <i16>::try_from)?;
                        floor_number = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if area_type.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <AreaTypeTag>::try_from)?;
                        area_type = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            location_name: location_name.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            floor_number: floor_number.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            area_type: area_type.ok_or((Some(FieldId::new(2)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (Self::FIELD_LOCATION_NAME, AttributeValue::from(self.location_name)),
            (
                Self::FIELD_FLOOR_NUMBER,
                match self.floor_number {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                }
            ),
            (
                Self::FIELD_AREA_TYPE,
                match self.area_type {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                }
            ),
        ]
    }
}

impl From<LocationDescriptorStruct> for AttributeValue {
    #[inline]
    fn from(val: LocationDescriptorStruct) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for LocationDescriptorStruct {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
