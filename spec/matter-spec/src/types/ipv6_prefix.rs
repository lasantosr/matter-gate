//! Matter `ipv6pre` semantic type (IPv6 prefix).

use core::{fmt, net::Ipv6Addr, ops::Deref};

use crate::{
    Bytes,
    model::{AttributeValue, MatterStatus},
};

/// IPv6 prefix (`ipv6pre`, variable-length `Bytes`).
///
/// Per CSA Matter Core Specification §7.19.2.10, `ipv6pre` encodes a 1-octet prefix
/// length (`0..=128` bits) followed by the leading prefix octets (up to 16 bytes,
/// `1..=17` bytes total on the wire). Construct explicitly with [`Ipv6Prefix::from_raw`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(transparent))]
#[repr(transparent)]
pub struct Ipv6Prefix(#[cfg_attr(feature = "schemars", schemars(with = "alloc::vec::Vec<u8>"))] Bytes);

impl Default for Ipv6Prefix {
    /// Returns the zero-length IPv6 prefix (`::/0`).
    #[inline]
    fn default() -> Self {
        Self(Bytes::from_static(&[0]))
    }
}

impl Ipv6Prefix {
    /// Construct from a raw byte buffer.
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: Bytes) -> Self {
        Self(raw)
    }

    /// Construct from a byte buffer, validating the prefix length (`0..=128`) and octet count (`1..=17` total).
    #[inline]
    pub fn from_bytes(bytes: Bytes) -> Result<Self, MatterStatus> {
        let Some((&prefix_len, prefix_octets)) = bytes.split_first() else {
            return Err(MatterStatus::ConstraintError);
        };
        if prefix_len > 128 || prefix_octets.len() > 16 || prefix_octets.len() < usize::from(prefix_len.div_ceil(8)) {
            return Err(MatterStatus::ConstraintError);
        }
        Ok(Self(bytes))
    }

    /// Borrow the underlying prefix bytes.
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

    /// Return the prefix length in bits if the buffer is non-empty and `<= 128`.
    #[inline]
    #[must_use]
    pub fn prefix_len(&self) -> Option<u8> {
        let len = *self.raw().first()?;
        (len <= 128).then_some(len)
    }

    /// Borrow the prefix octets following the leading length byte (empty if raw buffer is empty).
    #[inline]
    #[must_use]
    pub fn prefix_bytes(&self) -> &[u8] {
        self.raw().get(1..).unwrap_or(&[])
    }
}

impl AsRef<[u8]> for Ipv6Prefix {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Ipv6Prefix {
    type Error = MatterStatus;

    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let Some((&prefix_len, prefix_octets)) = bytes.split_first() else {
            return Err(MatterStatus::ConstraintError);
        };
        if prefix_len > 128 || prefix_octets.len() > 16 || prefix_octets.len() < usize::from(prefix_len.div_ceil(8)) {
            return Err(MatterStatus::ConstraintError);
        }
        Ok(Self(Bytes::copy_from_slice(bytes)))
    }
}

impl fmt::Display for Ipv6Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prefix_len) = self.prefix_len() {
            let prefix_octets = self.prefix_bytes();
            if prefix_octets.len() <= 16 && prefix_octets.len() >= usize::from(prefix_len.div_ceil(8)) {
                let mut octets = [0_u8; 16];
                if let Some(dst) = octets.get_mut(..prefix_octets.len()) {
                    dst.copy_from_slice(prefix_octets);
                    return write!(f, "{}/{prefix_len}", Ipv6Addr::from(octets));
                }
            }
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

impl Deref for Ipv6Prefix {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Ipv6Prefix> for AttributeValue {
    #[inline]
    fn from(val: Ipv6Prefix) -> Self {
        Self::Bytes(val.0)
    }
}

impl TryFrom<AttributeValue> for Ipv6Prefix {
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
    fn ipv6_prefix_validation_and_display() {
        let prefix = Ipv6Prefix::try_from(&[64, 0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0][..]).unwrap();
        assert_eq!(prefix.prefix_len(), Some(64));
        assert_eq!(prefix.prefix_bytes(), &[0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0]);
        assert_eq!(prefix.to_string(), "2001:db8::/64");

        let av = AttributeValue::from(prefix.clone());
        assert_eq!(Ipv6Prefix::try_from(av), Ok(prefix.clone()));
        assert_eq!(&*prefix, &[64, 0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0]);
        assert_eq!(
            Ipv6Prefix::try_from(AttributeValue::U8(1)),
            Err(MatterStatus::InvalidDataType)
        );

        // Invalid lengths and prefix_len > 128
        assert_eq!(Ipv6Prefix::try_from(&[][..]), Err(MatterStatus::ConstraintError));
        assert_eq!(
            Ipv6Prefix::try_from(&[0_u8; 18][..]),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(
            Ipv6Prefix::try_from(&[129_u8, 0][..]),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(
            Ipv6Prefix::try_from(&[64_u8, 0x20, 0x01][..]),
            Err(MatterStatus::ConstraintError)
        );

        // Out-of-range raw buffer falls back to hex display
        let raw = Ipv6Prefix::from_raw(Bytes::copy_from_slice(&[129, 0xaa]));
        assert_eq!(raw.prefix_len(), None);
        assert_eq!(raw.to_string(), "81:aa");

        let default_prefix = Ipv6Prefix::default();
        assert_eq!(default_prefix.prefix_len(), Some(0));
        assert_eq!(default_prefix.to_string(), "::/0");
    }
}
