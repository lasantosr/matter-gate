//! Matter `ipadr` semantic type (IPv4 or IPv6 address).

use core::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::Deref,
};

use super::{Ipv4Address, Ipv6Address};
use crate::{
    Bytes,
    model::{AttributeValue, MatterStatus},
};

/// IP address (`ipadr`): 4-byte IPv4 or 16-byte IPv6 address (wire type `Bytes`).
///
/// `TryFrom<&[u8]>` enforces that the slice length is
/// `4` or `16` bytes, returning [`MatterStatus::ConstraintError`] otherwise.
/// [`IpAddress::from_raw`] wraps a [`Bytes`] without length validation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(transparent))]
#[repr(transparent)]
pub struct IpAddress(#[cfg_attr(feature = "schemars", schemars(with = "alloc::vec::Vec<u8>"))] Bytes);

impl Default for IpAddress {
    /// Returns the unspecified IPv4 address (`0.0.0.0`).
    #[inline]
    fn default() -> Self {
        Self(Bytes::from_static(&[0, 0, 0, 0]))
    }
}

impl IpAddress {
    /// Construct from a raw byte buffer without length validation.
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: Bytes) -> Self {
        Self(raw)
    }

    /// Construct from a byte buffer, validating that the length is `4` (IPv4) or `16` (IPv6) bytes.
    #[inline]
    pub fn from_bytes(bytes: Bytes) -> Result<Self, MatterStatus> {
        if bytes.len() == 4 || bytes.len() == 16 {
            Ok(Self(bytes))
        } else {
            Err(MatterStatus::ConstraintError)
        }
    }

    /// Borrow the underlying IP address bytes.
    #[inline]
    #[must_use]
    pub fn raw(&self) -> &[u8] {
        &self.0
    }

    /// Consume and return the underlying byte buffer.
    #[inline]
    #[must_use]
    pub fn into_raw(self) -> Bytes {
        self.0
    }

    /// Return the address as an [`Ipv4Address`] if it is 4 bytes long.
    #[inline]
    #[must_use]
    pub fn as_ipv4(&self) -> Option<Ipv4Address> {
        let octets: [u8; 4] = self.raw().try_into().ok()?;
        Some(Ipv4Address::from_raw(octets))
    }

    /// Return the address as an [`Ipv6Address`] if it is 16 bytes long.
    #[inline]
    #[must_use]
    pub fn as_ipv6(&self) -> Option<Ipv6Address> {
        let octets: [u8; 16] = self.raw().try_into().ok()?;
        Some(Ipv6Address::from_raw(octets))
    }

    /// Convert to [`core::net::IpAddr`] if the address is 4 or 16 bytes long.
    #[inline]
    #[must_use]
    pub fn to_ip_addr(&self) -> Option<IpAddr> {
        if let Some(v4) = self.as_ipv4() {
            Some(IpAddr::V4(Ipv4Addr::from(v4)))
        } else {
            self.as_ipv6().map(|v6| IpAddr::V6(Ipv6Addr::from(v6)))
        }
    }
}

impl AsRef<[u8]> for IpAddress {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Ipv4Address> for IpAddress {
    #[inline]
    fn from(val: Ipv4Address) -> Self {
        Self(Bytes::copy_from_slice(&val.raw()))
    }
}

impl From<Ipv6Address> for IpAddress {
    #[inline]
    fn from(val: Ipv6Address) -> Self {
        Self(Bytes::copy_from_slice(&val.raw()))
    }
}

impl From<[u8; 4]> for IpAddress {
    #[inline]
    fn from(octets: [u8; 4]) -> Self {
        Self(Bytes::copy_from_slice(&octets))
    }
}

impl From<[u8; 16]> for IpAddress {
    #[inline]
    fn from(octets: [u8; 16]) -> Self {
        Self(Bytes::copy_from_slice(&octets))
    }
}

impl From<Ipv4Addr> for IpAddress {
    #[inline]
    fn from(addr: Ipv4Addr) -> Self {
        Self::from(addr.octets())
    }
}

impl From<Ipv6Addr> for IpAddress {
    #[inline]
    fn from(addr: Ipv6Addr) -> Self {
        Self::from(addr.octets())
    }
}

impl From<IpAddr> for IpAddress {
    #[inline]
    fn from(addr: IpAddr) -> Self {
        match addr {
            IpAddr::V4(v4) => Self::from(v4),
            IpAddr::V6(v6) => Self::from(v6),
        }
    }
}

impl TryFrom<IpAddress> for IpAddr {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: IpAddress) -> Result<Self, Self::Error> {
        val.to_ip_addr().ok_or(MatterStatus::ConstraintError)
    }
}

impl TryFrom<&[u8]> for IpAddress {
    type Error = MatterStatus;

    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() == 4 || bytes.len() == 16 {
            Ok(Self(Bytes::copy_from_slice(bytes)))
        } else {
            Err(MatterStatus::ConstraintError)
        }
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ip) = self.to_ip_addr() {
            return write!(f, "{ip}");
        }
        for (i, b) in self.raw().iter().enumerate() {
            if i > 0 {
                write!(f, ":")?;
            }
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

impl Deref for IpAddress {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<IpAddress> for AttributeValue {
    #[inline]
    fn from(val: IpAddress) -> Self {
        Self::Bytes(val.0)
    }
}

impl TryFrom<AttributeValue> for IpAddress {
    type Error = MatterStatus;

    #[inline]
    fn try_from(val: AttributeValue) -> Result<Self, Self::Error> {
        match val {
            AttributeValue::Bytes(bytes) => Self::from_bytes(bytes),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;

    #[test]
    fn ip_address_v4_v6_conversions_and_display() {
        let v4 = IpAddress::from(Ipv4Address::from_raw([10, 0, 0, 1]));
        assert_eq!(v4.to_string(), "10.0.0.1");
        assert_eq!(v4.as_ipv4(), Some(Ipv4Address::from_raw([10, 0, 0, 1])));
        assert_eq!(v4.as_ipv6(), None);
        assert_eq!(v4.to_ip_addr(), Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert_eq!(IpAddr::try_from(v4.clone()), Ok(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));

        let v6_octets = [0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let v6 = IpAddress::from(v6_octets);
        assert_eq!(v6.to_string(), "2001:db8::1");
        assert_eq!(v6.as_ipv4(), None);
        assert_eq!(v6.as_ipv6(), Some(Ipv6Address::from_raw(v6_octets)));
        assert_eq!(IpAddress::from(IpAddr::V6(Ipv6Addr::from(v6_octets))), v6);

        let av = AttributeValue::from(v4.clone());
        assert_eq!(IpAddress::try_from(av), Ok(v4.clone()));
        assert_eq!(&*v4, &[10, 0, 0, 1]);
        assert_eq!(IpAddress::try_from(&[1, 2, 3][..]), Err(MatterStatus::ConstraintError));
        assert_eq!(
            IpAddress::try_from(AttributeValue::U8(1)),
            Err(MatterStatus::InvalidDataType)
        );

        let raw = IpAddress::from_raw(Bytes::copy_from_slice(&[0xaa, 0xbb]));
        assert_eq!(raw.to_string(), "aa:bb");
        assert_eq!(IpAddr::try_from(raw), Err(MatterStatus::ConstraintError));
        assert_eq!(IpAddress::from_raw(Bytes::copy_from_slice(&[])).into_raw().len(), 0);

        let default_ip = IpAddress::default();
        assert_eq!(default_ip.to_string(), "0.0.0.0");
        assert_eq!(default_ip.to_ip_addr(), Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)));
    }
}
