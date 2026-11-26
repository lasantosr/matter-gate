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
pub struct IceCandidateStruct {
    pub candidate: String,
    pub sdp_mid: Nullable<String>,
    pub sdpm_line_index: Nullable<u16>,
}

impl IceCandidateStruct {
    pub const FIELD_CANDIDATE: FieldId = FieldId::new(0x0);
    pub const FIELD_SDPM_LINE_INDEX: FieldId = FieldId::new(0x2);
    pub const FIELD_SDP_MID: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(
        candidate: impl Into<String>,
        sdp_mid: impl Into<Nullable<String>>,
        sdpm_line_index: impl Into<Nullable<u16>>,
    ) -> Self {
        Self {
            candidate: candidate.into(),
            sdp_mid: sdp_mid.into(),
            sdpm_line_index: sdpm_line_index.into(),
        }
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut candidate = None;
        let mut sdp_mid = None;
        let mut sdpm_line_index = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if candidate.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <String>::try_from(fval)?;
                        candidate = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if sdp_mid.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <String>::try_from)?;
                        sdp_mid = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if sdpm_line_index.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded =
                            crate::traits::decode_nullable(fval, |v| crate::traits::decode_scalar::<u16>(&v))?;
                        sdpm_line_index = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            candidate: candidate.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            sdp_mid: sdp_mid.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            sdpm_line_index: sdpm_line_index.ok_or((Some(FieldId::new(2)), MatterStatus::ConstraintError))?,
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        alloc::vec![
            (Self::FIELD_CANDIDATE, AttributeValue::from(self.candidate)),
            (
                Self::FIELD_SDP_MID,
                match self.sdp_mid {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                }
            ),
            (
                Self::FIELD_SDPM_LINE_INDEX,
                match self.sdpm_line_index {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                }
            ),
        ]
    }
}

crate::impl_matter_struct_wire!(IceCandidateStruct);
