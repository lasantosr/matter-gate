//! Fabric, identity, and interaction context identifiers per CSA Matter Specification Chapters 2 & 4.

define_matter_id! {
    /// Strongly-typed Matter Vendor identifier (`u16`).
    pub struct VendorId(u16);
    display = hex;
    mei = false;
}

impl VendorId {
    /// Unspecified Vendor ID (`0x0000`) per CSA Matter Core Specification §2.5.2.
    pub const NOT_SPECIFIED: Self = Self(0);

    /// Return `true` if a non-zero Vendor ID is specified (`!= 0x0000`).
    #[must_use]
    pub const fn is_specified(self) -> bool {
        self.0 != 0
    }

    /// Return `true` if this is a CSA Test Vendor ID (`0xFFF1..=0xFFF4`) per §2.5.2.
    #[must_use]
    pub const fn is_test(self) -> bool {
        matches!(self.0, 0xFFF1..=0xFFF4)
    }

    /// Return `true` if this Vendor ID is in the reserved range (`0xFFF5..=0xFFFF`) per §2.5.2.
    #[must_use]
    pub const fn is_reserved(self) -> bool {
        self.0 >= 0xFFF5
    }

    /// Return `true` if this is a valid assigned Vendor ID (`0x0001..=0xFFF4`, manufacturer or test) per §2.5.2.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        matches!(self.0, 0x0001..=0xFFF4)
    }
}

define_matter_id! {
    /// Strongly-typed Matter Fabric Index (`u8`).
    pub struct FabricIndex(u8);
    display = decimal;
    mei = false;
}

impl FabricIndex {
    /// Unassigned Fabric Index (`0`) per CSA Matter Core Specification §2.5.4.
    /// Indicates absence of an assigned operational fabric.
    pub const NO_FABRIC: Self = Self(0);
    /// Alias for [`Self::NO_FABRIC`] (`0`).
    pub const UNDEFINED: Self = Self(0);

    /// Return `true` if this is a valid operational fabric index (`1..=254`).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        matches!(self.0, 1..=254)
    }
}

define_matter_id! {
    /// Strongly-typed Matter Fabric identifier (`u64`).
    pub struct FabricId(u64);
    display = hex;
    mei = false;
}

impl FabricId {
    /// Undefined / reserved Fabric ID (`0`) per CSA Matter Core Specification §2.5.3.
    pub const UNDEFINED: Self = Self(0);

    /// Return `true` if this is a valid non-zero Fabric ID (`!= 0`) per CSA Matter Core Specification §2.5.3.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

define_matter_id! {
    /// Strongly-typed Matter Group identifier (`u16`).
    pub struct GroupId(u16);
    display = hex;
    mei = false;
}

impl GroupId {
    /// All-Nodes universal group ID (`0xFFFF`) per CSA Matter Core Specification §2.5.6.1.
    pub const ALL_NODES: Self = Self(0xFFFF);
    /// All-Non-Sleepy-Nodes universal group ID (`0xFFFE`) per CSA Matter Core Specification §2.5.6.1.
    pub const ALL_NON_SLEEPY_NODES: Self = Self(0xFFFE);
    /// All-Proxies universal group ID (`0xFFFD`) per CSA Matter Core Specification §2.5.6.1.
    pub const ALL_PROXIES: Self = Self(0xFFFD);
    /// Undefined group identifier (`0x0000`).
    pub const UNDEFINED: Self = Self(0);

    /// Return `true` if this is the undefined Group ID (`0x0000`).
    #[must_use]
    pub const fn is_undefined(self) -> bool {
        self.0 == 0
    }

    /// Return `true` if this is an application group ID (`0x0001..=0xFEFF`).
    #[must_use]
    pub const fn is_application(self) -> bool {
        matches!(self.0, 0x0001..=0xFEFF)
    }

    /// Return `true` if this is a universal group ID (`0xFF00..=0xFFFF`).
    #[must_use]
    pub const fn is_universal(self) -> bool {
        self.0 >= 0xFF00
    }
}

define_matter_id! {
    /// Strongly-typed Matter Action identifier (`u8`).
    pub struct ActionId(u8);
    display = hex;
    mei = false;
}

define_matter_id! {
    /// Strongly-typed Matter Transaction identifier (`u32`).
    pub struct TransactionId(u32);
    display = hex;
    mei = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::AttributeValue;

    #[test]
    fn context_types_display_and_roundtrip() {
        assert_eq!(VendorId::NOT_SPECIFIED.raw(), 0);
        assert!(!VendorId::NOT_SPECIFIED.is_specified());
        assert!(!VendorId::NOT_SPECIFIED.is_valid());
        assert!(VendorId::new(0x0001).is_specified());
        assert!(VendorId::new(0x0001).is_valid());
        assert!(VendorId::new(0xFFF1).is_test());
        assert!(VendorId::new(0xFFF1).is_valid());
        assert!(!VendorId::new(0x0001).is_test());
        assert!(VendorId::new(0xFFF5).is_reserved());
        assert!(!VendorId::new(0xFFF5).is_valid());
        assert_eq!(FabricIndex::NO_FABRIC.raw(), 0);
        assert!(!FabricIndex::NO_FABRIC.is_valid());
        assert!(FabricIndex::new(1).is_valid());
        assert!(FabricIndex::new(254).is_valid());
        assert!(!FabricIndex::new(255).is_valid());

        assert_eq!(VendorId::new(0xFFF1).raw(), 0xFFF1);
        assert_eq!(alloc::format!("{}", VendorId::new(0xFFF1)), "0xfff1");
        assert_eq!(u16::from(VendorId::new(0xFFF1)), 0xFFF1);
        let vendor_val = AttributeValue::from(VendorId::new(0xFFF1));
        assert_eq!(vendor_val, AttributeValue::U16(0xFFF1));
        assert_eq!(VendorId::try_from(&vendor_val), Ok(VendorId::new(0xFFF1)));
        assert_eq!(VendorId::try_from(vendor_val), Ok(VendorId::new(0xFFF1)));

        assert_eq!(FabricIndex::new(1).raw(), 1);
        assert_eq!(alloc::format!("{}", FabricIndex::new(1)), "1");
        assert_eq!(u8::from(FabricIndex::new(1)), 1);
        let fabric_idx_val = AttributeValue::from(FabricIndex::new(1));
        assert_eq!(fabric_idx_val, AttributeValue::U8(1));
        assert_eq!(FabricIndex::try_from(&fabric_idx_val), Ok(FabricIndex::new(1)));

        assert_eq!(FabricId::UNDEFINED.raw(), 0);
        assert!(!FabricId::UNDEFINED.is_valid());
        assert!(FabricId::new(1).is_valid());
        assert_eq!(FabricId::new(0x100).raw(), 0x100);
        assert_eq!(alloc::format!("{}", FabricId::new(1)), "0x0000000000000001");
        assert_eq!(u64::from(FabricId::new(100)), 100);
        let fabric_id_val = AttributeValue::from(FabricId::new(888));
        assert_eq!(fabric_id_val, AttributeValue::U64(888));
        assert_eq!(FabricId::try_from(&fabric_id_val), Ok(FabricId::new(888)));

        assert!(GroupId::UNDEFINED.is_undefined());
        assert!(!GroupId::new(0x0001).is_undefined());
        assert!(GroupId::new(0x0001).is_application());
        assert_eq!(GroupId::new(0x0001).raw(), 0x0001);
        assert!(GroupId::ALL_NODES.is_universal());
        assert!(GroupId::ALL_NON_SLEEPY_NODES.is_universal());
        assert!(GroupId::ALL_PROXIES.is_universal());
        assert_eq!(alloc::format!("{}", GroupId::new(1)), "0x0001");
        assert_eq!(u16::from(GroupId::new(1)), 1);
        let group_val = AttributeValue::from(GroupId::new(10));
        assert_eq!(group_val, AttributeValue::U16(10));
        assert_eq!(GroupId::try_from(&group_val), Ok(GroupId::new(10)));

        assert_eq!(ActionId::new(5).raw(), 5);
        assert_eq!(alloc::format!("{}", ActionId::new(5)), "0x05");
        assert_eq!(u8::from(ActionId::new(5)), 5);
        let action_val = AttributeValue::from(ActionId::new(1));
        assert_eq!(action_val, AttributeValue::U8(1));
        assert_eq!(ActionId::try_from(&action_val), Ok(ActionId::new(1)));

        assert_eq!(TransactionId::new(0x1234).raw(), 0x1234);
        assert_eq!(alloc::format!("{}", TransactionId::new(1)), "0x00000001");
        assert_eq!(u32::from(TransactionId::new(1)), 1);
        let tx_val = AttributeValue::from(TransactionId::new(777));
        assert_eq!(tx_val, AttributeValue::U32(777));
        assert_eq!(TransactionId::try_from(&tx_val), Ok(TransactionId::new(777)));
    }
}
