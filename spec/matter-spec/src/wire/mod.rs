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
            let mut cluster_name: Option<String> = None;
            let mut cluster_id: Option<u32> = None;
            let mut event_name: Option<String> = None;
            let mut event_id: Option<u32> = None;
            let mut payload: Option<T> = None;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "payload" if payload.is_some() => {
                        return Err(de::Error::duplicate_field("payload"));
                    }
                    "payload" => {
                        payload = Some(map.next_value::<T>()?);
                    }
                    "cluster" if cluster_name.is_some() => {
                        return Err(de::Error::duplicate_field("cluster"));
                    }
                    "cluster" => {
                        cluster_name = Some(map.next_value::<String>()?);
                    }
                    "cluster_id" if cluster_id.is_some() => {
                        return Err(de::Error::duplicate_field("cluster_id"));
                    }
                    "cluster_id" => {
                        cluster_id = Some(map.next_value::<u32>()?);
                    }
                    "event" if event_name.is_some() => {
                        return Err(de::Error::duplicate_field("event"));
                    }
                    "event" => {
                        event_name = Some(map.next_value::<String>()?);
                    }
                    "event_id" if event_id.is_some() => {
                        return Err(de::Error::duplicate_field("event_id"));
                    }
                    "event_id" => {
                        event_id = Some(map.next_value::<u32>()?);
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
            let payload = payload.ok_or_else(|| de::Error::missing_field("payload"))?;
            let desc = payload.descriptor();
            let (expected_scope, expected_scope_id) = desc.scope();
            if let Some(actual) = cluster_name.as_deref()
                && actual != expected_scope
            {
                return Err(de::Error::custom(alloc::format!(
                    "expected cluster '{expected_scope}', got '{actual}'"
                )));
            }
            if let Some(actual) = cluster_id
                && actual != expected_scope_id
            {
                return Err(de::Error::custom(alloc::format!(
                    "expected cluster_id {expected_scope_id}, got {actual}"
                )));
            }
            if let Some((expected_item, expected_item_id)) = desc.item() {
                if let Some(actual) = event_name.as_deref()
                    && actual != expected_item
                {
                    return Err(de::Error::custom(alloc::format!(
                        "expected event '{expected_item}', got '{actual}'"
                    )));
                }
                if let Some(actual) = event_id
                    && actual != expected_item_id
                {
                    return Err(de::Error::custom(alloc::format!(
                        "expected event_id {expected_item_id}, got {actual}"
                    )));
                }
            }
            Ok(payload)
        }
    }

    deserializer.deserialize_map(EnvelopeVisitor(PhantomData))
}
