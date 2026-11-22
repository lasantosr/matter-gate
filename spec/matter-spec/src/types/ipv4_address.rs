//! Matter `ipv4adr` semantic type (4-byte IPv4 address).

use core::net::Ipv4Addr;

super::define_matter_ip_addr_type! {
    /// IPv4 address (`ipv4adr`, 4 octets in network byte order, wire type `[u8; 4]` / `octstr(4)`).
    ///
    /// `TryFrom<&[u8]>`, `TryFrom<AttributeValue>`, and `TryFrom<&AttributeValue>` enforce an exact length of `4` bytes,
    /// returning [`MatterStatus::ConstraintError`](crate::model::MatterStatus::ConstraintError) otherwise.
    /// Converts losslessly to and from [`core::net::Ipv4Addr`].
    ///
    /// ```rust
    /// use matter_spec::types::Ipv4Address;
    ///
    /// let addr = Ipv4Address::from_raw([192, 168, 1, 1]);
    /// assert_eq!(addr.to_string(), "192.168.1.1");
    /// ```
    pub struct Ipv4Address([u8; 4]);
    net_addr = Ipv4Addr;
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;
    use core::net::Ipv4Addr;

    use super::*;
    use crate::{
        Bytes,
        model::{AttributeValue, MatterStatus},
    };

    #[test]
    fn ipv4_round_trip_and_length_validation() {
        let addr = Ipv4Address::from_raw([192, 168, 1, 1]);
        assert_eq!(addr.to_string(), "192.168.1.1");
        assert_eq!(Ipv4Addr::from(addr), Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(Ipv4Address::from(Ipv4Addr::new(192, 168, 1, 1)), addr);
        let av = AttributeValue::from(addr);
        assert_eq!(Ipv4Address::try_from(&av), Ok(addr));
        assert_eq!(Ipv4Address::try_from(av), Ok(addr));

        assert_eq!(
            Ipv4Address::try_from(&AttributeValue::Bytes(Bytes::from(&[192_u8, 168, 1][..]))),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(
            Ipv4Address::try_from(&AttributeValue::Bytes(Bytes::from(&[192_u8, 168, 1, 1, 2][..]))),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(
            Ipv4Address::try_from(&AttributeValue::U32(0)),
            Err(MatterStatus::InvalidDataType)
        );
    }
}
