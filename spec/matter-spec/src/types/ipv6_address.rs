//! Matter `ipv6adr` semantic type (16-byte IPv6 address).

use core::net::Ipv6Addr;

super::define_matter_ip_addr_type! {
    /// IPv6 address (`ipv6adr`, 16 octets in network byte order, wire type `[u8; 16]` / `octstr(16)`).
    ///
    /// `TryFrom<&[u8]>`, `TryFrom<AttributeValue>`, and `TryFrom<&AttributeValue>` enforce an exact length of `16` bytes,
    /// returning [`MatterStatus::ConstraintError`](crate::model::MatterStatus::ConstraintError) otherwise.
    /// Converts losslessly to and from [`core::net::Ipv6Addr`].
    ///
    /// ```rust
    /// use core::net::Ipv6Addr;
    ///
    /// use matter_spec::types::Ipv6Address;
    ///
    /// let addr = Ipv6Address::from(Ipv6Addr::LOCALHOST);
    /// assert_eq!(addr.to_string(), "::1");
    /// ```
    pub struct Ipv6Address([u8; 16]);
    net_addr = Ipv6Addr;
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;
    use core::net::Ipv6Addr;

    use super::*;
    use crate::{
        Bytes,
        model::{AttributeValue, MatterStatus},
    };

    #[test]
    fn ipv6_round_trip_and_length_validation() {
        let addr = Ipv6Address::from(Ipv6Addr::LOCALHOST);
        assert_eq!(addr.to_string(), "::1");
        assert_eq!(Ipv6Addr::from(addr), Ipv6Addr::LOCALHOST);
        let av = AttributeValue::from(addr);
        assert_eq!(Ipv6Address::try_from(&av), Ok(addr));
        assert_eq!(Ipv6Address::try_from(av), Ok(addr));

        assert_eq!(
            Ipv6Address::try_from(&AttributeValue::Bytes(Bytes::from(&[0_u8; 4][..]))),
            Err(MatterStatus::ConstraintError)
        );
    }
}
