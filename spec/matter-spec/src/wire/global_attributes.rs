use alloc::vec::Vec;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    model::{AttributeId, AttributeValue, CommandId, EventId, MatterStatus, Optional},
    traits::decode_unsigned,
};

/// Global attributes (`0xFFF8..=0xFFFD`) common to Matter clusters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub struct GlobalAttributes<F = u32> {
    /// `GeneratedCommandList` (`0xFFF8`).
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub generated_command_list: Optional<Vec<CommandId>>,
    /// `AcceptedCommandList` (`0xFFF9`).
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub accepted_command_list: Optional<Vec<CommandId>>,
    /// `EventList` (`0xFFFA`, present in Matter `v1.1`–`v1.3`).
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub event_list: Optional<Vec<EventId>>,
    /// `AttributeList` (`0xFFFB`).
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub attribute_list: Optional<Vec<AttributeId>>,
    /// `FeatureMap` (`0xFFFC`).
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub feature_map: Optional<F>,
    /// `ClusterRevision` (`0xFFFD`).
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub cluster_revision: Optional<u16>,
}

impl<F> GlobalAttributes<F> {
    /// Constructs an empty [`GlobalAttributes`] with all fields omitted.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            generated_command_list: Optional::Omitted,
            accepted_command_list: Optional::Omitted,
            event_list: Optional::Omitted,
            attribute_list: Optional::Omitted,
            feature_map: Optional::Omitted,
            cluster_revision: Optional::Omitted,
        }
    }

    /// Sets `generated_command_list` (`0xFFF8`).
    #[must_use]
    pub fn with_generated_command_list(mut self, list: Vec<CommandId>) -> Self {
        self.generated_command_list = Optional::Present(list);
        self
    }

    /// Sets `accepted_command_list` (`0xFFF9`).
    #[must_use]
    pub fn with_accepted_command_list(mut self, list: Vec<CommandId>) -> Self {
        self.accepted_command_list = Optional::Present(list);
        self
    }

    /// Sets `event_list` (`0xFFFA`).
    #[must_use]
    pub fn with_event_list(mut self, list: Vec<EventId>) -> Self {
        self.event_list = Optional::Present(list);
        self
    }

    /// Sets `attribute_list` (`0xFFFB`).
    #[must_use]
    pub fn with_attribute_list(mut self, list: Vec<AttributeId>) -> Self {
        self.attribute_list = Optional::Present(list);
        self
    }

    /// Sets `feature_map` (`0xFFFC`).
    #[must_use]
    pub fn with_feature_map(mut self, feature_map: F) -> Self {
        self.feature_map = Optional::Present(feature_map);
        self
    }

    /// Sets `cluster_revision` (`0xFFFD`).
    #[must_use]
    pub const fn with_cluster_revision(mut self, revision: u16) -> Self {
        self.cluster_revision = Optional::Present(revision);
        self
    }

    /// Returns `true` if `id` is a Matter global attribute (`0xFFF8..=0xFFFD`).
    #[must_use]
    pub const fn is_global_id(id: AttributeId) -> bool {
        matches!(
            id,
            AttributeId::GENERATED_COMMAND_LIST
                | AttributeId::ACCEPTED_COMMAND_LIST
                | AttributeId::EVENT_LIST
                | AttributeId::ATTRIBUTE_LIST
                | AttributeId::FEATURE_MAP
                | AttributeId::CLUSTER_REVISION
        )
    }

    /// Attempts to apply a global attribute (`0xFFF8..=0xFFFD`).
    ///
    /// Returns `Ok(true)` if `id` was a global attribute and was applied,
    /// `Ok(false)` if `id` is not a global attribute, or `Err(MatterStatus)`
    /// if the wire value has an invalid type or duplicate/invalid data.
    pub fn apply_global_attribute(&mut self, id: AttributeId, value: AttributeValue) -> Result<bool, MatterStatus>
    where
        F: TryFrom<u32>,
    {
        match id {
            AttributeId::GENERATED_COMMAND_LIST => {
                let AttributeValue::List(items) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.generated_command_list.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    let raw = decode_unsigned::<u32>(&item)?;
                    list.push(CommandId::new(raw));
                }
                self.generated_command_list = Optional::Present(list);
                Ok(true)
            }
            AttributeId::ACCEPTED_COMMAND_LIST => {
                let AttributeValue::List(items) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.accepted_command_list.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    let raw = decode_unsigned::<u32>(&item)?;
                    list.push(CommandId::new(raw));
                }
                self.accepted_command_list = Optional::Present(list);
                Ok(true)
            }
            AttributeId::EVENT_LIST => {
                let AttributeValue::List(items) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.event_list.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    let raw = decode_unsigned::<u32>(&item)?;
                    list.push(EventId::new(raw));
                }
                self.event_list = Optional::Present(list);
                Ok(true)
            }
            AttributeId::ATTRIBUTE_LIST => {
                let AttributeValue::List(items) = value else {
                    return Err(MatterStatus::InvalidDataType);
                };
                if self.attribute_list.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let mut list = Vec::with_capacity(items.len());
                for item in items {
                    let raw = decode_unsigned::<u32>(&item)?;
                    list.push(AttributeId::new(raw));
                }
                self.attribute_list = Optional::Present(list);
                Ok(true)
            }
            AttributeId::FEATURE_MAP => {
                if self.feature_map.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let raw = decode_unsigned::<u32>(&value)?;
                self.feature_map = Optional::Present(F::try_from(raw).map_err(|_| MatterStatus::ConstraintError)?);
                Ok(true)
            }
            AttributeId::CLUSTER_REVISION => {
                if self.cluster_revision.is_present() {
                    return Err(MatterStatus::ConstraintError);
                }
                let rev = decode_unsigned::<u16>(&value)?;
                self.cluster_revision = Optional::Present(rev);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Appends all present global attributes as `(AttributeId, AttributeValue)` pairs.
    pub fn into_attributes(self, out: &mut Vec<(AttributeId, AttributeValue)>)
    where
        F: Into<u32>,
    {
        if let Optional::Present(list) = self.generated_command_list {
            out.push((
                AttributeId::GENERATED_COMMAND_LIST,
                AttributeValue::List(list.into_iter().map(|id| AttributeValue::U32(id.raw())).collect()),
            ));
        }
        if let Optional::Present(list) = self.accepted_command_list {
            out.push((
                AttributeId::ACCEPTED_COMMAND_LIST,
                AttributeValue::List(list.into_iter().map(|id| AttributeValue::U32(id.raw())).collect()),
            ));
        }
        if let Optional::Present(list) = self.event_list {
            out.push((
                AttributeId::EVENT_LIST,
                AttributeValue::List(list.into_iter().map(|id| AttributeValue::U32(id.raw())).collect()),
            ));
        }
        if let Optional::Present(list) = self.attribute_list {
            out.push((
                AttributeId::ATTRIBUTE_LIST,
                AttributeValue::List(list.into_iter().map(|id| AttributeValue::U32(id.raw())).collect()),
            ));
        }
        if let Optional::Present(fm) = self.feature_map {
            out.push((AttributeId::FEATURE_MAP, AttributeValue::U32(fm.into())));
        }
        if let Optional::Present(rev) = self.cluster_revision {
            out.push((AttributeId::CLUSTER_REVISION, AttributeValue::U16(rev)));
        }
    }
}
