use std::result::Result as StdResult;

use genco::{quote, tokens::quoted};

use super::context::{PushLine as _, Tokens, TypeContext, doc_comment, format_hex, is_primitive_int};
use crate::{
    error::Error,
    model::{
        AttributeDef, ClusterDef, ParsedBound, ResolvedType, format_dec_with_underscores, is_semantic_boolean_attr,
        lookup_boolean_semantics, parse_bound_for_type, to_upper_camel_case,
    },
};

type Result<T, E = Error> = StdResult<T, E>;

/// Maps semantic wrapper types to the primitive integer their numeric bounds are expressed in,
/// derived from `CHIP_TYPES_MAP`'s `attr_variant`.
///
/// Any other type resolves to `fallback`.
pub(crate) fn bound_primitive_type<'a>(resolved: &'a ResolvedType, fallback: &'a str) -> &'a str {
    let ResolvedType::SemanticType { attr_variant, .. } = resolved else {
        return fallback;
    };
    match *attr_variant {
        "U8" => "u8",
        "U16" => "u16",
        "U32" => "u32",
        "U64" => "u64",
        "I8" => "i8",
        "I16" => "i16",
        "I32" => "i32",
        "I64" => "i64",
        _ => fallback,
    }
}

pub(super) fn emit_attributes(
    out: &mut Tokens,
    cluster: &ClusterDef,
    cluster_struct: &str,
    ctx: &TypeContext<'_>,
) -> Result<()> {
    if cluster.attributes.is_empty() {
        return Ok(());
    }

    let mut attr_tokens_list = Vec::new();
    for attr in &cluster.attributes {
        attr_tokens_list.push(single_attribute_tokens(attr, cluster, cluster_struct, ctx)?);
    }

    let module_tokens: Tokens = quote! {
        pub mod attributes {
            #[allow(unused_imports, reason = "Generated attribute imports")]
            use alloc::{string::String, vec::Vec};

            #[allow(unused_imports, reason = "Generated attribute imports")]
            use crate::{
                model,
                model::{
                    ActionId, AttributeId, AttributeValue, ClusterId, CommandId, DeviceTypeId, EndpointId, EventId,
                    EventNumber, FabricId, FabricIndex, FieldId, GroupId, MatterStatus, NodeId, Nullable, Optional,
                    TransactionId,
                },
                types,
            };
            #[cfg(feature = $(quoted("serde")))]
            #[allow(unused_imports, reason = "Generated attribute imports")]
            use serde::{Deserialize, Serialize};

            $(for attr_tok in attr_tokens_list join ($['\n']) =>
                $attr_tok
            )
        }
    };

    out.append(module_tokens);
    out.push_blank();
    Ok(())
}

pub(super) fn single_attribute_tokens(
    attr: &AttributeDef,
    cluster: &ClusterDef,
    cluster_struct: &str,
    ctx: &TypeContext<'_>,
) -> Result<Tokens> {
    let AttributeDef {
        name,
        code,
        type_str,
        entry_type,
        is_array: attr_is_array,
        is_nullable,
        is_writable,
        is_optional,
        min,
        max,
        length,
        description,
        ..
    } = attr;
    let code = *code;
    let is_nullable = *is_nullable;
    let is_writable = *is_writable;
    let is_optional = *is_optional;
    let length = *length;
    let is_bool_type = type_str.eq_ignore_ascii_case("boolean") || type_str.eq_ignore_ascii_case("bool");
    let is_semantic_bool = is_bool_type && is_semantic_boolean_attr(name).is_some();
    let resolved = ctx.resolve_type(&cluster.name, name, type_str, entry_type.as_deref(), *attr_is_array)?;

    let base_type_str = sanitize_attr_type_path(resolved.rust_type_path(true));
    let base_type = base_type_str.as_str();
    let inner_type_str = if is_nullable {
        format!("Nullable<{base_type}>")
    } else {
        base_type.to_owned()
    };
    let inner_type = inner_type_str.as_str();

    let is_copy = resolved.is_copy();
    let can_derive_eq_hash = ctx.can_derive_eq_hash(&resolved);

    let attr_name = name;
    let struct_field = build_struct_field(base_type, inner_type, is_nullable);
    let is_semantic = matches!(&resolved, ResolvedType::SemanticType { .. });
    let raw_primitive = bound_primitive_type(&resolved, base_type);
    let methods = build_attribute_methods(base_type, raw_primitive, is_nullable, is_copy, is_semantic);
    let bool_consts = build_semantic_bool_consts(name, is_bool_type, is_nullable, &resolved);

    let is_array = matches!(&resolved, ResolvedType::Array { .. });
    let decode_body = build_decode_body(
        is_array,
        is_nullable,
        base_type,
        &resolved,
        min.as_deref(),
        max.as_deref(),
        length,
    );

    let attr_id_hex = format_hex(u64::from(code), 4);
    let is_nullable_bool = is_nullable.to_string();
    let is_writable_bool = is_writable.to_string();
    let is_optional_bool = is_optional.to_string();
    let copy_mode = if is_copy { "copy" } else { "owned" };
    let val_mode = if is_array { "list" } else { "scalar" };
    let disp_mode = if is_array {
        "list"
    } else if base_type == "crate::Bytes" {
        "bytes"
    } else if matches!(&resolved, ResolvedType::Struct { .. }) {
        "debug"
    } else {
        "display"
    };
    let prefix = if is_nullable { "nullable, " } else { "" };
    let impl_macro_line = format!(
        "crate::impl_matter_attribute!({prefix}{attr_name}, {base_type}, super::{cluster_struct}, {copy_mode}, \
         {val_mode}, {disp_mode});"
    );
    let wrap_macro_call = impl_macro_line.len() > 108;
    let extra_froms = build_extra_from_tokens(attr_name, is_nullable, is_semantic_bool);

    let tokens: Tokens = quote! {
        $(doc_comment(description))
        #[derive(Debug, Clone $(if is_copy => , Copy), PartialEq $(if can_derive_eq_hash => , Eq, Hash))]
        #[cfg_attr(feature = $(quoted("serde")), derive(Serialize, Deserialize), serde(transparent))]
        #[cfg_attr(feature = $(quoted("schemars")), derive(schemars::JsonSchema), schemars(transparent))]
        pub struct $attr_name($struct_field);

        impl $attr_name {
            pub const ATTRIBUTE_ID: AttributeId = AttributeId::new($attr_id_hex);
            pub const CLUSTER_ID: ClusterId = super::$cluster_struct::CLUSTER_ID;
            pub const NAME: &'static str = $(quoted(attr_name));
            pub const IS_NULLABLE: bool = $is_nullable_bool;
            pub const IS_WRITABLE: bool = $is_writable_bool;
            pub const IS_OPTIONAL: bool = $is_optional_bool;

            $methods
            $(if let Some(consts) = &bool_consts => $consts)

            pub fn try_from_value(val: AttributeValue) -> Result<Self, MatterStatus> {
                $(if is_nullable =>
                    if matches!(val, AttributeValue::Null) {
                        return Ok(Self(Nullable::Null));
                    }
                )
                $(if !is_nullable =>
                    if matches!(val, AttributeValue::Null) {
                        return Err(MatterStatus::InvalidDataType);
                    }
                )
                $decode_body
            }
        }

        $(if wrap_macro_call =>
            crate::impl_matter_attribute!(
                $(if is_nullable => nullable,)
                $attr_name,
                $base_type,
                super::$cluster_struct,
                $copy_mode,
                $val_mode,
                $disp_mode
            );
        )
        $(if !wrap_macro_call =>
            $impl_macro_line
        )
        $extra_froms
    };

    Ok(tokens)
}

fn build_struct_field(base_type: &str, inner_type: &str, is_nullable: bool) -> Tokens {
    if base_type == "crate::Bytes" {
        if is_nullable {
            quote! {
                #[cfg_attr(
                    feature = $(quoted("schemars")),
                    schemars(with = $(quoted("Option<Vec<u8>>")))
                )]
                $inner_type
            }
        } else {
            quote!(#[cfg_attr(feature = $(quoted("schemars")), schemars(with = $(quoted("Vec<u8>"))))] $inner_type)
        }
    } else if base_type == "Vec<crate::Bytes>" {
        if is_nullable {
            quote! {
                #[cfg_attr(
                    feature = $(quoted("schemars")),
                    schemars(with = $(quoted("Option<Vec<Vec<u8>>>")))
                )]
                $inner_type
            }
        } else {
            quote!(#[cfg_attr(feature = $(quoted("schemars")), schemars(with = $(quoted("Vec<Vec<u8>>"))))] $inner_type)
        }
    } else {
        quote!($inner_type)
    }
}

fn build_attribute_methods(
    base_type: &str,
    raw_primitive: &str,
    is_nullable: bool,
    is_copy: bool,
    is_semantic: bool,
) -> Tokens {
    if is_nullable {
        quote! {
            #[inline]
            #[must_use]
            pub fn new(val: Nullable<$base_type>) -> Self {
                Self(val)
            }

            #[inline]
            #[must_use]
            pub fn some(val: impl Into<$base_type>) -> Self {
                Self(Nullable::NonNull(val.into()))
            }

            #[inline]
            #[must_use]
            pub const fn null() -> Self {
                Self(Nullable::Null)
            }

            #[inline]
            #[must_use]
            pub const fn is_null(&self) -> bool {
                self.0.is_null()
            }

            #[inline]
            #[must_use]
            pub fn value(&self) -> Option<&$base_type> {
                self.0.as_option()
            }

            $(if is_semantic && is_copy =>
                #[inline]
                #[must_use]
                pub const fn raw(self) -> Nullable<$raw_primitive> {
                    match self.0 {
                        Nullable::NonNull(v) => Nullable::NonNull(v.raw()),
                        Nullable::Null => Nullable::Null,
                    }
                }
            )
            $(if !is_semantic && is_copy =>
                #[inline]
                #[must_use]
                pub const fn raw(self) -> Nullable<$base_type> {
                    self.0
                }
            )
            $(if !is_copy =>
                #[inline]
                #[must_use]
                pub fn raw(&self) -> Nullable<&$base_type> {
                    self.0.as_ref()
                }
            )
        }
    } else {
        quote! {
            #[inline]
            #[must_use]
            pub fn new(val: impl Into<$base_type>) -> Self {
                Self(val.into())
            }

            $(if !is_semantic && is_copy =>
                #[inline]
                #[must_use]
                pub const fn raw(self) -> $base_type {
                    self.0
                }
            )
            $(if !is_semantic && !is_copy =>
                #[inline]
                #[must_use]
                pub fn raw(&self) -> &$base_type {
                    &self.0
                }
            )
        }
    }
}

fn build_semantic_bool_consts(
    name: &str,
    is_bool_type: bool,
    is_nullable: bool,
    resolved: &ResolvedType,
) -> Option<Tokens> {
    if is_nullable {
        return None;
    }
    if is_bool_type && let Some((false_raw, true_raw)) = is_semantic_boolean_attr(name) {
        let semantics = lookup_boolean_semantics(name);
        let const_items: Vec<(String, String)> = semantics
            .iter()
            .map(|(const_name, val)| {
                let variant = if *val {
                    to_upper_camel_case(true_raw)
                } else {
                    to_upper_camel_case(false_raw)
                };
                ((*const_name).to_owned(), variant)
            })
            .collect();
        Some(quote! {

            $(for (const_name, variant) in &const_items join ($['\r']) =>
                /// Semantic boolean constant.
                pub const $(const_name): Self = Self(super::$(name)::$(variant));
            )
        })
    } else if let ResolvedType::Primitive {
        rust_type,
        attr_variant: _attr_variant,
        is_copy: _is_copy,
    } = resolved
        && *rust_type == "bool"
    {
        let semantics = lookup_boolean_semantics(name);
        Some(quote! {

            $(for (const_name, val) in &semantics join ($['\r']) =>
                /// Semantic boolean constant.
                pub const $(*const_name): Self = Self($(val.to_string()));
            )
        })
    } else {
        None
    }
}

fn build_decode_body(
    is_array: bool,
    is_nullable: bool,
    base_type: &str,
    resolved: &ResolvedType,
    min: Option<&str>,
    max: Option<&str>,
    length: Option<u32>,
) -> Tokens {
    if is_array {
        let elem_path = match resolved {
            ResolvedType::Array { element } => sanitize_attr_type_path(element.rust_type_path(true)),
            ResolvedType::Primitive { .. }
            | ResolvedType::IdType { .. }
            | ResolvedType::SemanticType { .. }
            | ResolvedType::Enum { .. }
            | ResolvedType::Bitmap { .. }
            | ResolvedType::Struct { .. } => base_type.to_owned(),
        };
        let elem_ref = elem_path.as_str();
        let elem_is_int = is_primitive_int(elem_ref);
        let len_str = length.map(|len| format_dec_with_underscores(i128::from(len)));
        quote! {
            let AttributeValue::List(items) = val else { return Err(MatterStatus::InvalidDataType); };
            $(if let Some(len) = &len_str =>
                if items.len() > $len { return Err(MatterStatus::ConstraintError); }
            )
            $(if elem_is_int =>
                let list = items
                    .into_iter()
                    .map(|item| crate::traits::decode_scalar::<$elem_ref>(&item))
                    .collect::<Result<Vec<_>, _>>()?;
            )
            $(if !elem_is_int =>
                let list = items.into_iter().map(<$elem_ref>::try_from).collect::<Result<Vec<_>, _>>()?;
            )
            $(if is_nullable =>
                Ok(Self(Nullable::NonNull(list)))
            )
            $(if !is_nullable =>
                Ok(Self(list))
            )
        }
    } else {
        let bound_type = bound_primitive_type(resolved, base_type);
        let parsed_min = min.and_then(|bound| parse_bound_for_type(bound, bound_type));
        let parsed_max = max.and_then(|bound| parse_bound_for_type(bound, bound_type));
        let cmp_target = if matches!(resolved, ResolvedType::SemanticType { .. }) {
            "raw.raw()"
        } else {
            "raw"
        };

        let is_id_type = matches!(resolved, ResolvedType::IdType { .. });
        let min_s = (!is_id_type)
            .then(|| {
                parsed_min
                    .filter(ParsedBound::should_emit_min)
                    .map(|bound| bound.to_string())
            })
            .flatten();
        let max_s = (!is_id_type)
            .then(|| {
                parsed_max
                    .filter(ParsedBound::should_emit_max)
                    .map(|bound| bound.to_string())
            })
            .flatten();
        let is_str_or_bytes = matches!(
            resolved,
            ResolvedType::Primitive {
                rust_type: "String" | "crate::Bytes",
                ..
            }
        );
        let len_str = is_str_or_bytes
            .then(|| length.map(|len| format_dec_with_underscores(i128::from(len))))
            .flatten();

        quote! {
            $(if is_primitive_int(base_type) =>
                let raw = crate::traits::decode_scalar::<$base_type>(&val)?;
            )
            $(if !is_primitive_int(base_type) =>
                let raw = <$base_type>::try_from(val)?;
            )
            $(if let Some(min_val) = &min_s =>
                if $cmp_target < $min_val { return Err(MatterStatus::ConstraintError); }
            )
            $(if let Some(max_val) = &max_s =>
                if $cmp_target > $max_val { return Err(MatterStatus::ConstraintError); }
            )
            $(if let Some(len) = &len_str =>
                if raw.len() > $len { return Err(MatterStatus::ConstraintError); }
            )
            $(if is_nullable =>
                Ok(Self(Nullable::NonNull(raw)))
            )
            $(if !is_nullable =>
                Ok(Self(raw))
            )
        }
    }
}

fn build_extra_from_tokens(attr_name: &str, is_nullable: bool, is_semantic_bool: bool) -> Tokens {
    if !is_semantic_bool {
        return Tokens::new();
    }
    quote! {
        impl From<bool> for $attr_name {
            #[inline]
            fn from(val: bool) -> Self {
                $(if is_nullable =>
                    Self(Nullable::NonNull(super::$attr_name::from(val)))
                )
                $(if !is_nullable =>
                    Self(super::$attr_name::from(val))
                )
            }
        }
    }
}

fn sanitize_attr_type_path(path: String) -> String {
    if path == "VendorId" {
        "model::VendorId".to_owned()
    } else if path == "DataVersion" {
        "model::DataVersion".to_owned()
    } else if let Some(inner) = path.strip_prefix("Vec<").and_then(|p| p.strip_suffix('>')) {
        let sanitized_inner = sanitize_attr_type_path(inner.to_owned());
        format!("Vec<{sanitized_inner}>")
    } else if let Some(inner) = path.strip_prefix("Nullable<").and_then(|p| p.strip_suffix('>')) {
        let sanitized_inner = sanitize_attr_type_path(inner.to_owned());
        format!("Nullable<{sanitized_inner}>")
    } else {
        path
    }
}
