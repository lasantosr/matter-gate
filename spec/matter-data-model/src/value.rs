//! Field identifier and dynamically-typed wire value per CSA Matter Specification Section 7.18.

extern crate alloc;

use alloc::{
    borrow::{Cow, ToOwned as _},
    string::String,
    vec::Vec,
};
use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
};

use bytes::Bytes;

use crate::status::MatterStatus;

define_matter_id! {
    /// Strongly-typed Matter Struct or Command Field identifier (`u8` context-specific TLV tag) per Section 7.18.
    pub struct FieldId(u8);
    display = hex;
    mei = false;
}

impl FieldId {
    /// Global Fabric Index field tag (`0xFE` / `254`) on fabric-scoped structs per CSA Matter Core Specification
    /// §7.5.3 & §7.18.2.25.
    pub const FABRIC_INDEX: Self = Self(0xFE);
}

/// Compact Matter attribute or field value wire representation per Section 7.18.
///
/// Avoids heap allocations for booleans, integers, and floats on the hot path while supporting
/// strings, binary buffers, lists, and field-tagged structs.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum AttributeValue {
    /// Null sentinel value (`null`).
    #[default]
    Null,
    /// Boolean value (`bool`).
    Bool(bool),
    /// Unsigned 8-bit integer (`uint8`).
    U8(u8),
    /// Unsigned 16-bit integer (`uint16`).
    U16(u16),
    /// Unsigned 32-bit integer (`uint32`).
    U32(u32),
    /// Unsigned 64-bit integer (`uint64`).
    U64(u64),
    /// Signed 8-bit integer (`int8`).
    I8(i8),
    /// Signed 16-bit integer (`int16`).
    I16(i16),
    /// Signed 32-bit integer (`int32`).
    I32(i32),
    /// Signed 64-bit integer (`int64`).
    I64(i64),
    /// IEEE 754 single-precision floating-point number (`single`).
    F32(f32),
    /// IEEE 754 double-precision floating-point number (`double`).
    F64(f64),
    /// UTF-8 character string (`string`).
    String(String),
    /// Octet string (`octstr`), backed by zero-copy `bytes::Bytes`.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes_repr"))]
    #[cfg_attr(feature = "schemars", schemars(with = "Vec<u8>"))]
    Bytes(Bytes),
    /// Ordered list of heterogeneous wire values (`list`).
    List(Vec<AttributeValue>),
    /// Field-tagged Matter struct (`struct`).
    Struct(Vec<(FieldId, AttributeValue)>),
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(val) => write!(f, "{val}"),
            Self::U8(val) => write!(f, "{val}"),
            Self::U16(val) => write!(f, "{val}"),
            Self::U32(val) => write!(f, "{val}"),
            Self::U64(val) => write!(f, "{val}"),
            Self::I8(val) => write!(f, "{val}"),
            Self::I16(val) => write!(f, "{val}"),
            Self::I32(val) => write!(f, "{val}"),
            Self::I64(val) => write!(f, "{val}"),
            Self::F32(val) => write!(f, "{val}"),
            Self::F64(val) => write!(f, "{val}"),
            Self::String(val) => write!(f, "{val:?}"),
            Self::Bytes(val) => write!(f, "<{} bytes>", val.len()),
            Self::List(items) => write!(f, "<list of {} items>", items.len()),
            Self::Struct(fields) => write!(f, "<struct of {} fields>", fields.len()),
        }
    }
}

impl AttributeValue {
    /// Construct an [`AttributeValue::Bytes`] from a byte buffer.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        Self::Bytes(bytes.into())
    }

    /// Construct an [`AttributeValue::List`] from any iterator of convertible items.
    #[must_use]
    pub fn from_list<T: Into<Self>>(items: impl IntoIterator<Item = T>) -> Self {
        Self::List(items.into_iter().map(Into::into).collect())
    }

    /// Construct an [`AttributeValue::Struct`] from any iterator of `(FieldId, value)` pairs.
    #[must_use]
    pub fn from_struct<T: Into<Self>>(fields: impl IntoIterator<Item = (FieldId, T)>) -> Self {
        Self::Struct(fields.into_iter().map(|(id, val)| (id, val.into())).collect())
    }

    /// Return `true` if this value is [`AttributeValue::Null`].
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Return the inner `bool` if this value is [`AttributeValue::Bool`].
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `u8` if this value is [`AttributeValue::U8`].
    #[must_use]
    pub const fn as_u8(&self) -> Option<u8> {
        match self {
            Self::U8(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `u16` if this value is [`AttributeValue::U16`].
    #[must_use]
    pub const fn as_u16(&self) -> Option<u16> {
        match self {
            Self::U16(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `u32` if this value is [`AttributeValue::U32`].
    #[must_use]
    pub const fn as_u32(&self) -> Option<u32> {
        match self {
            Self::U32(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `u64` if this value is [`AttributeValue::U64`].
    #[must_use]
    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            Self::U64(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `i8` if this value is [`AttributeValue::I8`].
    #[must_use]
    pub const fn as_i8(&self) -> Option<i8> {
        match self {
            Self::I8(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `i16` if this value is [`AttributeValue::I16`].
    #[must_use]
    pub const fn as_i16(&self) -> Option<i16> {
        match self {
            Self::I16(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `i32` if this value is [`AttributeValue::I32`].
    #[must_use]
    pub const fn as_i32(&self) -> Option<i32> {
        match self {
            Self::I32(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `i64` if this value is [`AttributeValue::I64`].
    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I64(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `f32` if this value is [`AttributeValue::F32`].
    #[must_use]
    pub const fn as_f32(&self) -> Option<f32> {
        match self {
            Self::F32(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner `f64` if this value is [`AttributeValue::F64`].
    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F64(v) => Some(*v),
            _ => None,
        }
    }

    /// Return the inner string slice if this value is [`AttributeValue::String`].
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Return the inner byte slice if this value is [`AttributeValue::Bytes`].
    #[must_use]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b.as_ref()),
            _ => None,
        }
    }

    /// Return the inner item slice if this value is [`AttributeValue::List`].
    #[must_use]
    pub fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    /// Return a mutable reference to the inner item vector if this value is [`AttributeValue::List`].
    #[must_use]
    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }

    /// Return the inner field slice if this value is [`AttributeValue::Struct`].
    #[must_use]
    pub fn as_struct(&self) -> Option<&[(FieldId, Self)]> {
        match self {
            Self::Struct(fields) => Some(fields.as_slice()),
            _ => None,
        }
    }

    /// Return a mutable reference to the inner field vector if this value is [`AttributeValue::Struct`].
    #[must_use]
    pub fn as_struct_mut(&mut self) -> Option<&mut Vec<(FieldId, Self)>> {
        match self {
            Self::Struct(fields) => Some(fields),
            _ => None,
        }
    }

    /// Look up a struct field by [`FieldId`] if this value is [`AttributeValue::Struct`].
    #[must_use]
    pub fn field(&self, id: FieldId) -> Option<&Self> {
        self.as_struct()?
            .iter()
            .find(|(field_id, _)| *field_id == id)
            .map(|(_, val)| val)
    }

    /// Look up a mutable struct field by [`FieldId`] if this value is [`AttributeValue::Struct`].
    #[must_use]
    pub fn field_mut(&mut self, id: FieldId) -> Option<&mut Self> {
        self.as_struct_mut()?
            .iter_mut()
            .find(|(field_id, _)| *field_id == id)
            .map(|(_, val)| val)
    }

    /// Insert or update a struct field by [`FieldId`] if this value is [`AttributeValue::Struct`].
    ///
    /// Returns `Ok(Some(prev))` if replaced, `Ok(None)` if inserted, or `Err(MatterStatus::InvalidDataType)`
    /// if `self` is not [`AttributeValue::Struct`].
    pub fn set_field(&mut self, id: FieldId, value: impl Into<Self>) -> Result<Option<Self>, MatterStatus> {
        let fields = self.as_struct_mut().ok_or(MatterStatus::InvalidDataType)?;
        let val = value.into();
        if let Some((_, existing)) = fields.iter_mut().find(|(field_id, _)| *field_id == id) {
            Ok(Some(mem::replace(existing, val)))
        } else {
            fields.push((id, val));
            Ok(None)
        }
    }

    /// Remove a struct field by [`FieldId`] if this value is [`AttributeValue::Struct`],
    /// returning its value if present.
    pub fn remove_field(&mut self, id: FieldId) -> Option<Self> {
        let fields = self.as_struct_mut()?;
        fields
            .iter()
            .position(|(field_id, _)| *field_id == id)
            .map(|idx| fields.remove(idx).1)
    }

    /// Return `true` if this value is [`AttributeValue::Struct`] and contains a field with `id`.
    #[must_use]
    pub fn has_field(&self, id: FieldId) -> bool {
        self.as_struct()
            .is_some_and(|fields| fields.iter().any(|(field_id, _)| *field_id == id))
    }
}

macro_rules! impl_primitive_attribute_value {
    ($($ty:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$ty> for AttributeValue {
                fn from(val: $ty) -> Self {
                    Self::$variant(val)
                }
            }

            impl TryFrom<&AttributeValue> for $ty {
                type Error = MatterStatus;

                fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
                    match val {
                        AttributeValue::$variant(v) => Ok(*v),
                        _ => Err(MatterStatus::InvalidDataType),
                    }
                }
            }

            impl TryFrom<AttributeValue> for $ty {
                type Error = MatterStatus;

                fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
                    (&val).try_into()
                }
            }
        )*
    };
}

impl_primitive_attribute_value!(
    bool => Bool,
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    f32 => F32,
    f64 => F64,
);

impl From<String> for AttributeValue {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}

impl From<&str> for AttributeValue {
    fn from(val: &str) -> Self {
        Self::String(val.to_owned())
    }
}

impl From<Bytes> for AttributeValue {
    fn from(val: Bytes) -> Self {
        Self::Bytes(val)
    }
}

impl From<Vec<u8>> for AttributeValue {
    fn from(val: Vec<u8>) -> Self {
        Self::Bytes(Bytes::from(val))
    }
}

impl From<&[u8]> for AttributeValue {
    fn from(val: &[u8]) -> Self {
        Self::Bytes(Bytes::copy_from_slice(val))
    }
}

impl<const N: usize> From<[u8; N]> for AttributeValue {
    fn from(val: [u8; N]) -> Self {
        Self::Bytes(Bytes::copy_from_slice(&val))
    }
}

impl<const N: usize> From<&[u8; N]> for AttributeValue {
    fn from(val: &[u8; N]) -> Self {
        Self::Bytes(Bytes::copy_from_slice(val))
    }
}

impl From<Cow<'_, str>> for AttributeValue {
    fn from(val: Cow<'_, str>) -> Self {
        Self::String(val.into_owned())
    }
}

impl From<Cow<'_, [u8]>> for AttributeValue {
    fn from(val: Cow<'_, [u8]>) -> Self {
        Self::Bytes(Bytes::from(val.into_owned()))
    }
}

impl From<Vec<Self>> for AttributeValue {
    fn from(val: Vec<Self>) -> Self {
        Self::List(val)
    }
}

impl From<Vec<(FieldId, Self)>> for AttributeValue {
    fn from(val: Vec<(FieldId, Self)>) -> Self {
        Self::Struct(val)
    }
}

impl<T: Into<Self>> From<Option<T>> for AttributeValue {
    fn from(val: Option<T>) -> Self {
        match val {
            None => Self::Null,
            Some(v) => v.into(),
        }
    }
}

#[cfg(feature = "serde")]
mod serde_bytes_repr {
    use alloc::vec::Vec;
    use core::fmt;

    use bytes::Bytes;
    use serde::{
        Deserializer, Serializer,
        de::{Error as DeError, SeqAccess, Visitor},
    };

    pub(super) fn serialize<S: Serializer>(val: &Bytes, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(val.as_ref())
    }

    struct BytesVisitor;

    impl<'de> Visitor<'de> for BytesVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a byte array")
        }

        fn visit_bytes<E: DeError>(self, v: &[u8]) -> Result<Self::Value, E> {
            Ok(v.to_vec())
        }

        fn visit_byte_buf<E: DeError>(self, v: Vec<u8>) -> Result<Self::Value, E> {
            Ok(v)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(b) = seq.next_element::<u8>()? {
                vec.push(b);
            }
            Ok(vec)
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Bytes, D::Error> {
        deserializer.deserialize_byte_buf(BytesVisitor).map(Bytes::from)
    }
}

// References TryFrom

impl TryFrom<&AttributeValue> for String {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::String(v) => Ok(v.clone()),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<&AttributeValue> for Bytes {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::Bytes(v) => Ok(v.clone()),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<&AttributeValue> for Vec<u8> {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::Bytes(v) => Ok(v.to_vec()),
            AttributeValue::List(items) => items
                .iter()
                .map(|item| match item {
                    AttributeValue::U8(b) => Ok(*b),
                    _ => Err(MatterStatus::InvalidDataType),
                })
                .collect(),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<&AttributeValue> for Vec<AttributeValue> {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::List(items) => Ok(items.clone()),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<&AttributeValue> for Vec<(FieldId, AttributeValue)> {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::Struct(fields) => Ok(fields.clone()),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl<'a> TryFrom<&'a AttributeValue> for &'a str {
    type Error = MatterStatus;

    fn try_from(val: &'a AttributeValue) -> Result<Self, MatterStatus> {
        val.as_str().ok_or(MatterStatus::InvalidDataType)
    }
}

impl<'a> TryFrom<&'a AttributeValue> for &'a [u8] {
    type Error = MatterStatus;

    fn try_from(val: &'a AttributeValue) -> Result<Self, MatterStatus> {
        val.as_bytes().ok_or(MatterStatus::InvalidDataType)
    }
}

// Owned TryFrom

impl TryFrom<AttributeValue> for String {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::String(v) => Ok(v),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<AttributeValue> for Bytes {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::Bytes(v) => Ok(v),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<AttributeValue> for Vec<u8> {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::Bytes(v) => Ok(v.to_vec()),
            AttributeValue::List(items) => items
                .into_iter()
                .map(|item| match item {
                    AttributeValue::U8(b) => Ok(b),
                    _ => Err(MatterStatus::InvalidDataType),
                })
                .collect(),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<AttributeValue> for Vec<AttributeValue> {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::List(items) => Ok(items),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

impl TryFrom<AttributeValue> for Vec<(FieldId, AttributeValue)> {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::Struct(fields) => Ok(fields),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_id_display_and_roundtrip() {
        assert_eq!(FieldId::FABRIC_INDEX.raw(), 0xFE);
        assert_eq!(FieldId::new(0x03).raw(), 0x03);
        assert_eq!(alloc::format!("{}", FieldId::new(0x03)), "0x03");
        assert_eq!(u8::from(FieldId::new(0x03)), 0x03);
        let val = AttributeValue::from(FieldId::new(0x03));
        assert_eq!(val, AttributeValue::U8(0x03));
        assert_eq!(FieldId::try_from(&val), Ok(FieldId::new(0x03)));
        assert_eq!(FieldId::try_from(val), Ok(FieldId::new(0x03)));
    }

    #[test]
    fn attribute_value_conversions() {
        assert_eq!(AttributeValue::from(true), AttributeValue::Bool(true));
        assert_eq!(bool::try_from(&AttributeValue::Bool(true)), Ok(true));
        assert_eq!(bool::try_from(AttributeValue::Bool(true)), Ok(true));

        assert_eq!(AttributeValue::from(42_u8), AttributeValue::U8(42));
        assert_eq!(u8::try_from(&AttributeValue::U8(42)), Ok(42));
        assert_eq!(u8::try_from(AttributeValue::U8(42)), Ok(42));

        assert_eq!(AttributeValue::from(1000_u16), AttributeValue::U16(1000));
        assert_eq!(u16::try_from(&AttributeValue::U16(1000)), Ok(1000));

        assert_eq!(AttributeValue::from(100_000_u32), AttributeValue::U32(100_000));
        assert_eq!(u32::try_from(&AttributeValue::U32(100_000)), Ok(100_000));

        assert_eq!(
            AttributeValue::from(10_000_000_000_u64),
            AttributeValue::U64(10_000_000_000)
        );
        assert_eq!(u64::try_from(&AttributeValue::U64(10_000_000_000)), Ok(10_000_000_000));

        assert_eq!(AttributeValue::from(-10_i8), AttributeValue::I8(-10));
        assert_eq!(i8::try_from(&AttributeValue::I8(-10)), Ok(-10));

        assert_eq!(AttributeValue::from(-500_i16), AttributeValue::I16(-500));
        assert_eq!(i16::try_from(&AttributeValue::I16(-500)), Ok(-500));

        assert_eq!(AttributeValue::from(-70_000_i32), AttributeValue::I32(-70_000));
        assert_eq!(i32::try_from(&AttributeValue::I32(-70_000)), Ok(-70_000));

        assert_eq!(
            AttributeValue::from(-8_000_000_000_i64),
            AttributeValue::I64(-8_000_000_000)
        );
        assert_eq!(i64::try_from(&AttributeValue::I64(-8_000_000_000)), Ok(-8_000_000_000));

        assert_eq!(AttributeValue::from(1.5_f32), AttributeValue::F32(1.5));
        assert_eq!(f32::try_from(&AttributeValue::F32(1.5)), Ok(1.5));

        assert_eq!(AttributeValue::from(2.5_f64), AttributeValue::F64(2.5));
        assert_eq!(f64::try_from(&AttributeValue::F64(2.5)), Ok(2.5));

        assert_eq!(AttributeValue::from("test"), AttributeValue::String("test".into()));
        assert_eq!(
            String::try_from(&AttributeValue::String("test".into())),
            Ok("test".into())
        );
        assert_eq!(
            String::try_from(AttributeValue::String("test".into())),
            Ok("test".into())
        );

        let b = Bytes::from_static(b"hello");
        assert_eq!(AttributeValue::from(b.clone()), AttributeValue::Bytes(b.clone()));
        assert_eq!(Bytes::try_from(&AttributeValue::Bytes(b.clone())), Ok(b.clone()));
        assert_eq!(Bytes::try_from(AttributeValue::Bytes(b.clone())), Ok(b));
        assert_eq!(
            Bytes::try_from(&AttributeValue::U8(1)),
            Err(MatterStatus::InvalidDataType)
        );

        let slice_val = AttributeValue::from(b"hello".as_slice());
        assert_eq!(alloc::format!("{slice_val}"), "<5 bytes>");
        assert_eq!(Vec::<u8>::try_from(&slice_val), Ok(b"hello".to_vec()));
        assert_eq!(Vec::<u8>::try_from(slice_val), Ok(b"hello".to_vec()));
        assert_eq!(
            Vec::<u8>::try_from(&AttributeValue::String("nope".into())),
            Err(MatterStatus::InvalidDataType)
        );

        let list = alloc::vec![AttributeValue::U8(1), AttributeValue::U8(2)];
        assert_eq!(AttributeValue::from(list.clone()), AttributeValue::List(list));

        let struct_val = AttributeValue::from(alloc::vec![(FieldId::new(0), AttributeValue::U8(10))]);
        assert_eq!(
            struct_val,
            AttributeValue::Struct(alloc::vec![(FieldId::new(0), AttributeValue::U8(10))])
        );

        assert_eq!(
            bool::try_from(&AttributeValue::Null),
            Err(MatterStatus::InvalidDataType)
        );
        assert_eq!(
            u8::try_from(&AttributeValue::Bool(true)),
            Err(MatterStatus::InvalidDataType)
        );
    }

    #[test]
    fn attribute_value_bytes_and_accessors() {
        // Vec<u8> converts to AttributeValue::Bytes
        let vec_bytes = AttributeValue::from(alloc::vec![1_u8, 2, 3]);
        assert_eq!(vec_bytes.as_bytes(), Some(&[1_u8, 2, 3][..]));
        assert_eq!(Vec::<u8>::try_from(&vec_bytes), Ok(alloc::vec![1, 2, 3]));
        assert_eq!(Vec::<u8>::try_from(vec_bytes), Ok(alloc::vec![1, 2, 3]));

        let list_val = AttributeValue::from_list([1_u16, 2, 3]);
        assert_eq!(
            Vec::<AttributeValue>::try_from(&list_val),
            Ok(alloc::vec![
                AttributeValue::U16(1),
                AttributeValue::U16(2),
                AttributeValue::U16(3),
            ])
        );

        assert_eq!(AttributeValue::from(None::<u8>), AttributeValue::Null);
        assert_eq!(AttributeValue::from(Some(5_u8)), AttributeValue::U8(5));

        let slice_bytes = AttributeValue::from(&b"hello"[..]);
        assert_eq!(slice_bytes.as_bytes(), Some(&b"hello"[..]));
        assert_eq!(<&[u8]>::try_from(&slice_bytes), Ok(&b"hello"[..]));

        let from_bytes_val = AttributeValue::from_bytes(alloc::vec![4_u8, 5]);
        assert_eq!(from_bytes_val.as_bytes(), Some(&[4_u8, 5][..]));

        let str_val = AttributeValue::from("matter");
        assert_eq!(str_val.as_str(), Some("matter"));
        assert_eq!(<&str>::try_from(&str_val), Ok("matter"));

        assert!(AttributeValue::Null.is_null());
        assert_eq!(AttributeValue::Bool(true).as_bool(), Some(true));
        assert_eq!(AttributeValue::U8(7).as_u8(), Some(7));
        assert_eq!(AttributeValue::U16(8).as_u16(), Some(8));
        assert_eq!(AttributeValue::U32(9).as_u32(), Some(9));
        assert_eq!(AttributeValue::U64(10).as_u64(), Some(10));
        assert_eq!(AttributeValue::I8(-1).as_i8(), Some(-1));
        assert_eq!(AttributeValue::I16(-2).as_i16(), Some(-2));
        assert_eq!(AttributeValue::I32(-3).as_i32(), Some(-3));
        assert_eq!(AttributeValue::I64(-4).as_i64(), Some(-4));
        assert_eq!(AttributeValue::F32(1.5).as_f32(), Some(1.5));
        assert_eq!(AttributeValue::F64(2.5).as_f64(), Some(2.5));
        assert!(AttributeValue::U8(1).as_bool().is_none());

        let s = AttributeValue::Struct(alloc::vec![(FieldId::new(1), AttributeValue::U8(42))]);
        assert_eq!(s.field(FieldId::new(1)), Some(&AttributeValue::U8(42)));
        assert!(s.field(FieldId::new(2)).is_none());

        let mut struct_val = AttributeValue::Struct(alloc::vec![(FieldId::new(1), AttributeValue::U8(10))]);
        if let Some(v) = struct_val.field_mut(FieldId::new(1)) {
            *v = AttributeValue::U8(20);
        }
        assert_eq!(struct_val.field(FieldId::new(1)), Some(&AttributeValue::U8(20)));
        assert_eq!(
            struct_val.set_field(FieldId::new(1), 30_u8),
            Ok(Some(AttributeValue::U8(20)))
        );
        assert_eq!(struct_val.set_field(FieldId::new(2), true), Ok(None));
        assert_eq!(
            AttributeValue::Null.set_field(FieldId::new(0), 1_u8),
            Err(MatterStatus::InvalidDataType)
        );
        assert_eq!(alloc::format!("{}", AttributeValue::from("a\"b")), "\"a\\\"b\"");
        assert!(struct_val.has_field(FieldId::new(2)));
        assert_eq!(struct_val.field(FieldId::new(2)), Some(&AttributeValue::Bool(true)));
        assert_eq!(
            struct_val.remove_field(FieldId::new(2)),
            Some(AttributeValue::Bool(true))
        );
        assert!(!struct_val.has_field(FieldId::new(2)));
        assert_eq!(
            Vec::<(FieldId, AttributeValue)>::try_from(&struct_val).map(|f| f.len()),
            Ok(1)
        );

        assert_eq!(
            AttributeValue::from(Cow::Borrowed("cow")),
            AttributeValue::String(String::from("cow"))
        );
        assert_eq!(AttributeValue::from([1_u8, 2]).as_bytes(), Some(&[1_u8, 2][..]));
        assert_eq!(AttributeValue::from(&[3_u8, 4]).as_bytes(), Some(&[3_u8, 4][..]));
        assert_eq!(
            AttributeValue::from(Cow::Borrowed(&[5_u8, 6][..])).as_bytes(),
            Some(&[5_u8, 6][..])
        );
        assert_eq!("0x03".parse::<FieldId>(), Ok(FieldId::new(3)));

        let from_struct_val = AttributeValue::from_struct([(FieldId::new(0), 10_u8), (FieldId::new(1), 20_u8)]);
        assert_eq!(from_struct_val.field(FieldId::new(0)), Some(&AttributeValue::U8(10)));
        assert_eq!(from_struct_val.field(FieldId::new(1)), Some(&AttributeValue::U8(20)));
    }

    #[test]
    fn attribute_value_matches_exhaustively() {
        let vals = [
            AttributeValue::Null,
            AttributeValue::Bool(true),
            AttributeValue::U8(1),
            AttributeValue::U16(2),
            AttributeValue::U32(3),
            AttributeValue::U64(4),
            AttributeValue::I8(-1),
            AttributeValue::I16(-2),
            AttributeValue::I32(-3),
            AttributeValue::I64(-4),
            AttributeValue::F32(1.0),
            AttributeValue::F64(2.0),
            AttributeValue::String(String::from("ok")),
            AttributeValue::Bytes(Bytes::from_static(b"ab")),
            AttributeValue::List(alloc::vec![]),
            AttributeValue::Struct(alloc::vec![]),
        ];

        for val in vals {
            let tag = match val {
                AttributeValue::Null => "null",
                AttributeValue::Bool(_) => "bool",
                AttributeValue::U8(_) => "u8",
                AttributeValue::U16(_) => "u16",
                AttributeValue::U32(_) => "u32",
                AttributeValue::U64(_) => "u64",
                AttributeValue::I8(_) => "i8",
                AttributeValue::I16(_) => "i16",
                AttributeValue::I32(_) => "i32",
                AttributeValue::I64(_) => "i64",
                AttributeValue::F32(_) => "f32",
                AttributeValue::F64(_) => "f64",
                AttributeValue::String(_) => "string",
                AttributeValue::Bytes(_) => "bytes",
                AttributeValue::List(_) => "list",
                AttributeValue::Struct(_) => "struct",
            };
            assert_ne!(tag, "");
        }
    }
}
