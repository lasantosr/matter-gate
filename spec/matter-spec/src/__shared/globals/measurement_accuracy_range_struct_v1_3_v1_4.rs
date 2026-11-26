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
pub struct MeasurementAccuracyRangeStruct {
    pub range_min: i64,
    pub range_max: i64,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub percent_max: Optional<types::Percent100ths>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub percent_min: Optional<types::Percent100ths>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub percent_typical: Optional<types::Percent100ths>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub fixed_max: Optional<u64>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub fixed_min: Optional<u64>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub fixed_typical: Optional<u64>,
}

impl MeasurementAccuracyRangeStruct {
    pub const FIELD_FIXED_MAX: FieldId = FieldId::new(0x5);
    pub const FIELD_FIXED_MIN: FieldId = FieldId::new(0x6);
    pub const FIELD_FIXED_TYPICAL: FieldId = FieldId::new(0x7);
    pub const FIELD_PERCENT_MAX: FieldId = FieldId::new(0x2);
    pub const FIELD_PERCENT_MIN: FieldId = FieldId::new(0x3);
    pub const FIELD_PERCENT_TYPICAL: FieldId = FieldId::new(0x4);
    pub const FIELD_RANGE_MAX: FieldId = FieldId::new(0x1);
    pub const FIELD_RANGE_MIN: FieldId = FieldId::new(0x0);

    #[must_use]
    pub fn new(range_min: impl Into<i64>, range_max: impl Into<i64>) -> Self {
        Self {
            range_min: range_min.into(),
            range_max: range_max.into(),
            percent_max: Optional::Omitted,
            percent_min: Optional::Omitted,
            percent_typical: Optional::Omitted,
            fixed_max: Optional::Omitted,
            fixed_min: Optional::Omitted,
            fixed_typical: Optional::Omitted,
        }
    }

    #[must_use]
    pub fn with_percent_max(mut self, percent_max: impl Into<types::Percent100ths>) -> Self {
        self.percent_max = Optional::Present(percent_max.into());
        self
    }

    #[must_use]
    pub fn with_percent_min(mut self, percent_min: impl Into<types::Percent100ths>) -> Self {
        self.percent_min = Optional::Present(percent_min.into());
        self
    }

    #[must_use]
    pub fn with_percent_typical(mut self, percent_typical: impl Into<types::Percent100ths>) -> Self {
        self.percent_typical = Optional::Present(percent_typical.into());
        self
    }

    #[must_use]
    pub fn with_fixed_max(mut self, fixed_max: impl Into<u64>) -> Self {
        self.fixed_max = Optional::Present(fixed_max.into());
        self
    }

    #[must_use]
    pub fn with_fixed_min(mut self, fixed_min: impl Into<u64>) -> Self {
        self.fixed_min = Optional::Present(fixed_min.into());
        self
    }

    #[must_use]
    pub fn with_fixed_typical(mut self, fixed_typical: impl Into<u64>) -> Self {
        self.fixed_typical = Optional::Present(fixed_typical.into());
        self
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut range_min = None;
        let mut range_max = None;
        let mut percent_max = None;
        let mut percent_min = None;
        let mut percent_typical = None;
        let mut fixed_max = None;
        let mut fixed_min = None;
        let mut fixed_typical = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if range_min.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<i64>(&fval)?;
                        (-0x4000_0000_0000_0000_i64..=0x4000_0000_0000_0000_i64)
                            .contains(&decoded)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        range_min = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if range_max.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<i64>(&fval)?;
                        (-0x4000_0000_0000_0000_i64..=0x4000_0000_0000_0000_i64)
                            .contains(&decoded)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        range_max = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if percent_max.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <types::Percent100ths>::try_from(fval)?;
                        percent_max = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                3 => {
                    if percent_min.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <types::Percent100ths>::try_from(fval)?;
                        percent_min = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                4 => {
                    if percent_typical.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <types::Percent100ths>::try_from(fval)?;
                        percent_typical = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                5 => {
                    if fixed_max.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u64>(&fval)?;
                        (decoded <= 0x3FFF_FFFF_FFFF_FFFF_u64)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        fixed_max = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                6 => {
                    if fixed_min.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u64>(&fval)?;
                        (decoded <= 0x3FFF_FFFF_FFFF_FFFF_u64)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        fixed_min = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                7 => {
                    if fixed_typical.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u64>(&fval)?;
                        (decoded <= 0x3FFF_FFFF_FFFF_FFFF_u64)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        fixed_typical = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            range_min: range_min.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            range_max: range_max.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            percent_max: Optional::from(percent_max),
            percent_min: Optional::from(percent_min),
            percent_typical: Optional::from(percent_typical),
            fixed_max: Optional::from(fixed_max),
            fixed_min: Optional::from(fixed_min),
            fixed_typical: Optional::from(fixed_typical),
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        let mut fields = Vec::with_capacity(8);
        fields.push((Self::FIELD_RANGE_MIN, AttributeValue::from(self.range_min)));
        fields.push((Self::FIELD_RANGE_MAX, AttributeValue::from(self.range_max)));
        if let Optional::Present(v) = self.percent_max {
            fields.push((Self::FIELD_PERCENT_MAX, AttributeValue::from(v)));
        }
        if let Optional::Present(v) = self.percent_min {
            fields.push((Self::FIELD_PERCENT_MIN, AttributeValue::from(v)));
        }
        if let Optional::Present(v) = self.percent_typical {
            fields.push((Self::FIELD_PERCENT_TYPICAL, AttributeValue::from(v)));
        }
        if let Optional::Present(v) = self.fixed_max {
            fields.push((Self::FIELD_FIXED_MAX, AttributeValue::from(v)));
        }
        if let Optional::Present(v) = self.fixed_min {
            fields.push((Self::FIELD_FIXED_MIN, AttributeValue::from(v)));
        }
        if let Optional::Present(v) = self.fixed_typical {
            fields.push((Self::FIELD_FIXED_TYPICAL, AttributeValue::from(v)));
        }
        fields
    }
}

crate::impl_matter_struct_wire!(MeasurementAccuracyRangeStruct);
