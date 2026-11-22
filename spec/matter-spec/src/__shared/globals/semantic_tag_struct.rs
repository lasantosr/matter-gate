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
pub struct SemanticTagStruct {
    pub mfg_code: Nullable<VendorId>,
    pub namespace_id: u8,
    pub tag: u8,
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Optional::is_omitted",
            deserialize_with = "crate::model::serde::deserialize_optional_nullable"
        )
    )]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub label: Optional<Nullable<String>>,
}

impl SemanticTagStruct {
    pub const FIELD_LABEL: FieldId = FieldId::new(0x3);
    pub const FIELD_MFG_CODE: FieldId = FieldId::new(0x0);
    pub const FIELD_NAMESPACE_ID: FieldId = FieldId::new(0x1);
    pub const FIELD_TAG: FieldId = FieldId::new(0x2);

    #[must_use]
    pub fn new(mfg_code: impl Into<Nullable<VendorId>>, namespace_id: impl Into<u8>, tag: impl Into<u8>) -> Self {
        Self {
            mfg_code: mfg_code.into(),
            namespace_id: namespace_id.into(),
            tag: tag.into(),
            label: Optional::Omitted,
        }
    }

    #[must_use]
    pub fn with_label(mut self, label: impl Into<Nullable<String>>) -> Self {
        self.label = Optional::Present(label.into());
        self
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut mfg_code = None;
        let mut namespace_id = None;
        let mut tag = None;
        let mut label = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if mfg_code.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <VendorId>::try_from)?;
                        mfg_code = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if namespace_id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <u8>::try_from(fval)?;
                        namespace_id = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if tag.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <u8>::try_from(fval)?;
                        tag = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                3 => {
                    if label.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <String>::try_from)?;
                        (!matches!(&decoded, Nullable::NonNull(v) if v.len() > 64))
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        label = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            mfg_code: mfg_code.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            namespace_id: namespace_id.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            tag: tag.ok_or((Some(FieldId::new(2)), MatterStatus::ConstraintError))?,
            label: Optional::from(label),
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        let mut fields = Vec::with_capacity(4);
        match self.mfg_code {
            Nullable::NonNull(v) => {
                fields.push((Self::FIELD_MFG_CODE, AttributeValue::from(v)));
            }
            Nullable::Null => {
                fields.push((Self::FIELD_MFG_CODE, AttributeValue::Null));
            }
        }
        fields.push((Self::FIELD_NAMESPACE_ID, AttributeValue::from(self.namespace_id)));
        fields.push((Self::FIELD_TAG, AttributeValue::from(self.tag)));
        if let Optional::Present(inner) = self.label {
            match inner {
                Nullable::NonNull(v) => {
                    fields.push((Self::FIELD_LABEL, AttributeValue::from(v)));
                }
                Nullable::Null => {
                    fields.push((Self::FIELD_LABEL, AttributeValue::Null));
                }
            }
        }
        fields
    }
}

impl From<SemanticTagStruct> for AttributeValue {
    #[inline]
    fn from(val: SemanticTagStruct) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for SemanticTagStruct {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
