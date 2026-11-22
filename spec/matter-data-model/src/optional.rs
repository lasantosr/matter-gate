//! Optionality and nullability representations per CSA Matter Specification Section 7.18.

extern crate alloc;

#[cfg(feature = "schemars")]
use alloc::borrow::Cow;
use alloc::{borrow::ToOwned as _, string::String};
use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
    ops::{Deref, DerefMut},
    option::IntoIter as OptionIntoIter,
};

use bytes::Bytes;
#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{status::MatterStatus, value::AttributeValue};

/// Explicit representation of Matter optionality (field may be omitted on the wire).
///
/// In Matter's Data Model, optionality (whether a field may be omitted) and nullability
/// (whether a field may be `AttributeValue::Null`) are orthogonal. `Optional<T>` represents
/// fields that may be omitted entirely from serialization.
///
/// # Serde Usage
///
/// On struct fields, pair `Optional<T>` with `#[serde(default, skip_serializing_if = "Optional::is_omitted")]`.
/// Note that field omission relies on self-describing keyed formats (JSON, CBOR, TOML, YAML, `MessagePack`);
/// non-self-describing positional binary formats (such as `postcard` or `bincode`) cannot omit struct fields.
///
/// When combining `Optional<Nullable<T>>`, also add
/// `deserialize_with = "matter_data_model::serde::deserialize_optional_nullable"` so that an explicit
/// `null` value deserializes as `Optional::Present(Nullable::Null)` rather than `Optional::Omitted`:
///
/// ```rust,ignore
/// #[serde(
///     default,
///     skip_serializing_if = "Optional::is_omitted",
///     deserialize_with = "matter_data_model::serde::deserialize_optional_nullable"
/// )]
/// pub label: Optional<Nullable<String>>,
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Optional<T> {
    /// Field is omitted from the wire representation.
    #[default]
    Omitted,
    /// Field is present on the wire with value `T`.
    Present(T),
}

impl<T> Optional<T> {
    /// Return `true` if this value is [`Optional::Present`].
    #[must_use]
    pub const fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }

    /// Return `true` if this value is [`Optional::Omitted`].
    #[must_use]
    pub const fn is_omitted(&self) -> bool {
        matches!(self, Self::Omitted)
    }

    /// Return `true` if this value is [`Optional::Present`] and the contained value satisfies `f`.
    #[must_use]
    pub fn is_present_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
        matches!(self, Self::Present(v) if f(v))
    }

    /// Return `true` if this value is [`Optional::Omitted`] or the contained value satisfies `f`.
    #[must_use]
    pub fn is_omitted_or(&self, f: impl FnOnce(&T) -> bool) -> bool {
        match self {
            Self::Omitted => true,
            Self::Present(v) => f(v),
        }
    }

    /// Convert from `&Optional<T>` to `Optional<&T>`.
    #[must_use]
    pub const fn as_ref(&self) -> Optional<&T> {
        match self {
            Self::Omitted => Optional::Omitted,
            Self::Present(v) => Optional::Present(v),
        }
    }

    /// Convert from `&mut Optional<T>` to `Optional<&mut T>`.
    #[must_use]
    pub fn as_mut(&mut self) -> Optional<&mut T> {
        match self {
            Self::Omitted => Optional::Omitted,
            Self::Present(v) => Optional::Present(v),
        }
    }

    /// Map an `Optional<T>` to `Optional<U>` by applying `f` to a present value.
    #[must_use]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Optional<U> {
        match self {
            Self::Omitted => Optional::Omitted,
            Self::Present(v) => Optional::Present(f(v)),
        }
    }

    /// Call `f` with a reference to the present value (if any), then return `self`.
    #[must_use]
    pub fn inspect(self, f: impl FnOnce(&T)) -> Self {
        if let Self::Present(v) = &self {
            f(v);
        }
        self
    }

    /// Return [`Optional::Omitted`] if omitted, otherwise return `optb`.
    #[must_use]
    pub fn and<U>(self, optb: Optional<U>) -> Optional<U> {
        match self {
            Self::Omitted => Optional::Omitted,
            Self::Present(_) => optb,
        }
    }

    /// Return [`Optional::Omitted`] if omitted, otherwise call `f` with the present value.
    #[must_use]
    pub fn and_then<U>(self, f: impl FnOnce(T) -> Optional<U>) -> Optional<U> {
        match self {
            Self::Omitted => Optional::Omitted,
            Self::Present(v) => f(v),
        }
    }

    /// Zip `self` with `other`, returning `Present((a, b))` if both are `Present`, or `Omitted`.
    #[must_use]
    pub fn zip<U>(self, other: Optional<U>) -> Optional<(T, U)> {
        match (self, other) {
            (Self::Present(a), Optional::Present(b)) => Optional::Present((a, b)),
            _ => Optional::Omitted,
        }
    }

    /// Convert into `Option<T>`, mapping [`Optional::Present`] to `Some` and [`Optional::Omitted`] to `None`.
    #[must_use]
    pub fn into_option(self) -> Option<T> {
        match self {
            Self::Omitted => None,
            Self::Present(v) => Some(v),
        }
    }

    /// Borrow as `Option<&T>`.
    #[must_use]
    pub const fn as_option(&self) -> Option<&T> {
        match self {
            Self::Omitted => None,
            Self::Present(v) => Some(v),
        }
    }

    /// Unwrap a present value, panicking if [`Optional::Omitted`].
    #[expect(clippy::panic, reason = "Explicit unwrap panics by design on Omitted")]
    pub fn unwrap(self) -> T {
        match self {
            Self::Present(v) => v,
            Self::Omitted => panic!("called `Optional::unwrap()` on an `Omitted` value"),
        }
    }

    /// Unwrap a present value, panicking with `msg` if [`Optional::Omitted`].
    #[expect(clippy::panic, reason = "Explicit expect panics by design on Omitted")]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Self::Present(v) => v,
            Self::Omitted => panic!("{msg}"),
        }
    }

    /// Return `default` if omitted, or apply `f` to the present value.
    #[must_use]
    pub fn map_or<U>(self, default: U, f: impl FnOnce(T) -> U) -> U {
        match self {
            Self::Omitted => default,
            Self::Present(v) => f(v),
        }
    }

    /// Compute a default from `default` if omitted, or apply `f` to the present value.
    #[must_use]
    pub fn map_or_else<U>(self, default: impl FnOnce() -> U, f: impl FnOnce(T) -> U) -> U {
        match self {
            Self::Omitted => default(),
            Self::Present(v) => f(v),
        }
    }

    /// Return `self` if present, otherwise return `optb`.
    #[must_use]
    pub fn or(self, optb: Self) -> Self {
        match self {
            Self::Present(_) => self,
            Self::Omitted => optb,
        }
    }

    /// Return `self` if present, otherwise call `f` and return the result.
    #[must_use]
    pub fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        match self {
            Self::Present(_) => self,
            Self::Omitted => f(),
        }
    }

    /// Return `Present` if exactly one of `self` and `optb` is `Present`, otherwise `Omitted`.
    #[must_use]
    pub fn xor(self, optb: Self) -> Self {
        match (self, optb) {
            (Self::Present(a), Self::Omitted) => Self::Present(a),
            (Self::Omitted, Self::Present(b)) => Self::Present(b),
            _ => Self::Omitted,
        }
    }

    /// Insert `value` into the `Optional`, returning a mutable reference to it.
    #[expect(clippy::unreachable, reason = "Self::Present was just assigned on the previous line")]
    pub fn insert(&mut self, value: T) -> &mut T {
        *self = Self::Present(value);
        match self {
            Self::Present(v) => v,
            Self::Omitted => unreachable!(),
        }
    }

    /// Insert `value` if omitted, then return a mutable reference to the contained value.
    #[expect(clippy::unreachable, reason = "Self::Present was just assigned if Omitted")]
    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        if self.is_omitted() {
            *self = Self::Present(value);
        }
        match self {
            Self::Present(v) => v,
            Self::Omitted => unreachable!(),
        }
    }

    /// Insert a value computed by `f` if omitted, then return a mutable reference to it.
    #[expect(clippy::unreachable, reason = "Self::Present was just assigned if Omitted")]
    pub fn get_or_insert_with(&mut self, f: impl FnOnce() -> T) -> &mut T {
        if self.is_omitted() {
            *self = Self::Present(f());
        }
        match self {
            Self::Present(v) => v,
            Self::Omitted => unreachable!(),
        }
    }

    /// Replace the value in this [`Optional`], returning the old value.
    #[must_use]
    pub const fn replace(&mut self, value: T) -> Self {
        mem::replace(self, Self::Present(value))
    }

    /// Return the contained value or `default` if omitted.
    #[must_use]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Present(v) => v,
            Self::Omitted => default,
        }
    }

    /// Return the contained value or `T::default()` if omitted.
    #[must_use]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Self::Present(v) => v,
            Self::Omitted => T::default(),
        }
    }

    /// Convert into a `Result`, using `err` when omitted.
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        self.into_option().ok_or(err)
    }

    /// Convert into a `Result`, computing the error from `err` when omitted.
    pub fn ok_or_else<E>(self, err: impl FnOnce() -> E) -> Result<T, E> {
        self.into_option().ok_or_else(err)
    }

    /// Return the contained value or compute it from `f` if omitted.
    #[must_use]
    pub fn unwrap_or_else(self, f: impl FnOnce() -> T) -> T {
        match self {
            Self::Present(v) => v,
            Self::Omitted => f(),
        }
    }

    /// Take the value out of this [`Optional`], leaving [`Optional::Omitted`] in its place.
    #[must_use]
    pub const fn take(&mut self) -> Self {
        mem::replace(self, Self::Omitted)
    }

    /// Reset this [`Optional`] to [`Optional::Omitted`] without returning the previous value.
    pub fn clear(&mut self) {
        *self = Self::Omitted;
    }

    /// Return [`Optional::Omitted`] if omitted, or call `predicate` on the wrapped value.
    #[must_use]
    pub fn filter(self, predicate: impl FnOnce(&T) -> bool) -> Self {
        match self {
            Self::Present(v) if predicate(&v) => Self::Present(v),
            _ => Self::Omitted,
        }
    }

    /// Convert from `&Optional<T>` to `Optional<&T::Target>`.
    #[must_use]
    pub fn as_deref(&self) -> Optional<&T::Target>
    where
        T: Deref,
    {
        match self {
            Self::Present(v) => Optional::Present(&**v),
            Self::Omitted => Optional::Omitted,
        }
    }

    /// Convert from `&mut Optional<T>` to `Optional<&mut T::Target>`.
    #[must_use]
    pub fn as_deref_mut(&mut self) -> Optional<&mut T::Target>
    where
        T: DerefMut,
    {
        match self {
            Self::Present(v) => Optional::Present(&mut **v),
            Self::Omitted => Optional::Omitted,
        }
    }

    /// Return an iterator over the possibly contained value.
    #[must_use]
    pub fn iter(&self) -> OptionIntoIter<&'_ T> {
        self.as_ref().into_option().into_iter()
    }

    /// Return a mutable iterator over the possibly contained value.
    #[must_use]
    pub fn iter_mut(&mut self) -> OptionIntoIter<&'_ mut T> {
        self.as_mut().into_option().into_iter()
    }
}

impl<T: Copy> Optional<&T> {
    /// Map an `Optional<&T>` to an `Optional<T>` by copying the contents.
    #[must_use]
    pub const fn copied(self) -> Optional<T> {
        match self {
            Self::Present(&v) => Optional::Present(v),
            Self::Omitted => Optional::Omitted,
        }
    }
}

impl<T: Clone> Optional<&T> {
    /// Map an `Optional<&T>` to an `Optional<T>` by cloning the contents.
    #[must_use]
    pub fn cloned(self) -> Optional<T> {
        match self {
            Self::Present(v) => Optional::Present(v.clone()),
            Self::Omitted => Optional::Omitted,
        }
    }
}

impl<T> Optional<Optional<T>> {
    /// Convert from `Optional<Optional<T>>` to `Optional<T>`.
    #[must_use]
    pub fn flatten(self) -> Optional<T> {
        match self {
            Self::Present(inner) => inner,
            Self::Omitted => Optional::Omitted,
        }
    }
}

impl<T, E> Optional<Result<T, E>> {
    /// Transpose an `Optional<Result<T, E>>` into a `Result<Optional<T>, E>`.
    pub fn transpose(self) -> Result<Optional<T>, E> {
        match self {
            Self::Omitted => Ok(Optional::Omitted),
            Self::Present(Ok(v)) => Ok(Optional::Present(v)),
            Self::Present(Err(e)) => Err(e),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<T: Serialize> Serialize for Optional<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Present(v) => serializer.serialize_some(v),
            Self::Omitted => serializer.serialize_none(),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Optional<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(feature = "schemars")]
#[cfg_attr(docsrs, doc(cfg(feature = "schemars")))]
impl<T: JsonSchema> JsonSchema for Optional<T> {
    fn schema_name() -> Cow<'static, str> {
        T::schema_name()
    }

    fn schema_id() -> Cow<'static, str> {
        T::schema_id()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        T::json_schema(generator)
    }

    fn inline_schema() -> bool {
        T::inline_schema()
    }
}

impl<T> Optional<Nullable<T>> {
    /// Construct `Optional::Present(Nullable::NonNull(val))`.
    #[must_use]
    pub const fn present_nonnull(val: T) -> Self {
        Self::Present(Nullable::NonNull(val))
    }

    /// Construct `Optional::Present(Nullable::Null)`.
    #[must_use]
    pub const fn present_null() -> Self {
        Self::Present(Nullable::Null)
    }

    /// Return `Some(&mut T)` when `Optional::Present(Nullable::NonNull(v))`, otherwise `None`.
    #[must_use]
    pub fn as_nonnull_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Present(Nullable::NonNull(v)) => Some(v),
            _ => None,
        }
    }

    /// Return `Some(&T)` when `Optional::Present(Nullable::NonNull(ref v))`, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_nonnull(&self) -> Option<&T> {
        match self {
            Self::Present(Nullable::NonNull(v)) => Some(v),
            Self::Omitted | Self::Present(Nullable::Null) => None,
        }
    }

    /// Return `Some(T)` when `Optional::Present(Nullable::NonNull(v))`, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn into_nonnull(self) -> Option<T> {
        match self {
            Self::Present(Nullable::NonNull(v)) => Some(v),
            Self::Omitted | Self::Present(Nullable::Null) => None,
        }
    }

    /// Return `true` when `Optional::Present(Nullable::Null)`.
    #[inline]
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Present(Nullable::Null))
    }

    /// Return `true` when `Optional::Present(Nullable::NonNull(_))`.
    #[inline]
    #[must_use]
    pub const fn is_nonnull(&self) -> bool {
        matches!(self, Self::Present(Nullable::NonNull(_)))
    }

    /// Map the inner value when `Optional::Present(Nullable::NonNull(v))`, preserving `Omitted` and `Null`.
    #[inline]
    #[must_use]
    pub fn map_nonnull<U>(self, f: impl FnOnce(T) -> U) -> Optional<Nullable<U>> {
        match self {
            Self::Omitted => Optional::Omitted,
            Self::Present(Nullable::Null) => Optional::Present(Nullable::Null),
            Self::Present(Nullable::NonNull(v)) => Optional::Present(Nullable::NonNull(f(v))),
        }
    }
}

impl<T> IntoIterator for Optional<T> {
    type IntoIter = OptionIntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.into_option().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Optional<T> {
    type IntoIter = OptionIntoIter<&'a T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Optional<T> {
    type IntoIter = OptionIntoIter<&'a mut T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: Display> Display for Optional<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Omitted => write!(f, "omitted"),
            Self::Present(v) => write!(f, "{v}"),
        }
    }
}

impl<T> From<Option<T>> for Optional<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => Self::Present(v),
            None => Self::Omitted,
        }
    }
}

impl<T> From<Optional<T>> for Option<T> {
    fn from(opt: Optional<T>) -> Self {
        opt.into_option()
    }
}

impl<T> From<T> for Optional<T> {
    fn from(val: T) -> Self {
        Self::Present(val)
    }
}

impl From<&str> for Optional<String> {
    #[inline]
    fn from(val: &str) -> Self {
        Self::Present(val.to_owned())
    }
}

impl From<&[u8]> for Optional<Bytes> {
    #[inline]
    fn from(val: &[u8]) -> Self {
        Self::Present(Bytes::copy_from_slice(val))
    }
}

/// Explicit representation of Matter nullability (value may be `AttributeValue::Null`).
///
/// In Matter's Data Model, nullability indicates that a value on the wire may take the
/// special `Null` sentinel. `Nullable<T>` represents nullable attributes, struct fields,
/// command arguments, and event fields.
///
/// # Serde Usage
///
/// Standalone `Nullable<T>` fields serialize `Null` as `null` and `NonNull(v)` as `v`.
/// When nested inside [`Optional`] (`Optional<Nullable<T>>`), add
/// `#[serde(default, skip_serializing_if = "Optional::is_omitted", deserialize_with =
/// "matter_data_model::serde::deserialize_optional_nullable")]` on the field so `null` is preserved as
/// `Optional::Present(Nullable::Null)` instead of `Optional::Omitted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Nullable<T> {
    /// Wire `Null` sentinel value.
    #[default]
    Null,
    /// Non-null value `T`.
    NonNull(T),
}

impl<T> Nullable<T> {
    /// Return `true` if this value is [`Nullable::Null`].
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Return `true` if this value is [`Nullable::NonNull`].
    #[must_use]
    pub const fn is_nonnull(&self) -> bool {
        matches!(self, Self::NonNull(_))
    }

    /// Return `true` if this value is [`Nullable::NonNull`] and the contained value satisfies `f`.
    #[must_use]
    pub fn is_nonnull_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
        matches!(self, Self::NonNull(v) if f(v))
    }

    /// Return `true` if this value is [`Nullable::Null`] or the contained value satisfies `f`.
    #[must_use]
    pub fn is_null_or(&self, f: impl FnOnce(&T) -> bool) -> bool {
        match self {
            Self::Null => true,
            Self::NonNull(v) => f(v),
        }
    }

    /// Convert from `&Nullable<T>` to `Nullable<&T>`.
    #[must_use]
    pub const fn as_ref(&self) -> Nullable<&T> {
        match self {
            Self::Null => Nullable::Null,
            Self::NonNull(v) => Nullable::NonNull(v),
        }
    }

    /// Convert from `&mut Nullable<T>` to `Nullable<&mut T>`.
    #[must_use]
    pub fn as_mut(&mut self) -> Nullable<&mut T> {
        match self {
            Self::Null => Nullable::Null,
            Self::NonNull(v) => Nullable::NonNull(v),
        }
    }

    /// Map a `Nullable<T>` to `Nullable<U>` by applying `f` to a non-null value.
    #[must_use]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Nullable<U> {
        match self {
            Self::Null => Nullable::Null,
            Self::NonNull(v) => Nullable::NonNull(f(v)),
        }
    }

    /// Call `f` with a reference to the non-null value (if any), then return `self`.
    #[must_use]
    pub fn inspect(self, f: impl FnOnce(&T)) -> Self {
        if let Self::NonNull(v) = &self {
            f(v);
        }
        self
    }

    /// Return [`Nullable::Null`] if null, otherwise return `optb`.
    #[must_use]
    pub fn and<U>(self, optb: Nullable<U>) -> Nullable<U> {
        match self {
            Self::Null => Nullable::Null,
            Self::NonNull(_) => optb,
        }
    }

    /// Return [`Nullable::Null`] if null, otherwise call `f` with the non-null value.
    #[must_use]
    pub fn and_then<U>(self, f: impl FnOnce(T) -> Nullable<U>) -> Nullable<U> {
        match self {
            Self::Null => Nullable::Null,
            Self::NonNull(v) => f(v),
        }
    }

    /// Zip `self` with `other`, returning `NonNull((a, b))` if both are `NonNull`, or `Null`.
    #[must_use]
    pub fn zip<U>(self, other: Nullable<U>) -> Nullable<(T, U)> {
        match (self, other) {
            (Self::NonNull(a), Nullable::NonNull(b)) => Nullable::NonNull((a, b)),
            _ => Nullable::Null,
        }
    }

    /// Convert into `Option<T>`, mapping [`Nullable::NonNull`] to `Some` and [`Nullable::Null`] to `None`.
    #[must_use]
    pub fn into_option(self) -> Option<T> {
        match self {
            Self::Null => None,
            Self::NonNull(v) => Some(v),
        }
    }

    /// Borrow as `Option<&T>`.
    #[must_use]
    pub const fn as_option(&self) -> Option<&T> {
        match self {
            Self::Null => None,
            Self::NonNull(v) => Some(v),
        }
    }

    /// Unwrap a non-null value, panicking if [`Nullable::Null`].
    #[expect(clippy::panic, reason = "Explicit unwrap panics by design on Null")]
    pub fn unwrap(self) -> T {
        match self {
            Self::NonNull(v) => v,
            Self::Null => panic!("called `Nullable::unwrap()` on a `Null` value"),
        }
    }

    /// Unwrap a non-null value, panicking with `msg` if [`Nullable::Null`].
    #[expect(clippy::panic, reason = "Explicit expect panics by design on Null")]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Self::NonNull(v) => v,
            Self::Null => panic!("{msg}"),
        }
    }

    /// Return `default` if null, or apply `f` to the non-null value.
    #[must_use]
    pub fn map_or<U>(self, default: U, f: impl FnOnce(T) -> U) -> U {
        match self {
            Self::Null => default,
            Self::NonNull(v) => f(v),
        }
    }

    /// Compute a default from `default` if null, or apply `f` to the non-null value.
    #[must_use]
    pub fn map_or_else<U>(self, default: impl FnOnce() -> U, f: impl FnOnce(T) -> U) -> U {
        match self {
            Self::Null => default(),
            Self::NonNull(v) => f(v),
        }
    }

    /// Return `self` if non-null, otherwise return `optb`.
    #[must_use]
    pub fn or(self, optb: Self) -> Self {
        match self {
            Self::NonNull(_) => self,
            Self::Null => optb,
        }
    }

    /// Return `self` if non-null, otherwise call `f` and return the result.
    #[must_use]
    pub fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        match self {
            Self::NonNull(_) => self,
            Self::Null => f(),
        }
    }

    /// Return `NonNull` if exactly one of `self` and `optb` is `NonNull`, otherwise `Null`.
    #[must_use]
    pub fn xor(self, optb: Self) -> Self {
        match (self, optb) {
            (Self::NonNull(a), Self::Null) => Self::NonNull(a),
            (Self::Null, Self::NonNull(b)) => Self::NonNull(b),
            _ => Self::Null,
        }
    }

    /// Insert `value` into the `Nullable`, returning a mutable reference to it.
    #[expect(clippy::unreachable, reason = "Self::NonNull was just assigned on the previous line")]
    pub fn insert(&mut self, value: T) -> &mut T {
        *self = Self::NonNull(value);
        match self {
            Self::NonNull(v) => v,
            Self::Null => unreachable!(),
        }
    }

    /// Insert `value` if null, then return a mutable reference to the contained value.
    #[expect(clippy::unreachable, reason = "Self::NonNull was just assigned if Null")]
    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        if self.is_null() {
            *self = Self::NonNull(value);
        }
        match self {
            Self::NonNull(v) => v,
            Self::Null => unreachable!(),
        }
    }

    /// Insert a value computed by `f` if null, then return a mutable reference to it.
    #[expect(clippy::unreachable, reason = "Self::NonNull was just assigned if Null")]
    pub fn get_or_insert_with(&mut self, f: impl FnOnce() -> T) -> &mut T {
        if self.is_null() {
            *self = Self::NonNull(f());
        }
        match self {
            Self::NonNull(v) => v,
            Self::Null => unreachable!(),
        }
    }

    /// Replace the value in this [`Nullable`], returning the old value.
    #[must_use]
    pub const fn replace(&mut self, value: T) -> Self {
        mem::replace(self, Self::NonNull(value))
    }

    /// Return the contained value or `default` if null.
    #[must_use]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::NonNull(v) => v,
            Self::Null => default,
        }
    }

    /// Return the contained value or `T::default()` if null.
    #[must_use]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Self::NonNull(v) => v,
            Self::Null => T::default(),
        }
    }

    /// Convert into a `Result`, using `err` when null.
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        self.into_option().ok_or(err)
    }

    /// Convert into a `Result`, computing the error from `err` when null.
    pub fn ok_or_else<E>(self, err: impl FnOnce() -> E) -> Result<T, E> {
        self.into_option().ok_or_else(err)
    }

    /// Return the contained value or compute it from `f` if null.
    #[must_use]
    pub fn unwrap_or_else(self, f: impl FnOnce() -> T) -> T {
        match self {
            Self::NonNull(v) => v,
            Self::Null => f(),
        }
    }

    /// Take the value out of this [`Nullable`], leaving [`Nullable::Null`] in its place.
    #[must_use]
    pub const fn take(&mut self) -> Self {
        mem::replace(self, Self::Null)
    }

    /// Reset this [`Nullable`] to [`Nullable::Null`] without returning the previous value.
    pub fn clear(&mut self) {
        *self = Self::Null;
    }

    /// Return [`Nullable::Null`] if null, or call `predicate` on the wrapped value.
    #[must_use]
    pub fn filter(self, predicate: impl FnOnce(&T) -> bool) -> Self {
        match self {
            Self::NonNull(v) if predicate(&v) => Self::NonNull(v),
            _ => Self::Null,
        }
    }

    /// Convert from `&Nullable<T>` to `Nullable<&T::Target>`.
    #[must_use]
    pub fn as_deref(&self) -> Nullable<&T::Target>
    where
        T: Deref,
    {
        match self {
            Self::NonNull(v) => Nullable::NonNull(&**v),
            Self::Null => Nullable::Null,
        }
    }

    /// Convert from `&mut Nullable<T>` to `Nullable<&mut T::Target>`.
    #[must_use]
    pub fn as_deref_mut(&mut self) -> Nullable<&mut T::Target>
    where
        T: DerefMut,
    {
        match self {
            Self::NonNull(v) => Nullable::NonNull(&mut **v),
            Self::Null => Nullable::Null,
        }
    }

    /// Return an iterator over the possibly contained value.
    #[must_use]
    pub fn iter(&self) -> OptionIntoIter<&'_ T> {
        self.as_ref().into_option().into_iter()
    }

    /// Return a mutable iterator over the possibly contained value.
    #[must_use]
    pub fn iter_mut(&mut self) -> OptionIntoIter<&'_ mut T> {
        self.as_mut().into_option().into_iter()
    }
}

impl<T> Nullable<Nullable<T>> {
    /// Convert from `Nullable<Nullable<T>>` to `Nullable<T>`.
    #[must_use]
    pub fn flatten(self) -> Nullable<T> {
        match self {
            Self::NonNull(inner) => inner,
            Self::Null => Nullable::Null,
        }
    }
}

impl<T, E> Nullable<Result<T, E>> {
    /// Transpose a `Nullable<Result<T, E>>` into a `Result<Nullable<T>, E>`.
    pub fn transpose(self) -> Result<Nullable<T>, E> {
        match self {
            Self::Null => Ok(Nullable::Null),
            Self::NonNull(Ok(v)) => Ok(Nullable::NonNull(v)),
            Self::NonNull(Err(e)) => Err(e),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<T: Serialize> Serialize for Nullable<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::NonNull(v) => serializer.serialize_some(v),
            Self::Null => serializer.serialize_none(),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(feature = "schemars")]
#[cfg_attr(docsrs, doc(cfg(feature = "schemars")))]
impl<T: JsonSchema> JsonSchema for Nullable<T> {
    fn schema_name() -> Cow<'static, str> {
        <Option<T>>::schema_name()
    }

    fn schema_id() -> Cow<'static, str> {
        <Option<T>>::schema_id()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <Option<T>>::json_schema(generator)
    }

    fn inline_schema() -> bool {
        <Option<T>>::inline_schema()
    }
}

impl<T: Copy> Nullable<&T> {
    /// Map a `Nullable<&T>` to a `Nullable<T>` by copying the contents.
    #[must_use]
    pub const fn copied(self) -> Nullable<T> {
        match self {
            Self::NonNull(&v) => Nullable::NonNull(v),
            Self::Null => Nullable::Null,
        }
    }
}

impl<T: Clone> Nullable<&T> {
    /// Map a `Nullable<&T>` to a `Nullable<T>` by cloning the contents.
    #[must_use]
    pub fn cloned(self) -> Nullable<T> {
        match self {
            Self::NonNull(v) => Nullable::NonNull(v.clone()),
            Self::Null => Nullable::Null,
        }
    }
}

impl<T> IntoIterator for Nullable<T> {
    type IntoIter = OptionIntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.into_option().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Nullable<T> {
    type IntoIter = OptionIntoIter<&'a T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Nullable<T> {
    type IntoIter = OptionIntoIter<&'a mut T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: Display> Display for Nullable<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Null => write!(f, "null"),
            Self::NonNull(v) => write!(f, "{v}"),
        }
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => Self::NonNull(v),
            None => Self::Null,
        }
    }
}

impl<T> From<Nullable<T>> for Option<T> {
    fn from(nul: Nullable<T>) -> Self {
        nul.into_option()
    }
}

impl<T> From<T> for Nullable<T> {
    fn from(val: T) -> Self {
        Self::NonNull(val)
    }
}

impl From<&str> for Nullable<String> {
    #[inline]
    fn from(val: &str) -> Self {
        Self::NonNull(val.to_owned())
    }
}

impl From<&[u8]> for Nullable<Bytes> {
    #[inline]
    fn from(val: &[u8]) -> Self {
        Self::NonNull(Bytes::copy_from_slice(val))
    }
}

impl<T: Into<AttributeValue>> From<Nullable<T>> for AttributeValue {
    fn from(val: Nullable<T>) -> Self {
        match val {
            Nullable::Null => Self::Null,
            Nullable::NonNull(v) => v.into(),
        }
    }
}

impl<T: for<'a> TryFrom<&'a AttributeValue, Error = MatterStatus>> TryFrom<&AttributeValue> for Nullable<T> {
    type Error = MatterStatus;

    fn try_from(val: &AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::Null => Ok(Self::Null),
            other => T::try_from(other).map(Self::NonNull),
        }
    }
}

impl<T: TryFrom<AttributeValue, Error = MatterStatus>> TryFrom<AttributeValue> for Nullable<T> {
    type Error = MatterStatus;

    fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
        match val {
            AttributeValue::Null => Ok(Self::Null),
            other => T::try_from(other).map(Self::NonNull),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_semantics() {
        let opt_omitted: Optional<u32> = Optional::Omitted;
        assert!(opt_omitted.is_omitted());
        assert!(!opt_omitted.is_present());
        assert_eq!(opt_omitted.into_option(), None);
        assert_eq!(alloc::format!("{opt_omitted}"), "omitted");

        let opt_present = Optional::Present(42_u32);
        assert!(opt_present.is_present());
        assert!(!opt_present.is_omitted());
        assert_eq!(opt_present.into_option(), Some(42));
        assert_eq!(opt_present.unwrap(), 42);
        assert_eq!(opt_present.unwrap_or(0), 42);
        assert_eq!(opt_omitted.unwrap_or(7), 7);
        assert_eq!(opt_omitted.unwrap_or_default(), 0);
        assert_eq!(opt_omitted.ok_or("x"), Err("x"));
        assert_eq!(opt_present.ok_or("x"), Ok(42));
        assert_eq!(alloc::format!("{opt_present}"), "42");
        assert_eq!(Optional::from(Some(42_u32)), opt_present);
        assert_eq!(Optional::from(None::<u32>), opt_omitted);
    }

    #[test]
    #[should_panic(expected = "Omitted")]
    fn optional_unwrap_panics_on_omitted() {
        let _: u32 = Optional::<u32>::Omitted.unwrap();
    }

    #[test]
    #[should_panic(expected = "Null")]
    fn nullable_unwrap_panics_on_null() {
        let _: u32 = Nullable::<u32>::Null.unwrap();
    }

    #[test]
    fn nullable_semantics() {
        let null_val: Nullable<u32> = Nullable::Null;
        assert!(null_val.is_null());
        assert!(!null_val.is_nonnull());
        assert_eq!(null_val.into_option(), None);
        assert_eq!(alloc::format!("{null_val}"), "null");
        assert_eq!(AttributeValue::from(null_val), AttributeValue::Null);

        let non_null_val = Nullable::NonNull(100_u32);
        assert!(non_null_val.is_nonnull());
        assert!(!non_null_val.is_null());
        assert_eq!(non_null_val.into_option(), Some(100));
        assert_eq!(non_null_val.unwrap(), 100);
        assert_eq!(non_null_val.unwrap_or(0), 100);
        assert_eq!(null_val.unwrap_or(7), 7);
        assert_eq!(null_val.unwrap_or_default(), 0);
        assert_eq!(null_val.ok_or("x"), Err("x"));
        assert_eq!(non_null_val.ok_or("x"), Ok(100));
        assert_eq!(alloc::format!("{non_null_val}"), "100");
        assert_eq!(AttributeValue::from(non_null_val), AttributeValue::U32(100));

        let decoded_null = Nullable::<u32>::try_from(&AttributeValue::Null).unwrap();
        assert_eq!(decoded_null, Nullable::Null);
        let decoded_val = Nullable::<u32>::try_from(&AttributeValue::U32(50)).unwrap();
        assert_eq!(decoded_val, Nullable::NonNull(50));

        let decoded_owned = Nullable::<u32>::try_from(AttributeValue::U32(50)).unwrap();
        assert_eq!(decoded_owned, Nullable::NonNull(50));
    }

    #[test]
    fn optional_nullable_combinators_and_iteration() {
        let opt_null = Optional::<Nullable<u8>>::present_nonnull(42);
        assert_eq!(opt_null.as_nonnull(), Some(&42));
        assert_eq!(opt_null.into_nonnull(), Some(42));
        assert_eq!(Optional::<Nullable<u8>>::present_null().as_nonnull(), None);

        let val = 10_u8;
        assert_eq!(Optional::Present(&val).copied(), Optional::Present(10));
        assert_eq!(Nullable::NonNull(&val).copied(), Nullable::NonNull(10));

        let s = String::from("hi");
        assert_eq!(Optional::Present(s.clone()).as_deref(), Optional::Present("hi"));
        assert_eq!(Nullable::NonNull(s).as_deref(), Nullable::NonNull("hi"));

        assert_eq!(Optional::<u8>::Omitted.unwrap_or_else(|| 7), 7);
        assert_eq!(Nullable::<u8>::Null.unwrap_or_else(|| 8), 8);

        let mut opt = Optional::Present(5);
        assert_eq!(opt.take(), Optional::Present(5));
        assert!(opt.is_omitted());
        assert_eq!(Optional::Present(5).filter(|&x| x > 3), Optional::Present(5));
        assert_eq!(Optional::Present(1).filter(|&x| x > 3), Optional::Omitted);

        let mut sum = 0;
        for &x in &Optional::Present(3) {
            sum += x;
        }
        for &x in &Nullable::NonNull(4) {
            sum += x;
        }
        assert_eq!(sum, 7);

        // Extended Option-like combinators
        assert_eq!(Optional::Present(5).expect("present"), 5);
        assert_eq!(Nullable::NonNull(5).expect("value"), 5);
        assert!(Optional::Present(5).is_present_and(|&x| x == 5));
        assert!(!Optional::<u8>::Omitted.is_present_and(|_| true));
        assert!(Nullable::NonNull(5).is_nonnull_and(|&x| x == 5));
        assert!(!Nullable::<u8>::Null.is_nonnull_and(|_| true));

        assert_eq!(Optional::Present(2).map_or(0, |x| x * 3), 6);
        assert_eq!(Optional::<u8>::Omitted.map_or(9, |x| x * 3), 9);
        assert_eq!(Optional::Present(2).map_or_else(|| 9, |x| x * 3), 6);
        assert_eq!(Nullable::<u8>::Null.map_or_else(|| 9, |x| x * 3), 9);

        assert_eq!(Optional::<u8>::Omitted.or(Optional::Present(7)), Optional::Present(7));
        assert_eq!(Optional::Present(1).xor(Optional::Omitted), Optional::Present(1));
        assert_eq!(Optional::Present(1).xor(Optional::Present(2)), Optional::Omitted);
        assert_eq!(
            Nullable::<u8>::Null.or_else(|| Nullable::NonNull(8)),
            Nullable::NonNull(8)
        );
        assert_eq!(Nullable::NonNull(1).xor(Nullable::Null), Nullable::NonNull(1));

        let mut opt_ins = Optional::<u8>::Omitted;
        assert_eq!(*opt_ins.get_or_insert(10), 10);
        assert_eq!(*opt_ins.get_or_insert_with(|| 99), 10);
        assert_eq!(opt_ins.replace(20), Optional::Present(10));
        assert_eq!(*opt_ins.insert(30), 30);

        let mut null_ins = Nullable::<u8>::Null;
        assert_eq!(*null_ins.get_or_insert_with(|| 11), 11);
        assert_eq!(null_ins.replace(22), Nullable::NonNull(11));
        assert_eq!(*null_ins.insert(33), 33);

        let mut opt_s = Optional::Present(String::from("ab"));
        assert!(opt_s.as_deref_mut().is_present());
        let mut null_s = Nullable::NonNull(String::from("cd"));
        assert!(null_s.as_deref_mut().is_nonnull());

        assert_eq!(Optional::Present(Optional::Present(4)).flatten(), Optional::Present(4));
        assert_eq!(Nullable::NonNull(Nullable::Null::<u8>).flatten(), Nullable::Null);
        assert_eq!(
            Optional::<Result<u8, &str>>::Present(Ok(1)).transpose(),
            Ok(Optional::Present(1))
        );
        assert_eq!(Nullable::<Result<u8, &str>>::NonNull(Err("e")).transpose(), Err("e"));

        // New Option-parity & Optional<Nullable<T>> helpers
        assert!(Optional::<u8>::Omitted.is_omitted_or(|_| false));
        assert!(Optional::Present(4_u8).is_omitted_or(|&x| x == 4));
        assert!(Nullable::<u8>::Null.is_null_or(|_| false));
        assert!(Nullable::NonNull(4_u8).is_null_or(|&x| x == 4));
        assert_eq!(
            Optional::Present(1_u8).and(Optional::Present(2_u8)),
            Optional::Present(2)
        );
        assert_eq!(Nullable::NonNull(1_u8).and(Nullable::<u8>::Null), Nullable::Null);
        assert_eq!(
            Optional::Present(1_u8).zip(Optional::Present("a")),
            Optional::Present((1, "a"))
        );
        assert_eq!(Nullable::NonNull(1_u8).zip(Nullable::<&str>::Null), Nullable::Null);
        assert_eq!(Optional::<u8>::Omitted.ok_or_else(|| "missing"), Err("missing"));
        assert_eq!(Nullable::<u8>::Null.ok_or_else(|| "null"), Err("null"));
        let mut seen = 0_u8;
        assert_eq!(Optional::Present(5_u8).inspect(|&v| seen = v), Optional::Present(5));
        assert_eq!(seen, 5);
        assert_eq!(Nullable::NonNull(9_u8).inspect(|&v| seen = v), Nullable::NonNull(9));
        assert_eq!(seen, 9);

        let mut on = Optional::<Nullable<u8>>::present_nonnull(7);
        assert!(on.is_nonnull());
        assert!(!on.is_null());
        if let Some(v) = on.as_nonnull_mut() {
            *v = 8;
        }
        assert_eq!(on.as_nonnull(), Some(&8));
        assert!(Optional::<Nullable<u8>>::present_null().is_null());

        // clear() resets without triggering unused_must_use
        let mut to_clear = Optional::Present(1_u8);
        to_clear.clear();
        assert!(to_clear.is_omitted());
        let mut nul_clear = Nullable::NonNull(1_u8);
        nul_clear.clear();
        assert!(nul_clear.is_null());

        // Optional<Nullable<T>> inherent helpers across Omitted, Present(Null), Present(NonNull(_))
        let omitted: Optional<Nullable<u8>> = Optional::Omitted;
        assert_eq!(omitted.as_nonnull(), None);
        assert!(!omitted.is_null());
        assert!(!omitted.is_nonnull());
        assert_eq!(omitted.map_nonnull(|v| v + 1), Optional::Omitted);
        assert_eq!(omitted.into_nonnull(), None);

        let null_val: Optional<Nullable<u8>> = Optional::Present(Nullable::Null);
        assert_eq!(null_val.as_nonnull(), None);
        assert!(null_val.is_null());
        assert!(!null_val.is_nonnull());
        assert_eq!(null_val.map_nonnull(|v| v + 1), Optional::Present(Nullable::Null));
        assert_eq!(null_val.into_nonnull(), None);

        let present: Optional<Nullable<u8>> = Optional::Present(Nullable::NonNull(41));
        assert_eq!(present.as_nonnull(), Some(&41));
        assert!(!present.is_null());
        assert!(present.is_nonnull());
        assert_eq!(present.map_nonnull(|v| v + 1), Optional::Present(Nullable::NonNull(42)));
        assert_eq!(present.into_nonnull(), Some(41));
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn optional_and_nullable_json_schema() {
        let mut generator = SchemaGenerator::default();
        let opt_schema = <Optional<u32>>::json_schema(&mut generator);
        let u32_schema = u32::json_schema(&mut generator);
        assert_eq!(opt_schema, u32_schema);

        let nul_schema = <Nullable<u32>>::json_schema(&mut generator);
        let opt_nul_schema = <Optional<Nullable<u32>>>::json_schema(&mut generator);
        assert_eq!(nul_schema, <Option<u32>>::json_schema(&mut generator));
        assert_eq!(opt_nul_schema, nul_schema);
    }
}
