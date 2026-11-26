use alloc::vec::Vec;

use crate::model::{AttributeValue, MatterStatus, Nullable};

/// Trait implemented by primitive Matter integer widths for `Fit`-mode decoding from [`AttributeValue`].
pub(crate) trait DecodeWireScalar: Sized {
    fn decode_wire(val: &AttributeValue) -> Result<Self, MatterStatus>;
}

macro_rules! impl_decode_wire_scalar {
    ($($ty:ty => $fn_name:ident),* $(,)?) => {
        $(
            impl DecodeWireScalar for $ty {
                #[inline]
                fn decode_wire(val: &AttributeValue) -> Result<Self, MatterStatus> {
                    $fn_name(val)
                }
            }
        )*
    };
}

impl_decode_wire_scalar! {
    u8 => decode_unsigned,
    u16 => decode_unsigned,
    u32 => decode_unsigned,
    u64 => decode_unsigned,
    i8 => decode_signed,
    i16 => decode_signed,
    i32 => decode_signed,
    i64 => decode_signed,
}

/// Decodes any unsigned integer [`AttributeValue`] (`U8`..=`U64`) into `T` with range checking.
///
/// Returns [`MatterStatus::InvalidDataType`] for signed or non-integer variants, and
/// [`MatterStatus::ConstraintError`] on numeric overflow.
#[inline]
pub(crate) fn decode_unsigned<T: TryFrom<u64>>(val: &AttributeValue) -> Result<T, MatterStatus> {
    let raw = match val {
        AttributeValue::U8(v) => u64::from(*v),
        AttributeValue::U16(v) => u64::from(*v),
        AttributeValue::U32(v) => u64::from(*v),
        AttributeValue::U64(v) => *v,
        _ => return Err(MatterStatus::InvalidDataType),
    };
    T::try_from(raw).map_err(|_| MatterStatus::ConstraintError)
}

/// Decodes any signed integer [`AttributeValue`] (`I8`..=`I64`) into `T` with range checking.
///
/// Returns [`MatterStatus::InvalidDataType`] for unsigned or non-integer variants, and
/// [`MatterStatus::ConstraintError`] on numeric overflow.
#[inline]
pub(crate) fn decode_signed<T: TryFrom<i64>>(val: &AttributeValue) -> Result<T, MatterStatus> {
    let raw = match val {
        AttributeValue::I8(v) => i64::from(*v),
        AttributeValue::I16(v) => i64::from(*v),
        AttributeValue::I32(v) => i64::from(*v),
        AttributeValue::I64(v) => *v,
        _ => return Err(MatterStatus::InvalidDataType),
    };
    T::try_from(raw).map_err(|_| MatterStatus::ConstraintError)
}

/// Decodes a primitive Matter integer (`u8`..`u64`, `i8`..`i64`) from [`AttributeValue`] using `Fit` semantics.
#[inline]
pub(crate) fn decode_scalar<T: DecodeWireScalar>(val: &AttributeValue) -> Result<T, MatterStatus> {
    T::decode_wire(val)
}

/// Helper to decode an array field from an owned [`AttributeValue::List`] without cloning.
#[inline]
pub(crate) fn decode_list<T, E>(
    val: AttributeValue,
    mut f: impl FnMut(AttributeValue) -> Result<T, E>,
) -> Result<Vec<T>, MatterStatus>
where
    MatterStatus: From<E>,
{
    match val {
        AttributeValue::List(items) => items
            .into_iter()
            .map(&mut f)
            .collect::<Result<Vec<T>, _>>()
            .map_err(Into::into),
        _ => Err(MatterStatus::InvalidDataType),
    }
}

/// Helper to decode a nullable field from an owned [`AttributeValue`] without cloning.
#[inline]
pub(crate) fn decode_nullable<T, E>(
    val: AttributeValue,
    f: impl FnOnce(AttributeValue) -> Result<T, E>,
) -> Result<Nullable<T>, MatterStatus>
where
    MatterStatus: From<E>,
{
    match val {
        AttributeValue::Null => Ok(Nullable::Null),
        other => f(other).map(Nullable::NonNull).map_err(Into::into),
    }
}

/// Helper to decode a nullable array field from an owned [`AttributeValue`] without cloning.
#[inline]
pub(crate) fn decode_nullable_list<T, E>(
    val: AttributeValue,
    mut f: impl FnMut(AttributeValue) -> Result<T, E>,
) -> Result<Nullable<Vec<T>>, MatterStatus>
where
    MatterStatus: From<E>,
{
    match val {
        AttributeValue::Null => Ok(Nullable::Null),
        AttributeValue::List(items) => {
            let res = items.into_iter().map(&mut f).collect::<Result<Vec<T>, _>>()?;
            Ok(Nullable::NonNull(res))
        }
        _ => Err(MatterStatus::InvalidDataType),
    }
}
