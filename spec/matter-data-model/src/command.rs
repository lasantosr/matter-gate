//! Matter Command entity per CSA Matter Specification Section 7.9.

extern crate alloc;

use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
    cluster::ClusterId,
    status::MatterStatus,
    value::{AttributeValue, FieldId},
};

define_matter_id! {
    /// Strongly-typed Matter Command identifier (`u32`).
    pub struct CommandId(u32);
    display = hex_32_16_or_8;
    mei = true;
}

impl CommandId {
    /// Wildcard Command ID (`0xFFFF_FFFF`) per CSA Matter Interaction Model §10.5.
    pub const WILDCARD: Self = Self(0xFFFF_FFFF);

    /// Return `true` if this is the Wildcard command (`0xFFFF_FFFF`).
    #[must_use]
    pub const fn is_wildcard(self) -> bool {
        self.0 == 0xFFFF_FFFF
    }

    /// Return `true` if this command ID lies within CSA Matter Core Specification §7.9 allocation ranges
    /// (`sub_id` in `0x0000..=0x00FF` with standard or valid manufacturer vendor prefix).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        (self.is_standard() || self.vendor_id().is_valid()) && self.sub_id() <= 0x00FF
    }
}

/// Direction of a Matter cluster command per the Interaction Model specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum CommandDirection {
    /// Request sent from a client to a server cluster.
    #[default]
    ClientToServer = 0,
    /// Response or command sent from a server cluster to a client.
    ServerToClient = 1,
}

impl From<CommandDirection> for u8 {
    fn from(dir: CommandDirection) -> Self {
        match dir {
            CommandDirection::ClientToServer => 0,
            CommandDirection::ServerToClient => 1,
        }
    }
}

impl TryFrom<u8> for CommandDirection {
    type Error = MatterStatus;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::ClientToServer),
            1 => Ok(Self::ServerToClient),
            _ => Err(MatterStatus::ConstraintError),
        }
    }
}

impl Display for CommandDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::ClientToServer => write!(f, "ClientToServer"),
            Self::ServerToClient => write!(f, "ServerToClient"),
        }
    }
}

impl From<CommandDirection> for AttributeValue {
    fn from(dir: CommandDirection) -> Self {
        Self::U8(u8::from(dir))
    }
}

impl TryFrom<&AttributeValue> for CommandDirection {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::U8(v) => Self::try_from(*v),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<AttributeValue> for CommandDirection {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        (&val).try_into()
    }
}

/// Wire representation of a cluster-scoped Matter Command invocation or response per Section 7.9.
///
/// Commands are standalone envelopes carrying [`Self::cluster_id`] without an endpoint or node
/// address; target/source `(NodeId, EndpointId)` routing lives at the Interaction Model layer.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct Command {
    /// Target or source Matter cluster identifier.
    pub cluster_id: ClusterId,
    /// Matter command identifier within the cluster.
    pub command_id: CommandId,
    /// Direction of the command in the Interaction Model.
    pub direction: CommandDirection,
    /// Context-tagged TLV struct fields of the command payload (empty for unit commands).
    pub payload: Vec<(FieldId, AttributeValue)>,
}

impl Command {
    /// Construct a new command with TLV struct payload fields.
    #[must_use]
    pub const fn new(
        cluster_id: ClusterId,
        command_id: CommandId,
        direction: CommandDirection,
        payload: Vec<(FieldId, AttributeValue)>,
    ) -> Self {
        Self {
            cluster_id,
            command_id,
            direction,
            payload,
        }
    }

    /// Construct a unit command with an empty payload field list.
    #[must_use]
    pub const fn unit(cluster_id: ClusterId, command_id: CommandId, direction: CommandDirection) -> Self {
        Self {
            cluster_id,
            command_id,
            direction,
            payload: Vec::new(),
        }
    }

    /// Set or replace the command payload fields.
    #[must_use]
    pub fn with_payload(mut self, payload: impl Into<Vec<(FieldId, AttributeValue)>>) -> Self {
        self.payload = payload.into();
        self
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}/{}:{} ({} fields)",
            self.cluster_id,
            self.command_id,
            self.direction,
            self.payload.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VendorId;

    #[test]
    fn command_id_and_construction() {
        assert_eq!(CommandId::WILDCARD.raw(), 0xFFFF_FFFF);

        let cid = CommandId::new(0x01);
        assert_eq!(cid.raw(), 0x01);
        assert_eq!(alloc::format!("{cid}"), "0x01");
        assert_eq!(u32::from(cid), 0x01);

        const ON_OFF_CLUSTER_ID: ClusterId = ClusterId::new(0x0006);

        let fields = alloc::vec![(FieldId::new(0), AttributeValue::Bool(true))];
        let cmd = Command::new(ON_OFF_CLUSTER_ID, cid, CommandDirection::ClientToServer, fields.clone());
        assert_eq!(cmd.cluster_id, ON_OFF_CLUSTER_ID);
        assert_eq!(cmd.command_id, cid);
        assert_eq!(cmd.direction, CommandDirection::ClientToServer);
        assert_eq!(cmd.payload, fields);

        let unit_cmd = Command::unit(ON_OFF_CLUSTER_ID, cid, CommandDirection::ServerToClient);
        assert_eq!(unit_cmd.payload, []);
        let with_payload_cmd = unit_cmd
            .clone()
            .with_payload(alloc::vec![(FieldId::new(0), AttributeValue::U8(42))]);
        assert_eq!(
            with_payload_cmd.payload,
            alloc::vec![(FieldId::new(0), AttributeValue::U8(42))]
        );

        let val = AttributeValue::from(cid);
        assert_eq!(CommandId::try_from(&val), Ok(cid));
        assert_eq!(CommandId::try_from(val), Ok(cid));

        let dir_val = AttributeValue::from(CommandDirection::ServerToClient);
        assert_eq!(
            CommandDirection::try_from(&dir_val),
            Ok(CommandDirection::ServerToClient)
        );
        assert_eq!(
            CommandDirection::try_from(dir_val),
            Ok(CommandDirection::ServerToClient)
        );
        assert_eq!(alloc::format!("{unit_cmd}"), "0x0006/0x01:ServerToClient (0 fields)");
        assert_eq!("0x01".parse::<CommandId>(), Ok(cid));
        assert!(cid.is_valid());
        assert!(CommandId::from_mei(VendorId::new(0xFFF1), 0x00FF).is_valid());
        assert!(!CommandId::new(0x0100).is_valid());
        assert!(!CommandId::WILDCARD.is_valid());
    }
}
