use core::ops::{Deref, DerefMut};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "dispatch", feature = "serde"))]
use serde::{Deserializer, de::DeserializeOwned};

#[cfg(all(feature = "dispatch", feature = "serde"))]
use crate::dispatch::deserialize_concrete_described;
#[cfg(feature = "dispatch")]
use crate::dispatch::{EntityDescriptor, MatterDescribe, StaticEntitySchema};
use crate::{
    error::EventError,
    model::{Event as ModelEvent, EventNumber, EventPriority, EventTimestamp, MatterStatus, Optional},
};

/// Thin wire-metadata envelope around a Matter event `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub struct WireEvent<T> {
    /// Wire event priority.
    pub priority: EventPriority,
    /// Optional wire `EventNumber`.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub event_number: Optional<EventNumber>,
    /// Optional wire `EventTimestamp`.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub timestamp: Optional<EventTimestamp>,
    /// Typed event payload.
    pub event: T,
}

impl<T> WireEvent<T> {
    /// Wraps `event` with its `priority`.
    #[must_use]
    pub const fn new(priority: EventPriority, event: T) -> Self {
        Self {
            priority,
            event_number: Optional::Omitted,
            timestamp: Optional::Omitted,
            event,
        }
    }

    /// Verifies that `wire_priority == expected_priority` before constructing [`WireEvent`].
    pub fn try_new(
        wire_priority: EventPriority,
        expected_priority: EventPriority,
        event: T,
    ) -> Result<Self, MatterStatus> {
        if wire_priority != expected_priority {
            return Err(MatterStatus::InvalidDataType);
        }
        Ok(Self::new(wire_priority, event))
    }

    /// Sets `event_number`.
    #[must_use]
    pub const fn with_event_number(mut self, event_number: EventNumber) -> Self {
        self.event_number = Optional::Present(event_number);
        self
    }

    /// Sets `timestamp`.
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp: EventTimestamp) -> Self {
        self.timestamp = Optional::Present(timestamp);
        self
    }

    /// Returns a shared reference to the inner event payload.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.event
    }

    /// Returns a mutable reference to the inner event payload.
    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.event
    }

    /// Consumes the wrapper and returns the inner event payload.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.event
    }
}

impl<T> From<WireEvent<T>> for ModelEvent
where
    T: Into<ModelEvent>,
{
    fn from(wire: WireEvent<T>) -> Self {
        let mut event: Self = wire.event.into();
        event.priority = wire.priority;
        event.event_number = wire.event_number;
        event.timestamp = wire.timestamp;
        event
    }
}

impl<T> TryFrom<ModelEvent> for WireEvent<T>
where
    T: TryFrom<ModelEvent, Error = EventError>,
{
    type Error = EventError;

    fn try_from(wire: ModelEvent) -> Result<Self, Self::Error> {
        let priority = wire.priority;
        let event_number = wire.event_number;
        let timestamp = wire.timestamp;
        let event = T::try_from(wire)?;
        Ok(Self {
            priority,
            event_number,
            timestamp,
            event,
        })
    }
}

impl<T> Deref for WireEvent<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<T> DerefMut for WireEvent<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

#[cfg(all(feature = "dispatch", not(feature = "serde")))]
impl<T> MatterDescribe for WireEvent<T>
where
    T: MatterDescribe,
{
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema {
        is_empty: false,
        ..T::STATIC_SCHEMA
    };

    #[inline]
    fn descriptor(&self) -> EntityDescriptor {
        self.event.descriptor()
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        false
    }
}

#[cfg(all(feature = "dispatch", feature = "serde"))]
impl<T> MatterDescribe for WireEvent<T>
where
    T: MatterDescribe + DeserializeOwned,
{
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema {
        is_empty: false,
        ..T::STATIC_SCHEMA
    };

    #[inline]
    fn descriptor(&self) -> EntityDescriptor {
        self.event.descriptor()
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        false
    }

    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let (Some((cluster, cluster_id)), Some((event, event_id))) = (T::STATIC_SCHEMA.scope, T::STATIC_SCHEMA.item)
        {
            deserialize_concrete_described(
                deserializer,
                EntityDescriptor::Event {
                    cluster,
                    cluster_id,
                    event,
                    event_id,
                },
                false,
            )
        } else {
            super::deserialize_envelope_described(deserializer)
        }
    }
}
