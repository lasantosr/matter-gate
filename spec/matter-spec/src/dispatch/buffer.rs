//! Minimal `#![no_std]` + `alloc` serde value buffer for out-of-order enum payload deserialization.

use alloc::{
    format,
    string::String,
    vec::{IntoIter, Vec},
};
use core::fmt;

use serde::{
    Deserialize, Deserializer,
    de::{
        self, DeserializeSeed, EnumAccess, IntoDeserializer as _, MapAccess, SeqAccess, VariantAccess, Visitor,
        value::Error as ValueError,
    },
    forward_to_deserialize_any,
};

#[derive(Debug, Clone)]
pub(crate) enum BufferedValue {
    Unit,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Str(String),
    Bytes(Vec<u8>),
    Seq(Vec<Self>),
    Map(Vec<(Self, Self)>),
}

impl<'de> Deserialize<'de> for BufferedValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BufferedVisitor;

        impl<'de> Visitor<'de> for BufferedVisitor {
            type Value = BufferedValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("any valid JSON/Matter value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(BufferedValue::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(BufferedValue::I64(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(BufferedValue::U64(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(BufferedValue::F64(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(BufferedValue::Str(String::from(v)))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
                Ok(BufferedValue::Str(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(BufferedValue::Bytes(v.to_vec()))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                Ok(BufferedValue::Bytes(v))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(BufferedValue::Unit)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(BufferedValue::Unit)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                BufferedValue::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut items = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(item) = seq.next_element()? {
                    items.push(item);
                }
                Ok(BufferedValue::Seq(items))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((k, v)) = map.next_entry()? {
                    entries.push((k, v));
                }
                Ok(BufferedValue::Map(entries))
            }
        }

        deserializer.deserialize_any(BufferedVisitor)
    }
}

impl<'de> Deserializer<'de> for BufferedValue {
    type Error = ValueError;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf seq tuple tuple_struct map identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Unit => visitor.visit_unit(),
            Self::Bool(v) => visitor.visit_bool(v),
            Self::I64(v) => visitor.visit_i64(v),
            Self::U64(v) => visitor.visit_u64(v),
            Self::F64(v) => visitor.visit_f64(v),
            Self::Str(v) => visitor.visit_string(v),
            Self::Bytes(v) => visitor.visit_byte_buf(v),
            Self::Seq(items) => visitor.visit_seq(BufferedSeq::new(items)),
            Self::Map(entries) => visitor.visit_map(BufferedMap::new(entries)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Unit => visitor.visit_none(),
            other => visitor.visit_some(other),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Unit => visitor.visit_unit(),
            Self::Map(entries) if entries.is_empty() => visitor.visit_unit(),
            other => other.deserialize_any(visitor),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Unit if fields.is_empty() => visitor.visit_map(BufferedMap::new(Vec::new())),
            other => other.deserialize_any(visitor),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Str(variant) => visitor.visit_enum(BufferedEnum::Unit(variant)),
            Self::Map(mut entries) => {
                if entries.len() != 1 {
                    return Err(de::Error::invalid_length(
                        entries.len(),
                        &"map with a single variant entry",
                    ));
                }
                let (key, val) = entries.remove(0);
                let Self::Str(variant) = key else {
                    return Err(de::Error::custom("enum variant key must be a string"));
                };
                visitor.visit_enum(BufferedEnum::Newtype(variant, val))
            }
            other => Err(de::Error::custom(format!("expected enum string or map, got {other:?}"))),
        }
    }
}

struct BufferedSeq {
    iter: IntoIter<BufferedValue>,
}

impl BufferedSeq {
    fn new(items: Vec<BufferedValue>) -> Self {
        Self {
            iter: items.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for BufferedSeq {
    type Error = ValueError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(val) => seed.deserialize(val).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct BufferedMap {
    iter: IntoIter<(BufferedValue, BufferedValue)>,
    pending_value: Option<BufferedValue>,
}

impl BufferedMap {
    fn new(entries: Vec<(BufferedValue, BufferedValue)>) -> Self {
        Self {
            iter: entries.into_iter(),
            pending_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for BufferedMap {
    type Error = ValueError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((k, v)) => {
                self.pending_value = Some(v);
                seed.deserialize(k).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.pending_value.take() {
            Some(v) => seed.deserialize(v),
            None => Err(de::Error::custom("next_value_seed called before next_key_seed")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

enum BufferedEnum {
    Unit(String),
    Newtype(String, BufferedValue),
}

impl<'de> EnumAccess<'de> for BufferedEnum {
    type Error = ValueError;
    type Variant = BufferedVariant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self {
            Self::Unit(name) => {
                let val = seed.deserialize(name.into_deserializer())?;
                Ok((val, BufferedVariant::Unit))
            }
            Self::Newtype(name, value) => {
                let val = seed.deserialize(name.into_deserializer())?;
                Ok((val, BufferedVariant::Newtype(value)))
            }
        }
    }
}

enum BufferedVariant {
    Unit,
    Newtype(BufferedValue),
}

impl<'de> VariantAccess<'de> for BufferedVariant {
    type Error = ValueError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self {
            Self::Unit | Self::Newtype(BufferedValue::Unit) => Ok(()),
            Self::Newtype(BufferedValue::Map(entries)) if entries.is_empty() => Ok(()),
            Self::Newtype(_) => Err(de::Error::custom("expected unit variant")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self {
            Self::Newtype(val) => seed.deserialize(val),
            Self::Unit => seed.deserialize(BufferedValue::Unit),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Newtype(val) => val.deserialize_seq(visitor),
            Self::Unit => Err(de::Error::custom("expected tuple variant")),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Newtype(val) => val.deserialize_map(visitor),
            Self::Unit => Err(de::Error::custom("expected struct variant")),
        }
    }
}
