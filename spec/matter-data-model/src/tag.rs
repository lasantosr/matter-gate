//! Semantic tags per CSA Matter Core Specification §9.5.4.4 (`SemanticTagStruct` in Descriptor `TagList`).

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;

use crate::{
    optional::{Nullable, Optional},
    status::MatterStatus,
    types::VendorId,
    value::{AttributeValue, FieldId},
};

/// Matter semantic tag per Core Specification §9.5.4.4 (`SemanticTagStruct`).
///
/// Used to disambiguate composed multi-gang endpoints and multi-room accessories in controllers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct SemanticTag {
    /// Manufacturer vendor ID for vendor-specific namespaces, or `Nullable::Null` for standard namespaces (field ID
    /// `0`).
    pub mfg_code: Nullable<VendorId>,
    /// Common or vendor-specific semantic tag namespace ID (field ID `1`).
    pub namespace_id: u8,
    /// Tag value within `namespace_id` (`enum8` on the wire, field ID `2`).
    pub tag: u8,
    /// Optional human-readable label, max 64 bytes (field ID `3`).
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Optional::is_omitted",
            deserialize_with = "crate::serde::deserialize_optional_nullable"
        )
    )]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub label: Optional<Nullable<String>>,
}

impl SemanticTag {
    /// Construct a new standard-namespace [`SemanticTag`] with `mfg_code = Null` and `label = Omitted`.
    #[must_use]
    pub const fn new(namespace_id: u8, tag: u8) -> Self {
        Self {
            mfg_code: Nullable::Null,
            namespace_id,
            tag,
            label: Optional::Omitted,
        }
    }

    /// Set a manufacturer vendor ID on this semantic tag.
    #[must_use]
    pub const fn with_mfg_code(mut self, mfg_code: VendorId) -> Self {
        self.mfg_code = Nullable::NonNull(mfg_code);
        self
    }

    /// Set a nullable manufacturer vendor ID on this semantic tag.
    #[must_use]
    pub fn with_nullable_mfg_code(mut self, mfg_code: impl Into<Nullable<VendorId>>) -> Self {
        self.mfg_code = mfg_code.into();
        self
    }

    /// Attach a human-readable label (or `Nullable::Null`) to this semantic tag.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<Nullable<String>>) -> Self {
        self.label = Optional::Present(label.into());
        self
    }

    /// Set or omit the optional human-readable label on this semantic tag.
    #[must_use]
    pub fn with_optional_label(mut self, label: impl Into<Optional<Nullable<String>>>) -> Self {
        self.label = label.into();
        self
    }
}

impl fmt::Display for SemanticTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ns={}, tag={}", self.namespace_id, self.tag)?;
        if let Nullable::NonNull(mfg) = self.mfg_code {
            write!(f, ", mfg={mfg}")?;
        }
        if let Optional::Present(Nullable::NonNull(label)) = &self.label {
            write!(f, ", label={label:?}")?;
        }
        Ok(())
    }
}

impl From<SemanticTag> for AttributeValue {
    fn from(tag: SemanticTag) -> Self {
        let mut fields = Vec::with_capacity(4);
        let mfg_val = match tag.mfg_code {
            Nullable::Null => Self::Null,
            Nullable::NonNull(v) => Self::U16(v.raw()),
        };
        fields.push((FieldId::new(0), mfg_val));
        fields.push((FieldId::new(1), Self::U8(tag.namespace_id)));
        fields.push((FieldId::new(2), Self::U8(tag.tag)));
        if let Optional::Present(label) = tag.label {
            let label_val = match label {
                Nullable::Null => Self::Null,
                Nullable::NonNull(s) => Self::String(s),
            };
            fields.push((FieldId::new(3), label_val));
        }
        Self::Struct(fields)
    }
}

impl TryFrom<&AttributeValue> for SemanticTag {
    type Error = MatterStatus;

    fn try_from(value: &AttributeValue) -> Result<Self, Self::Error> {
        let AttributeValue::Struct(fields) = value else {
            return Err(MatterStatus::InvalidDataType);
        };

        let mut mfg_code: Option<Nullable<VendorId>> = None;
        let mut namespace_id: Option<u8> = None;
        let mut tag: Option<u8> = None;
        let mut label: Optional<Nullable<String>> = Optional::Omitted;
        let mut seen_unknown: Vec<FieldId> = Vec::new();

        for (id, val) in fields {
            match id.raw() {
                0 => {
                    if mfg_code.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    mfg_code = Some(match val {
                        AttributeValue::Null => Nullable::Null,
                        AttributeValue::U16(v) => Nullable::NonNull(VendorId::new(*v)),
                        _ => return Err(MatterStatus::InvalidDataType),
                    });
                }
                1 => {
                    if namespace_id.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    let AttributeValue::U8(ns) = val else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    namespace_id = Some(*ns);
                }
                2 => {
                    if tag.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    let AttributeValue::U8(t) = val else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    tag = Some(*t);
                }
                3 => {
                    if label.is_present() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    label = Optional::Present(parse_label(val)?);
                }
                _ => {
                    if seen_unknown.contains(id) {
                        return Err(MatterStatus::ConstraintError);
                    }
                    seen_unknown.push(*id);
                }
            }
        }

        Ok(Self {
            mfg_code: mfg_code.ok_or(MatterStatus::ConstraintError)?,
            namespace_id: namespace_id.ok_or(MatterStatus::ConstraintError)?,
            tag: tag.ok_or(MatterStatus::ConstraintError)?,
            label,
        })
    }
}

fn parse_label(val: &AttributeValue) -> Result<Nullable<String>, MatterStatus> {
    match val {
        AttributeValue::Null => Ok(Nullable::Null),
        AttributeValue::String(s) => Ok(Nullable::NonNull(s.clone())),
        _ => Err(MatterStatus::InvalidDataType),
    }
}

fn parse_label_owned(val: AttributeValue) -> Result<Nullable<String>, MatterStatus> {
    match val {
        AttributeValue::Null => Ok(Nullable::Null),
        AttributeValue::String(s) => Ok(Nullable::NonNull(s)),
        _ => Err(MatterStatus::InvalidDataType),
    }
}

impl TryFrom<AttributeValue> for SemanticTag {
    type Error = MatterStatus;

    fn try_from(value: AttributeValue) -> Result<Self, Self::Error> {
        let AttributeValue::Struct(fields) = value else {
            return Err(MatterStatus::InvalidDataType);
        };

        let mut mfg_code: Option<Nullable<VendorId>> = None;
        let mut namespace_id: Option<u8> = None;
        let mut tag: Option<u8> = None;
        let mut label: Optional<Nullable<String>> = Optional::Omitted;
        let mut seen_unknown: Vec<FieldId> = Vec::new();

        for (id, val) in fields {
            match id.raw() {
                0 => {
                    if mfg_code.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    mfg_code = Some(match val {
                        AttributeValue::Null => Nullable::Null,
                        AttributeValue::U16(v) => Nullable::NonNull(VendorId::new(v)),
                        _ => return Err(MatterStatus::InvalidDataType),
                    });
                }
                1 => {
                    if namespace_id.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    let AttributeValue::U8(ns) = val else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    namespace_id = Some(ns);
                }
                2 => {
                    if tag.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    let AttributeValue::U8(t) = val else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    tag = Some(t);
                }
                3 => {
                    if label.is_present() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    label = Optional::Present(parse_label_owned(val)?);
                }
                _ => {
                    if seen_unknown.contains(&id) {
                        return Err(MatterStatus::ConstraintError);
                    }
                    seen_unknown.push(id);
                }
            }
        }

        Ok(Self {
            mfg_code: mfg_code.ok_or(MatterStatus::ConstraintError)?,
            namespace_id: namespace_id.ok_or(MatterStatus::ConstraintError)?,
            tag: tag.ok_or(MatterStatus::ConstraintError)?,
            label,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::*;

    #[test]
    fn semantic_tag_wire_roundtrip() {
        let tag = SemanticTag::new(1, 2)
            .with_mfg_code(VendorId::new(0xFFF1))
            .with_label("Left");
        assert_eq!(tag.mfg_code, Nullable::NonNull(VendorId::new(0xFFF1)));
        assert_eq!(tag.namespace_id, 1);
        assert_eq!(tag.tag, 2);
        assert_eq!(tag.label, Optional::Present(Nullable::NonNull(String::from("Left"))));

        let val = AttributeValue::from(tag.clone());
        assert_eq!(SemanticTag::try_from(&val), Ok(tag.clone()));
        assert_eq!(SemanticTag::try_from(val), Ok(tag.clone()));

        let omitted_label = tag.with_optional_label(Optional::Omitted);
        assert_eq!(omitted_label.label, Optional::Omitted);
    }

    #[test]
    fn semantic_tag_without_optional_fields() {
        let tag = SemanticTag::new(3, 4);
        assert_eq!(tag.mfg_code, Nullable::Null);
        assert_eq!(tag.label, Optional::Omitted);

        let val = AttributeValue::from(tag.clone());
        assert_eq!(SemanticTag::try_from(&val), Ok(tag));
    }

    #[test]
    fn semantic_tag_rejects_missing_or_duplicate_fields_and_ignores_unknown() {
        let missing_mfg = AttributeValue::Struct(alloc::vec![
            (FieldId::new(1), AttributeValue::U8(3)),
            (FieldId::new(2), AttributeValue::U8(4)),
        ]);
        assert_eq!(SemanticTag::try_from(&missing_mfg), Err(MatterStatus::ConstraintError));

        let unknown_field = AttributeValue::Struct(alloc::vec![
            (FieldId::new(0), AttributeValue::Null),
            (FieldId::new(1), AttributeValue::U8(3)),
            (FieldId::new(2), AttributeValue::U8(4)),
            (FieldId::new(99), AttributeValue::Bool(true)),
        ]);
        assert_eq!(SemanticTag::try_from(&unknown_field), Ok(SemanticTag::new(3, 4)));

        let duplicate_unknown = AttributeValue::Struct(alloc::vec![
            (FieldId::new(0), AttributeValue::Null),
            (FieldId::new(1), AttributeValue::U8(3)),
            (FieldId::new(2), AttributeValue::U8(4)),
            (FieldId::new(99), AttributeValue::Bool(true)),
            (FieldId::new(99), AttributeValue::Bool(false)),
        ]);
        assert_eq!(
            SemanticTag::try_from(&duplicate_unknown),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(
            SemanticTag::try_from(duplicate_unknown),
            Err(MatterStatus::ConstraintError)
        );

        let duplicate_field = AttributeValue::Struct(alloc::vec![
            (FieldId::new(0), AttributeValue::Null),
            (FieldId::new(0), AttributeValue::U16(0xFFF1)),
            (FieldId::new(1), AttributeValue::U8(3)),
            (FieldId::new(2), AttributeValue::U8(4)),
        ]);
        assert_eq!(
            SemanticTag::try_from(&duplicate_field),
            Err(MatterStatus::ConstraintError)
        );
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn semantic_tag_json_schema_marks_label_optional() {
        let mut generator = schemars::SchemaGenerator::default();
        let schema = <SemanticTag as schemars::JsonSchema>::json_schema(&mut generator);
        let required = schema
            .as_object()
            .and_then(|o| o.get("required"))
            .and_then(|v| v.as_array());
        assert!(
            required.is_some_and(|req| !req.iter().any(|v| v.as_str() == Some("label"))),
            "label must not be required in SemanticTag JSON schema"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_optional_nullable_preserves_present_null() {
        use serde::{
            Deserializer,
            de::{IntoDeserializer as _, Visitor, value::Error as DeValueError},
            forward_to_deserialize_any,
        };

        use crate::serde::deserialize_optional_nullable;

        struct MockOptionDe(Option<&'static str>);

        impl<'de> Deserializer<'de> for MockOptionDe {
            type Error = DeValueError;

            forward_to_deserialize_any! {
                bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
                bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
                map struct enum identifier ignored_any
            }

            fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
                match self.0 {
                    None => visitor.visit_none(),
                    Some(s) => visitor.visit_str(s),
                }
            }

            fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
                match self.0 {
                    None => visitor.visit_none(),
                    Some(s) => visitor.visit_some(s.into_deserializer()),
                }
            }
        }

        let null_res: Optional<Nullable<String>> = deserialize_optional_nullable(MockOptionDe(None)).unwrap();
        assert_eq!(null_res, Optional::Present(Nullable::Null));

        let str_res: Optional<Nullable<String>> = deserialize_optional_nullable(MockOptionDe(Some("Left"))).unwrap();
        assert_eq!(str_res, Optional::Present(Nullable::NonNull(String::from("Left"))));
    }

    #[test]
    fn semantic_tag_ordering() {
        let a = SemanticTag::new(1, 0);
        let b = SemanticTag::new(1, 1);
        let c = SemanticTag::new(2, 0);
        assert!(a < b);
        assert!(b < c);
    }
}
