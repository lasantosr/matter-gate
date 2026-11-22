#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

use crate::__shared::globals::power_threshold_source_enum::*;
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
pub struct PowerThresholdStruct {
    pub power_threshold_source: Nullable<PowerThresholdSourceEnum>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub power_threshold: Optional<types::PowerMw>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub apparent_power_threshold: Optional<types::PowerMva>,
}

impl PowerThresholdStruct {
    pub const FIELD_APPARENT_POWER_THRESHOLD: FieldId = FieldId::new(0x1);
    pub const FIELD_POWER_THRESHOLD: FieldId = FieldId::new(0x0);
    pub const FIELD_POWER_THRESHOLD_SOURCE: FieldId = FieldId::new(0x2);

    #[must_use]
    pub fn new(power_threshold_source: impl Into<Nullable<PowerThresholdSourceEnum>>) -> Self {
        Self {
            power_threshold_source: power_threshold_source.into(),
            power_threshold: Optional::Omitted,
            apparent_power_threshold: Optional::Omitted,
        }
    }

    #[must_use]
    pub fn with_power_threshold(mut self, power_threshold: impl Into<types::PowerMw>) -> Self {
        self.power_threshold = Optional::Present(power_threshold.into());
        self
    }

    #[must_use]
    pub fn with_apparent_power_threshold(mut self, apparent_power_threshold: impl Into<types::PowerMva>) -> Self {
        self.apparent_power_threshold = Optional::Present(apparent_power_threshold.into());
        self
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut power_threshold = None;
        let mut apparent_power_threshold = None;
        let mut power_threshold_source = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if power_threshold.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <types::PowerMw>::try_from(fval)?;
                        (-0x4000_0000_0000_0000_i64..=0x3FFF_FFFF_FFFF_FFFF_i64)
                            .contains(&decoded.raw())
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        power_threshold = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if apparent_power_threshold.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <types::PowerMva>::try_from(fval)?;
                        (-0x4000_0000_0000_0000_i64..=0x3FFF_FFFF_FFFF_FFFF_i64)
                            .contains(&decoded.raw())
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        apparent_power_threshold = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if power_threshold_source.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <PowerThresholdSourceEnum>::try_from)?;
                        power_threshold_source = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            power_threshold: Optional::from(power_threshold),
            apparent_power_threshold: Optional::from(apparent_power_threshold),
            power_threshold_source: power_threshold_source
                .ok_or((Some(FieldId::new(2)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        let mut fields = Vec::with_capacity(3);
        if let Optional::Present(v) = self.power_threshold {
            fields.push((Self::FIELD_POWER_THRESHOLD, AttributeValue::from(v)));
        }
        if let Optional::Present(v) = self.apparent_power_threshold {
            fields.push((Self::FIELD_APPARENT_POWER_THRESHOLD, AttributeValue::from(v)));
        }
        match self.power_threshold_source {
            Nullable::NonNull(v) => {
                fields.push((Self::FIELD_POWER_THRESHOLD_SOURCE, AttributeValue::from(v)));
            }
            Nullable::Null => {
                fields.push((Self::FIELD_POWER_THRESHOLD_SOURCE, AttributeValue::Null));
            }
        }
        fields
    }
}

impl From<Nullable<PowerThresholdSourceEnum>> for PowerThresholdStruct {
    #[inline]
    fn from(val: Nullable<PowerThresholdSourceEnum>) -> Self {
        Self::new(val)
    }
}

impl From<PowerThresholdSourceEnum> for PowerThresholdStruct {
    #[inline]
    fn from(val: PowerThresholdSourceEnum) -> Self {
        Self::new(val)
    }
}

impl From<Option<PowerThresholdSourceEnum>> for PowerThresholdStruct {
    #[inline]
    fn from(val: Option<PowerThresholdSourceEnum>) -> Self {
        Self::new(val)
    }
}

impl From<PowerThresholdStruct> for AttributeValue {
    #[inline]
    fn from(val: PowerThresholdStruct) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for PowerThresholdStruct {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
