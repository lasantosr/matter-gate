//! Lossless wire wrappers for Matter clusters and events.
//!
//! These types pair typed domain payloads with instance-level protocol metadata
//! (`data_version`, [`GlobalAttributes`], `event_number`, `timestamp`, and
//! manufacturer-specific `extra_attributes`) without bloating lean domain structs.

mod cluster;
mod event;
mod global_attributes;

#[cfg(test)]
mod tests;

pub use cluster::WireCluster;
pub use event::WireEvent;
pub use global_attributes::GlobalAttributes;
#[cfg(all(feature = "dispatch", feature = "serde"))]
use serde::{Deserializer, de::DeserializeOwned};

#[cfg(all(feature = "dispatch", feature = "serde"))]
use crate::dispatch::MatterDescribe;

/// Deserializes a self-describing JSON envelope when `T` has no static scope metadata.
#[cfg(all(feature = "dispatch", feature = "serde"))]
fn deserialize_envelope_described<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned + MatterDescribe,
    D: Deserializer<'de>,
{
    use alloc::string::String;
    use core::{fmt, marker::PhantomData};

    use serde::de::{self, MapAccess, Visitor};

    struct EnvelopeVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for EnvelopeVisitor<T>
    where
        T: DeserializeOwned + MatterDescribe,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a self-describing Matter envelope object")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut payload: Option<T> = None;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "payload" if payload.is_some() => {
                        return Err(de::Error::duplicate_field("payload"));
                    }
                    "payload" => {
                        payload = Some(map.next_value::<T>()?);
                    }
                    "cluster" | "cluster_id" | "event" | "event_id" => {
                        let _ignored: de::IgnoredAny = map.next_value::<de::IgnoredAny>()?;
                    }
                    #[cfg(feature = "deny-unknown-fields")]
                    other => {
                        return Err(de::Error::unknown_field(
                            other,
                            &["cluster", "cluster_id", "event", "event_id", "payload"],
                        ));
                    }
                    #[cfg(not(feature = "deny-unknown-fields"))]
                    _ => {
                        let _ignored_val: de::IgnoredAny = map.next_value::<de::IgnoredAny>()?;
                    }
                }
            }
            payload.ok_or_else(|| de::Error::missing_field("payload"))
        }
    }

    deserializer.deserialize_map(EnvelopeVisitor(PhantomData))
}
