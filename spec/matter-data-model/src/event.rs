//! Matter Event entity per CSA Matter Specification Section 7.10.

use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{cluster::ClusterId, optional::Optional, status::MatterStatus, value::AttributeValue};

define_matter_id! {
    /// Strongly-typed Matter Event identifier (`u32`).
    pub struct EventId(u32);
    display = hex_32_16_or_8;
    mei = true;
}

impl EventId {
    /// Wildcard Event ID (`0xFFFF_FFFF`) per CSA Matter Interaction Model §10.5.
    pub const WILDCARD: Self = Self(0xFFFF_FFFF);

    /// Return `true` if this is the Wildcard event (`0xFFFF_FFFF`).
    #[must_use]
    pub const fn is_wildcard(self) -> bool {
        self.0 == 0xFFFF_FFFF
    }

    /// Return `true` if this event ID lies within CSA Matter Core Specification §7.10 allocation ranges
    /// (`sub_id` in `0x0000..=0x00FF` with standard or valid manufacturer vendor prefix).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        (self.is_standard() || self.vendor_id().is_valid()) && self.sub_id() <= 0x00FF
    }
}

define_matter_id! {
    /// Strongly-typed Matter `EventNumber` identifier (`u64`).
    pub struct EventNumber(u64);
    display = decimal;
    mei = false;
}

/// Priority level for Matter events per the Interaction Model specification Section 7.10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum EventPriority {
    /// Diagnostic / debug-level event.
    Debug = 0,
    /// Informational event.
    #[default]
    Info = 1,
    /// Critical event requiring priority retention and delivery.
    Critical = 2,
}

impl From<EventPriority> for u8 {
    fn from(priority: EventPriority) -> Self {
        match priority {
            EventPriority::Debug => 0,
            EventPriority::Info => 1,
            EventPriority::Critical => 2,
        }
    }
}

impl TryFrom<u8> for EventPriority {
    type Error = MatterStatus;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::Debug),
            1 => Ok(Self::Info),
            2 => Ok(Self::Critical),
            _ => Err(MatterStatus::ConstraintError),
        }
    }
}

impl Display for EventPriority {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Debug => write!(f, "Debug"),
            Self::Info => write!(f, "Info"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

impl From<EventPriority> for AttributeValue {
    fn from(priority: EventPriority) -> Self {
        Self::U8(u8::from(priority))
    }
}

impl TryFrom<&AttributeValue> for EventPriority {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::U8(v) => Self::try_from(*v),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<AttributeValue> for EventPriority {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        (&val).try_into()
    }
}

/// Wire timestamp for Matter events per CSA Matter Interaction Model §10.6.9 (`EventDataIB`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(
    all(not(feature = "serde"), feature = "schemars"),
    schemars(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum EventTimestamp {
    /// Epoch timestamp in milliseconds (`epoch-ms`).
    EpochMs(u64),
    /// System timestamp in milliseconds since boot (`systime-ms`).
    SystemMs(u64),
}

impl Display for EventTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::EpochMs(ms) => write!(f, "epoch:{ms}ms"),
            Self::SystemMs(ms) => write!(f, "sys:{ms}ms"),
        }
    }
}

/// Wire representation of a Matter Event record per Section 7.10.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct Event {
    /// Owning Matter cluster identifier.
    pub cluster_id: ClusterId,
    /// Matter event identifier within the cluster.
    pub event_id: EventId,
    /// Interaction Model priority level of the event.
    pub priority: EventPriority,
    /// Optional monotonic event sequence number assigned by the node.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub event_number: Optional<EventNumber>,
    /// Optional event timestamp (`epoch-ms` or `systime-ms`) per Interaction Model §10.6.9.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub timestamp: Optional<EventTimestamp>,
    /// Wire payload of the event.
    pub payload: AttributeValue,
}

impl Event {
    /// Construct a new event with omitted event number and timestamp.
    #[must_use]
    pub const fn new(
        cluster_id: ClusterId,
        event_id: EventId,
        priority: EventPriority,
        payload: AttributeValue,
    ) -> Self {
        Self {
            cluster_id,
            event_id,
            priority,
            event_number: Optional::Omitted,
            timestamp: Optional::Omitted,
            payload,
        }
    }

    /// Set the monotonic event number.
    #[must_use]
    pub const fn with_event_number(mut self, event_number: EventNumber) -> Self {
        self.event_number = Optional::Present(event_number);
        self
    }

    /// Set or omit the optional monotonic event number.
    #[must_use]
    pub fn with_optional_event_number(mut self, event_number: impl Into<Optional<EventNumber>>) -> Self {
        self.event_number = event_number.into();
        self
    }

    /// Set the event timestamp.
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp: EventTimestamp) -> Self {
        self.timestamp = Optional::Present(timestamp);
        self
    }

    /// Set or omit the optional event timestamp.
    #[must_use]
    pub fn with_optional_timestamp(mut self, timestamp: impl Into<Optional<EventTimestamp>>) -> Self {
        self.timestamp = timestamp.into();
        self
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}/{} [{}]", self.cluster_id, self.event_id, self.priority)?;
        if let Optional::Present(num) = self.event_number {
            write!(f, " #{num}")?;
        }
        write!(f, " ({})", self.payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VendorId;

    #[test]
    fn event_id_and_construction() {
        assert_eq!(EventId::WILDCARD.raw(), 0xFFFF_FFFF);

        let eid = EventId::new(0x02);
        assert_eq!(eid.raw(), 0x02);
        assert_eq!(alloc::format!("{eid}"), "0x02");
        assert_eq!(u32::from(eid), 0x02);

        let ev_num = EventNumber::new(100);
        assert_eq!(ev_num.raw(), 100);
        assert_eq!(alloc::format!("{ev_num}"), "100");
        assert_eq!(u64::from(ev_num), 100);

        const ON_OFF_CLUSTER_ID: ClusterId = ClusterId::new(0x0006);

        let event = Event::new(ON_OFF_CLUSTER_ID, eid, EventPriority::Info, AttributeValue::Bool(true))
            .with_event_number(ev_num);

        assert_eq!(event.cluster_id, ON_OFF_CLUSTER_ID);
        assert_eq!(event.event_id, eid);
        assert_eq!(event.priority, EventPriority::Info);
        assert_eq!(event.event_number, Optional::Present(ev_num));
        assert_eq!(event.timestamp, Optional::Omitted);
        assert_eq!(event.payload, AttributeValue::Bool(true));

        let ts_event = event.clone().with_timestamp(EventTimestamp::EpochMs(1_000_000));
        assert_eq!(
            ts_event.timestamp,
            Optional::Present(EventTimestamp::EpochMs(1_000_000))
        );
        assert_eq!(alloc::format!("{}", EventTimestamp::SystemMs(42)), "sys:42ms");

        let val_eid = AttributeValue::from(eid);
        assert_eq!(EventId::try_from(&val_eid), Ok(eid));
        assert_eq!(EventId::try_from(val_eid), Ok(eid));

        let val_ev_num = AttributeValue::from(ev_num);
        assert_eq!(EventNumber::try_from(&val_ev_num), Ok(ev_num));
        assert_eq!(EventNumber::try_from(val_ev_num), Ok(ev_num));

        let prio_val = AttributeValue::from(EventPriority::Critical);
        assert_eq!(EventPriority::try_from(&prio_val), Ok(EventPriority::Critical));
        assert_eq!(EventPriority::try_from(prio_val), Ok(EventPriority::Critical));
        assert_eq!(alloc::format!("{event}"), "0x0006/0x02 [Info] #100 (true)");
        assert_eq!("0x02".parse::<EventId>(), Ok(eid));
        assert_eq!("100".parse::<EventNumber>(), Ok(ev_num));
        assert!(eid.is_valid());
        assert!(EventId::from_mei(VendorId::new(0xFFF1), 0x00FF).is_valid());
        assert!(!EventId::new(0x0100).is_valid());
        assert!(!EventId::WILDCARD.is_valid());
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn event_json_schema_marks_optional_fields() {
        let mut generator = schemars::SchemaGenerator::default();
        let schema = <Event as schemars::JsonSchema>::json_schema(&mut generator);
        let required = schema
            .as_object()
            .and_then(|o| o.get("required"))
            .and_then(|v| v.as_array());
        assert!(
            required.is_some_and(|req| {
                !req.iter().any(|v| v.as_str() == Some("event_number"))
                    && !req.iter().any(|v| v.as_str() == Some("timestamp"))
            }),
            "event_number and timestamp must not be required in Event JSON schema"
        );
    }
}
