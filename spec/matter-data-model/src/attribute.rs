//! Matter Attribute entity per CSA Matter Specification Section 7.8.

use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::value::AttributeValue;

define_matter_id! {
    /// Strongly-typed Matter Attribute identifier (`u32`).
    pub struct AttributeId(u32);
    display = hex_32_or_16;
    mei = true;
}

impl AttributeId {
    /// Accepted Command List Global Attribute ID (`0xFFF9`) per CSA Matter Core Specification §7.8.
    pub const ACCEPTED_COMMAND_LIST: Self = Self(0xFFF9);
    /// Attribute List Global Attribute ID (`0xFFFB`) per CSA Matter Core Specification §7.8.
    pub const ATTRIBUTE_LIST: Self = Self(0xFFFB);
    /// Cluster Revision Global Attribute ID (`0xFFFD`) per CSA Matter Core Specification §7.8.
    pub const CLUSTER_REVISION: Self = Self(0xFFFD);
    /// Event List Global Attribute ID (`0xFFFA`) per CSA Matter Core Specification §7.8.
    pub const EVENT_LIST: Self = Self(0xFFFA);
    /// Feature Map Global Attribute ID (`0xFFFC`) per CSA Matter Core Specification §7.8.
    pub const FEATURE_MAP: Self = Self(0xFFFC);
    /// Generated Command List Global Attribute ID (`0xFFF8`) per CSA Matter Core Specification §7.8.
    pub const GENERATED_COMMAND_LIST: Self = Self(0xFFF8);
    /// Wildcard Attribute ID (`0xFFFF_FFFF`) per CSA Matter Interaction Model §10.5.
    pub const WILDCARD: Self = Self(0xFFFF_FFFF);

    /// Return `true` if this is a standard global attribute ID (`0xF000..=0xFFFE`).
    #[must_use]
    pub const fn is_global(self) -> bool {
        self.is_standard() && matches!(self.sub_id(), 0xF000..=0xFFFE)
    }

    /// Return `true` if this is the Wildcard attribute (`0xFFFF_FFFF`).
    #[must_use]
    pub const fn is_wildcard(self) -> bool {
        self.0 == 0xFFFF_FFFF
    }

    /// Return `true` if this is a cluster-specific attribute ID (`sub_id` in `0x0000..=0x4FFF`
    /// with standard or valid manufacturer vendor prefix) per CSA Matter Core Specification §7.8.
    #[must_use]
    pub const fn is_cluster_specific(self) -> bool {
        (self.is_standard() || self.vendor_id().is_valid()) && self.sub_id() <= 0x4FFF
    }

    /// Return `true` if this attribute ID lies within CSA Matter Core Specification §7.8 allocation ranges
    /// (cluster-specific `0x0000..=0x4FFF` or standard global `0xF000..=0xFFFE`).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.is_cluster_specific() || self.is_global()
    }
}

/// Wire representation of a Matter Attribute instance ([`AttributeId`] and [`AttributeValue`]) per Section 7.8.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct Attribute {
    /// Matter attribute identifier.
    pub id: AttributeId,
    /// Wire value of the attribute.
    pub value: AttributeValue,
}

impl Attribute {
    /// Construct a new wire attribute instance.
    #[must_use]
    pub const fn new(id: AttributeId, value: AttributeValue) -> Self {
        Self { id, value }
    }
}

impl<T: Into<AttributeValue>> From<(AttributeId, T)> for Attribute {
    fn from((id, value): (AttributeId, T)) -> Self {
        Self {
            id,
            value: value.into(),
        }
    }
}

impl From<Attribute> for (AttributeId, AttributeValue) {
    fn from(attr: Attribute) -> Self {
        (attr.id, attr.value)
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} = {}", self.id, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VendorId;

    #[test]
    fn attribute_id_and_attribute() {
        assert_eq!(AttributeId::CLUSTER_REVISION.raw(), 0xFFFD);
        assert_eq!(AttributeId::FEATURE_MAP.raw(), 0xFFFC);
        assert_eq!(AttributeId::ATTRIBUTE_LIST.raw(), 0xFFFB);
        assert_eq!(AttributeId::EVENT_LIST.raw(), 0xFFFA);
        assert_eq!(AttributeId::ACCEPTED_COMMAND_LIST.raw(), 0xFFF9);
        assert_eq!(AttributeId::GENERATED_COMMAND_LIST.raw(), 0xFFF8);
        assert_eq!(AttributeId::WILDCARD.raw(), 0xFFFF_FFFF);

        let attr_id = AttributeId::new(0x0011);
        assert_eq!(attr_id.raw(), 0x0011);
        assert_eq!(alloc::format!("{attr_id}"), "0x0011");
        assert_eq!(u32::from(attr_id), 0x0011);

        let attr = Attribute::new(attr_id, AttributeValue::Bool(true));
        assert_eq!(attr.id, attr_id);
        assert_eq!(attr.value, AttributeValue::Bool(true));

        let tuple: (AttributeId, AttributeValue) = attr.clone().into();
        assert_eq!(tuple, (attr_id, AttributeValue::Bool(true)));
        assert_eq!(Attribute::from(tuple), attr);

        let val = AttributeValue::from(attr_id);
        assert_eq!(AttributeId::try_from(&val), Ok(attr_id));
        assert_eq!(AttributeId::try_from(val), Ok(attr_id));

        assert_eq!(Attribute::from((attr_id, true)), attr);
        assert!(attr_id.is_cluster_specific());
        assert!(attr_id.is_valid());
        assert!(AttributeId::CLUSTER_REVISION.is_valid());
        assert!(!AttributeId::CLUSTER_REVISION.is_cluster_specific());
        assert!(!AttributeId::new(0x5000).is_valid());
        assert!(AttributeId::from_mei(VendorId::new(0xFFF1), 0x0001).is_valid());
        assert!(!AttributeId::from_mei(VendorId::new(0xFFF1), 0xFFFD).is_valid());
        assert!(!AttributeId::WILDCARD.is_valid());
    }
}
