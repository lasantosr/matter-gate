//! Serde deserialization helpers.

use ::serde::{Deserialize, Deserializer};

use crate::optional::{Nullable, Optional};

/// Deserialize an `Optional<Nullable<T>>` field without collapsing `null` into `Optional::Omitted`.
///
/// Pair with `#[serde(default, skip_serializing_if = "Optional::is_omitted", deserialize_with =
/// "matter_data_model::serde::deserialize_optional_nullable")]`:
/// - absent key → `Optional::Omitted`
/// - `null` value → `Optional::Present(Nullable::Null)`
/// - non-null value → `Optional::Present(Nullable::NonNull(v))`
pub fn deserialize_optional_nullable<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Optional<Nullable<T>>, D::Error> {
    Nullable::<T>::deserialize(deserializer).map(Optional::Present)
}
