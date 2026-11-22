#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

use crate::__shared::globals::{measurement_accuracy_range_struct_v1_5_v1_6::*, measurement_type_enum::*};
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
pub struct MeasurementAccuracyStruct {
    pub measurement_type: MeasurementTypeEnum,
    pub measured: bool,
    pub min_measured_value: i64,
    pub max_measured_value: i64,
    pub accuracy_ranges: Vec<MeasurementAccuracyRangeStruct>,
}

impl MeasurementAccuracyStruct {
    pub const FIELD_ACCURACY_RANGES: FieldId = FieldId::new(0x4);
    pub const FIELD_MAX_MEASURED_VALUE: FieldId = FieldId::new(0x3);
    pub const FIELD_MEASURED: FieldId = FieldId::new(0x1);
    pub const FIELD_MEASUREMENT_TYPE: FieldId = FieldId::new(0x0);
    pub const FIELD_MIN_MEASURED_VALUE: FieldId = FieldId::new(0x2);

    #[must_use]
    pub fn new(
        measurement_type: impl Into<MeasurementTypeEnum>,
        measured: impl Into<bool>,
        min_measured_value: impl Into<i64>,
        max_measured_value: impl Into<i64>,
        accuracy_ranges: impl Into<Vec<MeasurementAccuracyRangeStruct>>,
    ) -> Self {
        Self {
            measurement_type: measurement_type.into(),
            measured: measured.into(),
            min_measured_value: min_measured_value.into(),
            max_measured_value: max_measured_value.into(),
            accuracy_ranges: accuracy_ranges.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut measurement_type = None;
        let mut measured = None;
        let mut min_measured_value = None;
        let mut max_measured_value = None;
        let mut accuracy_ranges = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if measurement_type.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <MeasurementTypeEnum>::try_from(fval)?;
                        measurement_type = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if measured.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <bool>::try_from(fval)?;
                        measured = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if min_measured_value.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <i64>::try_from(fval)?;
                        (-0x4000_0000_0000_0000_i64..=0x4000_0000_0000_0000_i64)
                            .contains(&decoded)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        min_measured_value = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                3 => {
                    if max_measured_value.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <i64>::try_from(fval)?;
                        (-0x4000_0000_0000_0000_i64..=0x4000_0000_0000_0000_i64)
                            .contains(&decoded)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        max_measured_value = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                4 => {
                    if accuracy_ranges.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_list(fval, <MeasurementAccuracyRangeStruct>::try_from)?;
                        accuracy_ranges = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            measurement_type: measurement_type.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            measured: measured.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            min_measured_value: min_measured_value.ok_or((Some(FieldId::new(2)), MatterStatus::ConstraintError))?,
            max_measured_value: max_measured_value.ok_or((Some(FieldId::new(3)), MatterStatus::ConstraintError))?,
            accuracy_ranges: accuracy_ranges.ok_or((Some(FieldId::new(4)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (
                Self::FIELD_MEASUREMENT_TYPE,
                AttributeValue::from(self.measurement_type)
            ),
            (Self::FIELD_MEASURED, AttributeValue::from(self.measured)),
            (
                Self::FIELD_MIN_MEASURED_VALUE,
                AttributeValue::from(self.min_measured_value)
            ),
            (
                Self::FIELD_MAX_MEASURED_VALUE,
                AttributeValue::from(self.max_measured_value)
            ),
            (
                Self::FIELD_ACCURACY_RANGES,
                AttributeValue::List(self.accuracy_ranges.into_iter().map(AttributeValue::from).collect())
            ),
        ]
    }
}

impl From<MeasurementAccuracyStruct> for AttributeValue {
    #[inline]
    fn from(val: MeasurementAccuracyStruct) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for MeasurementAccuracyStruct {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
