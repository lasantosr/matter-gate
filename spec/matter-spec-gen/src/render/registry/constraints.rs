//! Static `ValueConstraint`, `FieldDescriptor`, and RPN `ConformanceOp` slice rendering for `SpecRegistry`.

use std::collections::BTreeSet;

use genco::{quote, tokens::quoted};

use super::super::context::{Tokens, TypeContext, format_hex, to_rust_field_name};
use crate::{
    error::Error,
    model::{ClusterDef, ConformanceOp, FieldDef, ParsedBound, ResolvedType, parse_bound_for_type},
};

/// Compiles parsed CSA conformance ops into a `&'static [ConformanceOp]` RPN slice literal.
pub(super) fn compile_conformance_slice(
    ops: &[ConformanceOp],
    is_optional: bool,
    cluster: Option<&ClusterDef>,
    context: &str,
) -> Result<Tokens, Error> {
    if ops.is_empty() {
        return Ok(if is_optional {
            quote!(&[ConformanceOp::Optional])
        } else {
            quote!(&[ConformanceOp::Mandatory])
        });
    }

    let mut emitted: Vec<Tokens> = Vec::new();
    for op in ops {
        match op {
            ConformanceOp::Mandatory => emitted.push(quote!(ConformanceOp::Mandatory)),
            ConformanceOp::Optional => emitted.push(quote!(ConformanceOp::Optional)),
            ConformanceOp::Disallow => emitted.push(quote!(ConformanceOp::Disallow)),
            ConformanceOp::Provisional => emitted.push(quote!(ConformanceOp::Provisional)),
            ConformanceOp::Deprecated => emitted.push(quote!(ConformanceOp::Deprecate)),
            ConformanceOp::Otherwise => emitted.push(quote!(ConformanceOp::Otherwise)),
            ConformanceOp::Not => emitted.push(quote!(ConformanceOp::Not)),
            ConformanceOp::Greater => emitted.push(quote!(ConformanceOp::Greater)),
            ConformanceOp::GreaterOrEqual => emitted.push(quote!(ConformanceOp::GreaterOrEqual)),
            ConformanceOp::And(n) => {
                for _ in 1..*n {
                    emitted.push(quote!(ConformanceOp::And));
                }
            }
            ConformanceOp::Or(n) => {
                for _ in 1..*n {
                    emitted.push(quote!(ConformanceOp::Or));
                }
            }
            ConformanceOp::Feature(code) => {
                if let Some(feature) =
                    cluster.and_then(|c| c.features.iter().find(|f| f.code == *code || f.name == *code))
                {
                    let mask = 1_u32.checked_shl(feature.bit).unwrap_or(0);
                    let mask_hex = format_hex(u64::from(mask), 2);
                    emitted.push(quote!(ConformanceOp::PushFeature($mask_hex)));
                } else if cluster.is_none_or(|c| c.features.is_empty()) {
                    // Nested structs without cluster context or clusters missing `<features>` in upstream XMLs.
                    emitted.push(quote!(ConformanceOp::PushBool(false)));
                } else {
                    return Err(Error::MalformedConformance {
                        context: context.to_owned(),
                        reason: format!("unresolved feature reference '{code}'"),
                    });
                }
            }
            ConformanceOp::Attribute(attr_name) => {
                if attr_name.eq_ignore_ascii_case("ClusterRevision") {
                    emitted.push(quote!(ConformanceOp::PushAttrValue(0xFFFD)));
                } else if let Some(attr) = cluster.and_then(|c| {
                    c.attributes.iter().find(|a| {
                        a.name.eq_ignore_ascii_case(attr_name)
                            || a.define.as_deref().is_some_and(|d| d.eq_ignore_ascii_case(attr_name))
                    })
                }) {
                    let code_hex = format_hex(u64::from(attr.code), 4);
                    emitted.push(quote!(ConformanceOp::PushAttrPresent($code_hex)));
                } else {
                    return Err(Error::MalformedConformance {
                        context: context.to_owned(),
                        reason: format!("unresolved attribute reference '{attr_name}'"),
                    });
                }
            }
            ConformanceOp::Condition(cond) => {
                if cond.eq_ignore_ascii_case("Matter") {
                    emitted.push(quote!(ConformanceOp::PushBool(true)));
                } else if cond.eq_ignore_ascii_case("Zigbee") {
                    emitted.push(quote!(ConformanceOp::PushBool(false)));
                } else {
                    return Err(Error::MalformedConformance {
                        context: context.to_owned(),
                        reason: format!("unsupported condition '{cond}'"),
                    });
                }
            }
            ConformanceOp::Command(_) => {
                // Command presence (`[Cmd]`) evaluates to `false` in attribute-only conformance.
                emitted.push(quote!(ConformanceOp::PushBool(false)));
            }
            ConformanceOp::Literal(lit) => {
                if let Ok(n) = lit.trim().parse::<i64>() {
                    emitted.push(quote!(ConformanceOp::PushConst($n)));
                } else {
                    emitted.push(quote!(ConformanceOp::PushConst(0)));
                }
            }
        }
    }

    Ok(quote!(&[$(for tok in &emitted join (, ) => $tok)]))
}

/// Renders a `ValueConstraint` descriptor literal (handling arrays, bounds, enums, bitmaps, and nested structs).
pub(super) fn value_constraint_tokens(
    type_str: &str,
    entry_type: Option<&str>,
    is_array: bool,
    is_nullable: bool,
    min: Option<&str>,
    max: Option<&str>,
    min_length: Option<u32>,
    length: Option<u32>,
    ctx: &TypeContext<'_>,
    visiting: &mut BTreeSet<String>,
) -> Result<Tokens, Error> {
    let nullable_str = if is_nullable { "true" } else { "false" };

    if is_array || type_str.eq_ignore_ascii_case("array") {
        let item_type = entry_type.unwrap_or(type_str);
        let item_constraint = if item_type.eq_ignore_ascii_case("array") {
            quote!(ValueConstraint {
                value_type: ValueType::Any,
                bounds: ValueBounds::None,
                nullable: false
            })
        } else {
            value_constraint_tokens(item_type, None, false, false, min, max, None, None, ctx, visiting)?
        };
        let min_len = usize::try_from(min_length.unwrap_or(0)).unwrap_or(0);
        let max_len = length.map_or_else(
            || quote!(usize::MAX),
            |l| quote!($(usize::try_from(l).unwrap_or(usize::MAX))),
        );
        return Ok(quote! {
            ValueConstraint {
                value_type: ValueType::List(&ListLength {
                    min: $min_len,
                    max: $max_len,
                    item_constraint: &$item_constraint,
                }),
                bounds: ValueBounds::None,
                nullable: $nullable_str,
            }
        });
    }

    let Ok(resolved) = ctx.resolve_type("", "", type_str, entry_type, false) else {
        return Ok(quote!(ValueConstraint {
            value_type: ValueType::Any,
            bounds: ValueBounds::None,
            nullable: $nullable_str
        }));
    };
    match resolved {
        ResolvedType::Primitive { rust_type, .. } => Ok(primitive_constraint_tokens(
            rust_type,
            min,
            max,
            min_length,
            length,
            nullable_str,
        )),
        ResolvedType::IdType { rust_type, .. } => {
            let vt = match rust_type {
                "FabricIndex" | "FieldId" | "ActionId" => quote!(ValueType::U8),
                "EndpointId" | "GroupId" | "VendorId" => quote!(ValueType::U16),
                "NodeId" | "FabricId" | "EventNumber" => quote!(ValueType::U64),
                _ => quote!(ValueType::U32),
            };
            Ok(quote!(ValueConstraint { value_type: $vt, bounds: ValueBounds::None, nullable: $nullable_str }))
        }
        ResolvedType::SemanticType { rust_type, .. } => {
            let (vt, bounds) = match rust_type {
                "types::HardwareAddress"
                | "types::IpAddress"
                | "types::Ipv6Prefix"
                | "types::Ipv4Address"
                | "types::Ipv6Address" => (quote!(ValueType::OctetString), bytes_bounds(min_length, length)),
                "types::Temperature" => (
                    quote!(ValueType::I16),
                    signed_bounds("i16", min, max, i64::from(i16::MIN), i64::from(i16::MAX)),
                ),
                "types::AmperageMa" | "types::VoltageMv" | "types::PowerMw" | "types::PowerMva"
                | "types::PowerMvar" | "types::EnergyMwh" | "types::EnergyMvah" | "types::EnergyMvarh" => (
                    quote!(ValueType::I64),
                    signed_bounds("i64", min, max, i64::MIN, i64::MAX),
                ),
                "types::Percent" => (
                    quote!(ValueType::U8),
                    unsigned_bounds("u8", min, max, u64::from(u8::MAX)),
                ),
                "types::Percent100ths" => (
                    quote!(ValueType::U16),
                    unsigned_bounds("u16", min, max, u64::from(u16::MAX)),
                ),
                "types::EpochUs" | "types::PosixMs" | "types::SystimeMs" | "types::SystimeUs" => {
                    (quote!(ValueType::U64), unsigned_bounds("u64", min, max, u64::MAX))
                }
                _ => (
                    quote!(ValueType::U32),
                    unsigned_bounds("u32", min, max, u64::from(u32::MAX)),
                ),
            };
            Ok(quote!(ValueConstraint { value_type: $vt, bounds: $bounds, nullable: $nullable_str }))
        }
        ResolvedType::Enum { name, .. } => Ok(enum_constraint_tokens(&name, nullable_str, ctx)),
        ResolvedType::Bitmap { name, .. } => Ok(bitmap_constraint_tokens(&name, nullable_str, ctx)),
        ResolvedType::Struct { name, .. } => {
            if !visiting.insert(name.clone()) {
                return Ok(quote!(ValueConstraint {
                    value_type: ValueType::Any,
                    bounds: ValueBounds::None,
                    nullable: $nullable_str
                }));
            }
            let st_opt = ctx
                .cluster_structs
                .iter()
                .chain(ctx.global_defs.structs.iter())
                .chain(ctx.all_structs.values())
                .find(|s| s.name == name);
            let fields_tok = if let Some(st) = st_opt {
                let mut rendered_fields = Vec::with_capacity(st.fields.len());
                for f in &st.fields {
                    rendered_fields.push(field_descriptor_tokens(f, ctx, None, visiting)?);
                }
                quote! {
                    &[$(for field_tok in &rendered_fields join ($['\r']) =>
                        $field_tok,
                    )]
                }
            } else {
                quote!(&[])
            };
            visiting.remove(&name);
            Ok(quote! {
                ValueConstraint {
                    value_type: ValueType::Struct(&StructDescriptor {
                        name: $(quoted(&name)),
                        fields: $fields_tok,
                    }),
                    bounds: ValueBounds::None,
                    nullable: $nullable_str,
                }
            })
        }
        ResolvedType::Array { .. } => Ok(quote!(ValueConstraint {
            value_type: ValueType::Any,
            bounds: ValueBounds::None,
            nullable: $nullable_str
        })),
    }
}

/// Renders a static `FieldDescriptor` literal for a struct field, command argument, or event field.
pub(super) fn field_descriptor_tokens(
    f: &FieldDef,
    ctx: &TypeContext<'_>,
    cluster: Option<&ClusterDef>,
    visiting: &mut BTreeSet<String>,
) -> Result<Tokens, Error> {
    let fid_hex = format_hex(u64::from(f.field_id), 2);
    let fname = to_rust_field_name(&f.name);
    let constraint = value_constraint_tokens(
        &f.type_str,
        f.entry_type.as_deref(),
        f.is_array,
        f.is_nullable,
        f.min.as_deref(),
        f.max.as_deref(),
        f.min_length,
        f.length,
        ctx,
        visiting,
    )?;
    let context = match cluster {
        Some(c) => format!("{}::{}", c.name, f.name),
        None => f.name.clone(),
    };
    let conf = compile_conformance_slice(&f.conformance, f.is_optional, cluster, &context)?;
    Ok(quote! {
        FieldDescriptor {
            id: FieldId::new($fid_hex),
            name: $(quoted(&fname)),
            constraint: $constraint,
            conformance: $conf,
        }
    })
}

fn primitive_constraint_tokens(
    rust_type: &str,
    min: Option<&str>,
    max: Option<&str>,
    min_length: Option<u32>,
    length: Option<u32>,
    nullable_str: &str,
) -> Tokens {
    let (vt, bounds) = match rust_type {
        "bool" => (quote!(ValueType::Bool), quote!(ValueBounds::None)),
        "u8" => (
            quote!(ValueType::U8),
            unsigned_bounds("u8", min, max, u64::from(u8::MAX)),
        ),
        "u16" => (
            quote!(ValueType::U16),
            unsigned_bounds("u16", min, max, u64::from(u16::MAX)),
        ),
        "u32" => (
            quote!(ValueType::U32),
            unsigned_bounds("u32", min, max, u64::from(u32::MAX)),
        ),
        "u64" => (quote!(ValueType::U64), unsigned_bounds("u64", min, max, u64::MAX)),
        "i8" => (
            quote!(ValueType::I8),
            signed_bounds("i8", min, max, i64::from(i8::MIN), i64::from(i8::MAX)),
        ),
        "i16" => (
            quote!(ValueType::I16),
            signed_bounds("i16", min, max, i64::from(i16::MIN), i64::from(i16::MAX)),
        ),
        "i32" => (
            quote!(ValueType::I32),
            signed_bounds("i32", min, max, i64::from(i32::MIN), i64::from(i32::MAX)),
        ),
        "i64" => (
            quote!(ValueType::I64),
            signed_bounds("i64", min, max, i64::MIN, i64::MAX),
        ),
        "f32" => (quote!(ValueType::F32), quote!(ValueBounds::None)),
        "f64" => (quote!(ValueType::F64), quote!(ValueBounds::None)),
        "String" => {
            let b = match (min_length, length) {
                (None, None) => quote!(ValueBounds::None),
                (lo, hi) => {
                    let lo_u = usize::try_from(lo.unwrap_or(0)).unwrap_or(0);
                    let hi_tok = hi.map_or_else(
                        || quote!(usize::MAX),
                        |v| quote!($(usize::try_from(v).unwrap_or(usize::MAX))),
                    );
                    quote!(ValueBounds::StringLen { min: $lo_u, max: $hi_tok })
                }
            };
            (quote!(ValueType::String), b)
        }
        "crate::Bytes" => (quote!(ValueType::OctetString), bytes_bounds(min_length, length)),
        _ => (quote!(ValueType::Any), quote!(ValueBounds::None)),
    };
    quote!(ValueConstraint { value_type: $vt, bounds: $bounds, nullable: $nullable_str })
}

fn bytes_bounds(min_length: Option<u32>, length: Option<u32>) -> Tokens {
    match (min_length, length) {
        (None, None) => quote!(ValueBounds::None),
        (lo, hi) => {
            let lo_u = usize::try_from(lo.unwrap_or(0)).unwrap_or(0);
            let hi_tok = hi.map_or_else(
                || quote!(usize::MAX),
                |v| quote!($(usize::try_from(v).unwrap_or(usize::MAX))),
            );
            quote!(ValueBounds::BytesLen { min: $lo_u, max: $hi_tok })
        }
    }
}

fn unsigned_bounds(rust_type: &str, min: Option<&str>, max: Option<&str>, type_max: u64) -> Tokens {
    let lo = min.and_then(|s| parse_u64_bound(s, rust_type)).unwrap_or(0);
    let hi = max.and_then(|s| parse_u64_bound(s, rust_type)).unwrap_or(type_max);
    if lo == 0 && hi == type_max {
        quote!(ValueBounds::None)
    } else {
        quote!(ValueBounds::Unsigned { min: $lo, max: $hi })
    }
}

fn signed_bounds(rust_type: &str, min: Option<&str>, max: Option<&str>, type_min: i64, type_max: i64) -> Tokens {
    let lo = min.and_then(|s| parse_i64_bound(s, rust_type)).unwrap_or(type_min);
    let hi = max.and_then(|s| parse_i64_bound(s, rust_type)).unwrap_or(type_max);
    if lo == type_min && hi == type_max {
        quote!(ValueBounds::None)
    } else {
        quote!(ValueBounds::Signed { min: $lo, max: $hi })
    }
}

fn parse_u64_bound(s: &str, rust_type: &str) -> Option<u64> {
    match parse_bound_for_type(s, rust_type)? {
        ParsedBound::U8(v) => Some(u64::from(v)),
        ParsedBound::U16(v) => Some(u64::from(v)),
        ParsedBound::U32(v) => Some(u64::from(v)),
        ParsedBound::U64(v) => Some(v),
        _ => None,
    }
}

fn parse_i64_bound(s: &str, rust_type: &str) -> Option<i64> {
    match parse_bound_for_type(s, rust_type)? {
        ParsedBound::I8(v) => Some(i64::from(v)),
        ParsedBound::I16(v) => Some(i64::from(v)),
        ParsedBound::I32(v) => Some(i64::from(v)),
        ParsedBound::I64(v) => Some(v),
        _ => None,
    }
}

fn enum_constraint_tokens(name: &str, nullable_str: &str, ctx: &TypeContext<'_>) -> Tokens {
    let enum_def = ctx
        .cluster_enums
        .iter()
        .chain(ctx.global_defs.enums.iter())
        .chain(ctx.all_enums.values())
        .find(|e| e.name == name);
    let bits: u8 = if enum_def.is_some_and(|e| e.is_16bit) { 16 } else { 8 };
    let vals: Vec<u64> = enum_def
        .map(|e| e.items.iter().map(|(_, v)| u64::from(*v)).collect())
        .unwrap_or_default();
    quote! {
        ValueConstraint {
            value_type: ValueType::Enum { bits: $bits, valid_values: &[$(for &v in &vals join (, ) => $v)] },
            bounds: ValueBounds::None,
            nullable: $nullable_str,
        }
    }
}

fn bitmap_constraint_tokens(name: &str, nullable_str: &str, ctx: &TypeContext<'_>) -> Tokens {
    let bitmap_def = ctx
        .cluster_bitmaps
        .iter()
        .chain(ctx.global_defs.bitmaps.iter())
        .chain(ctx.all_bitmaps.values())
        .find(|b| b.name == name);
    let bits: u8 = bitmap_def
        .map(|b| match b.type_str.to_ascii_lowercase().as_str() {
            "bitmap8" => 8,
            "bitmap16" => 16,
            "bitmap64" => 64,
            _ => 32,
        })
        .unwrap_or(32);
    let mask: u64 = bitmap_def
        .map(|b| b.fields.iter().fold(0_u64, |acc, (_, m)| acc | *m))
        .unwrap_or(u64::MAX);
    let mask_hex = format_hex(mask, 2);
    quote! {
        ValueConstraint {
            value_type: ValueType::Bitmap { bits: $bits, valid_mask: $mask_hex },
            bounds: ValueBounds::None,
            nullable: $nullable_str,
        }
    }
}
