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
pub struct IceServerStruct {
    pub ur_ls: Vec<String>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub username: Optional<String>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub credential: Optional<String>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub caid: Optional<u16>,
}

impl IceServerStruct {
    pub const FIELD_CAID: FieldId = FieldId::new(0x3);
    pub const FIELD_CREDENTIAL: FieldId = FieldId::new(0x2);
    pub const FIELD_UR_LS: FieldId = FieldId::new(0x0);
    pub const FIELD_USERNAME: FieldId = FieldId::new(0x1);

    #[must_use]
    pub fn new(ur_ls: impl Into<Vec<String>>) -> Self {
        Self {
            ur_ls: ur_ls.into(),
            username: Optional::Omitted,
            credential: Optional::Omitted,
            caid: Optional::Omitted,
        }
    }

    #[must_use]
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Optional::Present(username.into());
        self
    }

    #[must_use]
    pub fn with_credential(mut self, credential: impl Into<String>) -> Self {
        self.credential = Optional::Present(credential.into());
        self
    }

    #[must_use]
    pub fn with_caid(mut self, caid: impl Into<u16>) -> Self {
        self.caid = Optional::Present(caid.into());
        self
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut ur_ls = None;
        let mut username = None;
        let mut credential = None;
        let mut caid = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if ur_ls.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_list(fval, <String>::try_from)?;
                        (decoded.len() <= 10)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        ur_ls = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if username.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <String>::try_from(fval)?;
                        (decoded.len() <= 508)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        username = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if credential.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = <String>::try_from(fval)?;
                        (decoded.len() <= 512)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        credential = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                3 => {
                    if caid.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_scalar::<u16>(&fval)?;
                        (decoded <= 0xFFFE_u16)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        caid = Some(decoded);
                        Ok(())
                    })()
                    .map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            ur_ls: ur_ls.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            username: Optional::from(username),
            credential: Optional::from(credential),
            caid: Optional::from(caid),
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        let mut fields = Vec::with_capacity(4);
        fields.push((
            Self::FIELD_UR_LS,
            AttributeValue::List(self.ur_ls.into_iter().map(AttributeValue::from).collect()),
        ));
        if let Optional::Present(v) = self.username {
            fields.push((Self::FIELD_USERNAME, AttributeValue::from(v)));
        }
        if let Optional::Present(v) = self.credential {
            fields.push((Self::FIELD_CREDENTIAL, AttributeValue::from(v)));
        }
        if let Optional::Present(v) = self.caid {
            fields.push((Self::FIELD_CAID, AttributeValue::from(v)));
        }
        fields
    }
}

impl From<Vec<String>> for IceServerStruct {
    #[inline]
    fn from(val: Vec<String>) -> Self {
        Self::new(val)
    }
}

crate::impl_matter_struct_wire!(IceServerStruct);
