#[allow(unused_imports, reason = "Generated global imports")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
#[allow(unused_imports, reason = "Generated global imports")]
use serde::{Deserialize, Serialize};

use crate::__shared::globals::stream_usage_enum::*;
#[allow(unused_imports, reason = "Generated global imports")]
use crate::{
    model::{
        ActionId, AttributeId, AttributeValue, ClusterId, CommandId, DataVersion, DeviceTypeId, EndpointId, EventId,
        EventNumber, FabricId, FabricIndex, FieldId, GroupId, MatterStatus, NodeId, Nullable, Optional, TransactionId,
        VendorId,
    },
    types,
};
/// Fabric-scoped data structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub struct WebRtcSessionStruct {
    pub id: u16,
    pub peer_node_id: NodeId,
    pub peer_endpoint_id: EndpointId,
    pub stream_usage: StreamUsageEnum,
    pub video_stream_id: Nullable<u16>,
    pub audio_stream_id: Nullable<u16>,
    pub metadata_enabled: bool,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub video_streams: Optional<Vec<u16>>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub audio_streams: Optional<Vec<u16>>,
}

impl WebRtcSessionStruct {
    pub const FIELD_AUDIO_STREAMS: FieldId = FieldId::new(0x8);
    pub const FIELD_AUDIO_STREAM_ID: FieldId = FieldId::new(0x5);
    pub const FIELD_ID: FieldId = FieldId::new(0x0);
    pub const FIELD_METADATA_ENABLED: FieldId = FieldId::new(0x6);
    pub const FIELD_PEER_ENDPOINT_ID: FieldId = FieldId::new(0x2);
    pub const FIELD_PEER_NODE_ID: FieldId = FieldId::new(0x1);
    pub const FIELD_STREAM_USAGE: FieldId = FieldId::new(0x3);
    pub const FIELD_VIDEO_STREAMS: FieldId = FieldId::new(0x7);
    pub const FIELD_VIDEO_STREAM_ID: FieldId = FieldId::new(0x4);

    #[must_use]
    pub fn new(
        id: impl Into<u16>,
        peer_node_id: impl Into<NodeId>,
        peer_endpoint_id: impl Into<EndpointId>,
        stream_usage: impl Into<StreamUsageEnum>,
        video_stream_id: impl Into<Nullable<u16>>,
        audio_stream_id: impl Into<Nullable<u16>>,
        metadata_enabled: impl Into<bool>,
    ) -> Self {
        Self {
            id: id.into(),
            peer_node_id: peer_node_id.into(),
            peer_endpoint_id: peer_endpoint_id.into(),
            stream_usage: stream_usage.into(),
            video_stream_id: video_stream_id.into(),
            audio_stream_id: audio_stream_id.into(),
            metadata_enabled: metadata_enabled.into(),
            video_streams: Optional::Omitted,
            audio_streams: Optional::Omitted,
        }
    }

    #[must_use]
    pub fn with_video_streams(mut self, video_streams: impl Into<Vec<u16>>) -> Self {
        self.video_streams = Optional::Present(video_streams.into());
        self
    }

    #[must_use]
    pub fn with_audio_streams(mut self, audio_streams: impl Into<Vec<u16>>) -> Self {
        self.audio_streams = Optional::Present(audio_streams.into());
        self
    }

    pub(crate) fn decode_fields_inner(
        fields: Vec<(FieldId, AttributeValue)>,
    ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
        let mut id = None;
        let mut peer_node_id = None;
        let mut peer_endpoint_id = None;
        let mut stream_usage = None;
        let mut video_stream_id = None;
        let mut audio_stream_id = None;
        let mut metadata_enabled = None;
        let mut video_streams = None;
        let mut audio_streams = None;
        for (fid, fval) in fields {
            match fid.raw() {
                0 => {
                    if id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <u16>::try_from(fval)?;
                        id = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                1 => {
                    if peer_node_id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <NodeId>::try_from(fval)?;
                        peer_node_id = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                2 => {
                    if peer_endpoint_id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <EndpointId>::try_from(fval)?;
                        peer_endpoint_id = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                3 => {
                    if stream_usage.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <StreamUsageEnum>::try_from(fval)?;
                        stream_usage = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                4 => {
                    if video_stream_id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <u16>::try_from)?;
                        video_stream_id = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                5 => {
                    if audio_stream_id.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_nullable(fval, <u16>::try_from)?;
                        audio_stream_id = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                6 => {
                    if metadata_enabled.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = <bool>::try_from(fval)?;
                        metadata_enabled = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                7 => {
                    if video_streams.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_list(fval, <u16>::try_from)?;
                        (decoded.len() <= 16)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        video_streams = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                8 => {
                    if audio_streams.is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    let decode_one = || -> Result<(), MatterStatus> {
                        let decoded = crate::traits::decode_list(fval, <u16>::try_from)?;
                        (decoded.len() <= 16)
                            .then_some(())
                            .ok_or(MatterStatus::ConstraintError)?;
                        audio_streams = Some(decoded);
                        Ok(())
                    };
                    decode_one().map_err(|st| (Some(fid), st))?;
                }
                _ => return Err((Some(fid), MatterStatus::ConstraintError)),
            }
        }
        Ok(Self {
            id: id.ok_or((Some(FieldId::new(0)), MatterStatus::ConstraintError))?,
            peer_node_id: peer_node_id.ok_or((Some(FieldId::new(1)), MatterStatus::ConstraintError))?,
            peer_endpoint_id: peer_endpoint_id.ok_or((Some(FieldId::new(2)), MatterStatus::ConstraintError))?,
            stream_usage: stream_usage.ok_or((Some(FieldId::new(3)), MatterStatus::ConstraintError))?,
            video_stream_id: video_stream_id.ok_or((Some(FieldId::new(4)), MatterStatus::ConstraintError))?,
            audio_stream_id: audio_stream_id.ok_or((Some(FieldId::new(5)), MatterStatus::ConstraintError))?,
            metadata_enabled: metadata_enabled.ok_or((Some(FieldId::new(6)), MatterStatus::ConstraintError))?,
            video_streams: Optional::from(video_streams),
            audio_streams: Optional::from(audio_streams),
        })
    }

    pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
        let mut fields = Vec::with_capacity(9);
        fields.push((Self::FIELD_ID, AttributeValue::from(self.id)));
        fields.push((Self::FIELD_PEER_NODE_ID, AttributeValue::from(self.peer_node_id)));
        fields.push((
            Self::FIELD_PEER_ENDPOINT_ID,
            AttributeValue::from(self.peer_endpoint_id),
        ));
        fields.push((Self::FIELD_STREAM_USAGE, AttributeValue::from(self.stream_usage)));
        match self.video_stream_id {
            Nullable::NonNull(v) => {
                fields.push((Self::FIELD_VIDEO_STREAM_ID, AttributeValue::from(v)));
            }
            Nullable::Null => {
                fields.push((Self::FIELD_VIDEO_STREAM_ID, AttributeValue::Null));
            }
        }
        match self.audio_stream_id {
            Nullable::NonNull(v) => {
                fields.push((Self::FIELD_AUDIO_STREAM_ID, AttributeValue::from(v)));
            }
            Nullable::Null => {
                fields.push((Self::FIELD_AUDIO_STREAM_ID, AttributeValue::Null));
            }
        }
        fields.push((
            Self::FIELD_METADATA_ENABLED,
            AttributeValue::from(self.metadata_enabled),
        ));
        if let Optional::Present(v) = self.video_streams {
            fields.push((
                Self::FIELD_VIDEO_STREAMS,
                AttributeValue::List(v.into_iter().map(AttributeValue::from).collect()),
            ));
        }
        if let Optional::Present(v) = self.audio_streams {
            fields.push((
                Self::FIELD_AUDIO_STREAMS,
                AttributeValue::List(v.into_iter().map(AttributeValue::from).collect()),
            ));
        }
        fields
    }
}

impl From<WebRtcSessionStruct> for AttributeValue {
    #[inline]
    fn from(val: WebRtcSessionStruct) -> Self {
        AttributeValue::Struct(val.encode_fields_inner())
    }
}

impl TryFrom<AttributeValue> for WebRtcSessionStruct {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        let AttributeValue::Struct(fields) = val else {
            return Err(MatterStatus::InvalidDataType);
        };
        Self::decode_fields_inner(fields).map_err(|(_, status)| status)
    }
}
