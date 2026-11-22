//! Interaction Model status codes per CSA Matter Specification Section 8.10.

use core::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

use crate::value::AttributeValue;

macro_rules! matter_status {
    ($($(#[$meta:meta])* $name:ident = $code:literal),+ $(,)?) => {
        /// Normative Matter Interaction Model Status Codes (`StatusCode`) per Section 8.10.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum MatterStatus {
            $($(#[$meta])* $name = $code),+
        }

        impl MatterStatus {
            /// Every defined status, in declaration order.
            ///
            /// [`MatterStatus`] is `#[non_exhaustive]`, so new variants may be added in future versions.
            pub const ALL: &'static [Self] = &[$(Self::$name),+];

            /// Return the raw wire `u8` status code.
            #[must_use]
            pub const fn code(self) -> u8 {
                match self {
                    $(Self::$name => $code),+
                }
            }

            /// Return `true` if this status is [`MatterStatus::Success`].
            #[must_use]
            pub const fn is_success(self) -> bool {
                matches!(self, Self::Success)
            }

            /// Return `true` if this status is any non-success error code.
            #[must_use]
            pub const fn is_error(self) -> bool {
                !self.is_success()
            }
        }

        impl From<MatterStatus> for u8 {
            fn from(status: MatterStatus) -> Self {
                status.code()
            }
        }

        /// Error returned when a raw wire byte does not correspond to a known [`MatterStatus`].
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct UnknownStatusCode(u8);

        impl UnknownStatusCode {
            /// Return the unrecognized raw wire status byte.
            #[must_use]
            pub const fn code(self) -> u8 {
                self.0
            }
        }

        impl core::fmt::Display for UnknownStatusCode {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "unknown Matter status code: {:#04x}", self.0)
            }
        }

        impl core::error::Error for UnknownStatusCode {}

        impl From<UnknownStatusCode> for u8 {
            fn from(err: UnknownStatusCode) -> Self {
                err.0
            }
        }

        impl From<UnknownStatusCode> for MatterStatus {
            fn from(_: UnknownStatusCode) -> Self {
                Self::Failure
            }
        }

        impl TryFrom<u8> for MatterStatus {
            type Error = UnknownStatusCode;

            fn try_from(code: u8) -> Result<Self, Self::Error> {
                match code {
                    $($code => Ok(Self::$name),)+
                    _ => Err(UnknownStatusCode(code)),
                }
            }
        }
    };
}

matter_status! {
    /// Operation completed successfully (`0x00`).
    Success = 0x00,
    /// Unspecified general failure (`0x01`).
    Failure = 0x01,
    /// Subscription ID is not known or active (`0x7D`).
    InvalidSubscription = 0x7D,
    /// Sender lacks required ACL privilege for the target (`0x7E`).
    UnsupportedAccess = 0x7E,
    /// Target endpoint does not exist on the node (`0x7F`).
    UnsupportedEndpoint = 0x7F,
    /// Requested action is malformed or invalid in the current context (`0x80`).
    InvalidAction = 0x80,
    /// Target command is not supported on the cluster (`0x81`).
    UnsupportedCommand = 0x81,
    /// Command payload or arguments are invalid (`0x85`).
    InvalidCommand = 0x85,
    /// Target attribute is not supported on the cluster (`0x86`).
    UnsupportedAttribute = 0x86,
    /// Value violates range, length, or schema constraints (`0x87`).
    ConstraintError = 0x87,
    /// Attribute is read-only or cannot be written (`0x88`).
    UnsupportedWrite = 0x88,
    /// Insufficient resources to complete the request (`0x89`).
    ResourceExhausted = 0x89,
    /// Requested entry or resource was not found (`0x8B`).
    NotFound = 0x8B,
    /// Attribute does not support reporting (`0x8C`).
    UnreportableAttribute = 0x8C,
    /// Wire value data type does not match the expected type (`0x8D`).
    InvalidDataType = 0x8D,
    /// Attribute is write-only or cannot be read (`0x8F`).
    UnsupportedRead = 0x8F,
    /// Supplied data version does not match the cluster's current data version (`0x92`).
    DataVersionMismatch = 0x92,
    /// Operation timed out (`0x94`).
    Timeout = 0x94,
    /// Server is temporarily busy and cannot process the request (`0x9C`).
    Busy = 0x9C,
    /// Access to the target is restricted by Access Control (`0x9D`).
    AccessRestricted = 0x9D,
    /// Target cluster is not supported on the endpoint (`0xC3`).
    UnsupportedCluster = 0xC3,
    /// Upstream subscription does not exist (`0xC5`).
    NoUpstreamSubscription = 0xC5,
    /// Action requires a preceding Timed Request (`0xC6`).
    NeedsTimedInteraction = 0xC6,
    /// Target event is not supported on the cluster (`0xC7`).
    UnsupportedEvent = 0xC7,
    /// Maximum number of interaction paths exceeded (`0xC8`).
    PathsExhausted = 0xC8,
    /// Timed interaction state does not match the request (`0xC9`).
    TimedRequestMismatch = 0xC9,
    /// Action requires an armed fail-safe timer (`0xCA`).
    FailsafeRequired = 0xCA,
    /// Action is invalid in the current device or cluster state (`0xCB`).
    InvalidInState = 0xCB,
    /// No command response was generated (`0xCC`).
    NoCommandResponse = 0xCC,
    /// Write was silently ignored (`0xF0`).
    WriteIgnored = 0xF0,
}

impl Display for MatterStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{self:?} ({:#04x})", self.code())
    }
}

impl Error for MatterStatus {}

impl From<MatterStatus> for AttributeValue {
    fn from(status: MatterStatus) -> Self {
        Self::U8(status.code())
    }
}

impl TryFrom<&AttributeValue> for MatterStatus {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::U8(code) => Self::try_from(*code).map_err(|_| MatterStatus::ConstraintError),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<AttributeValue> for MatterStatus {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        (&val).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matter_status_codes() {
        assert_eq!(MatterStatus::Success.code(), 0x00);
        assert_eq!(MatterStatus::InvalidSubscription.code(), 0x7D);
        assert_eq!(MatterStatus::UnsupportedEvent.code(), 0xC7);
        assert_eq!(MatterStatus::WriteIgnored.code(), 0xF0);
        assert_eq!(MatterStatus::ALL.len(), 30);
    }

    #[test]
    fn matter_status_roundtrip_all() {
        for &s in MatterStatus::ALL {
            assert_eq!(MatterStatus::try_from(s.code()), Ok(s));
        }
        assert_eq!(MatterStatus::try_from(0x02).map_err(UnknownStatusCode::code), Err(0x02));
        assert_eq!(MatterStatus::try_from(0xFE).map_err(UnknownStatusCode::code), Err(0xFE));
        assert_eq!(
            MatterStatus::from(MatterStatus::try_from(0xFE).unwrap_err()),
            MatterStatus::Failure
        );

        assert!(MatterStatus::Success < MatterStatus::Failure);
        let status_val = AttributeValue::from(MatterStatus::NotFound);
        assert_eq!(status_val, AttributeValue::U8(0x8B));
        assert_eq!(MatterStatus::try_from(&status_val), Ok(MatterStatus::NotFound));
        assert_eq!(MatterStatus::try_from(status_val), Ok(MatterStatus::NotFound));
        assert_eq!(
            MatterStatus::try_from(&AttributeValue::U8(0xFE)),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(
            MatterStatus::try_from(&AttributeValue::Bool(true)),
            Err(MatterStatus::InvalidDataType)
        );
    }
}
