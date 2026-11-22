//! Matter `hwadr` semantic type (EUI-48 or EUI-64 hardware address).

use core::{fmt, ops::Deref};

use crate::{
    Bytes,
    model::{AttributeValue, MatterStatus},
};

/// Hardware address (`hwadr`): 6-byte EUI-48 or 8-byte EUI-64 (wire type `Bytes`).
///
/// `TryFrom<&[u8]>` enforces that the slice length is
/// `6` or `8` bytes, returning [`MatterStatus::ConstraintError`] otherwise.
/// [`HardwareAddress::from_raw`] wraps a [`Bytes`] without length validation.
///
/// ```rust
/// use matter_spec::types::HardwareAddress;
///
/// let mac = HardwareAddress::try_from(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff][..]).unwrap();
/// assert_eq!(mac.to_string(), "aa:bb:cc:dd:ee:ff");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(transparent))]
#[repr(transparent)]
pub struct HardwareAddress(#[cfg_attr(feature = "schemars", schemars(with = "alloc::vec::Vec<u8>"))] Bytes);

impl Default for HardwareAddress {
    /// Returns the all-zero 6-byte EUI-48 hardware address (`00:00:00:00:00:00`).
    #[inline]
    fn default() -> Self {
        Self(Bytes::from_static(&[0; 6]))
    }
}

impl HardwareAddress {
    /// Construct from a raw byte buffer without length validation.
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: Bytes) -> Self {
        Self(raw)
    }

    /// Construct from a byte buffer, validating that the length is `6` (EUI-48) or `8` (EUI-64) bytes.
    #[inline]
    pub fn from_bytes(bytes: Bytes) -> Result<Self, MatterStatus> {
        if bytes.len() == 6 || bytes.len() == 8 {
            Ok(Self(bytes))
        } else {
            Err(MatterStatus::ConstraintError)
        }
    }

    /// Borrow the underlying hardware address bytes.
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

    /// Return `true` if the address is a 6-byte EUI-48 address.
    #[inline]
    #[must_use]
    pub fn is_eui48(&self) -> bool {
        self.raw().len() == 6
    }

    /// Return `true` if the address is an 8-byte EUI-64 address.
    #[inline]
    #[must_use]
    pub fn is_eui64(&self) -> bool {
        self.raw().len() == 8
    }
}

impl AsRef<[u8]> for HardwareAddress {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 6]> for HardwareAddress {
    #[inline]
    fn from(bytes: [u8; 6]) -> Self {
        Self(Bytes::copy_from_slice(&bytes))
    }
}

impl From<[u8; 8]> for HardwareAddress {
    #[inline]
    fn from(bytes: [u8; 8]) -> Self {
        Self(Bytes::copy_from_slice(&bytes))
    }
}

impl TryFrom<&[u8]> for HardwareAddress {
    type Error = MatterStatus;

    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() == 6 || bytes.len() == 8 {
            Ok(Self(Bytes::copy_from_slice(bytes)))
        } else {
            Err(MatterStatus::ConstraintError)
        }
    }
}

impl fmt::Display for HardwareAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, b) in self.raw().iter().enumerate() {
            if i > 0 {
                write!(f, ":")?;
            }
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

impl Deref for HardwareAddress {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<HardwareAddress> for AttributeValue {
    #[inline]
    fn from(val: HardwareAddress) -> Self {
        Self::Bytes(val.0)
    }
}

impl TryFrom<AttributeValue> for HardwareAddress {
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
    use crate::model::Nullable;

    #[test]
    fn hardware_address_validation() {
        let mac6 = HardwareAddress::try_from(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff][..]).unwrap();
        assert_eq!(mac6.to_string(), "aa:bb:cc:dd:ee:ff");
        assert!(mac6.is_eui48());
        assert!(!mac6.is_eui64());
        assert_eq!(HardwareAddress::from([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]), mac6);
        let mac8 = HardwareAddress::from([1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(mac8.is_eui64());
        assert!(!mac8.is_eui48());
        let av = AttributeValue::from(mac6.clone());
        assert_eq!(HardwareAddress::try_from(av.clone()), Ok(mac6.clone()));
        assert_eq!(
            Nullable::<HardwareAddress>::try_from(av),
            Ok(Nullable::NonNull(mac6.clone()))
        );
        assert_eq!(&*mac6, &[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);

        let mac8_slice = HardwareAddress::try_from(&[1, 2, 3, 4, 5, 6, 7, 8][..]).unwrap();
        assert_eq!(mac8_slice.raw().len(), 8);

        assert_eq!(
            HardwareAddress::try_from(AttributeValue::Bytes(Bytes::from(&[1_u8, 2, 3][..]))),
            Err(MatterStatus::ConstraintError)
        );

        let default_mac = HardwareAddress::default();
        assert!(default_mac.is_eui48());
        assert_eq!(default_mac.to_string(), "00:00:00:00:00:00");
    }
}
