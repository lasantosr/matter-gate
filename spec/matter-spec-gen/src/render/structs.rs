use std::{collections::BTreeMap, result::Result as StdResult};

use genco::{quote, tokens::quoted};

use super::{
    attributes::bound_primitive_type,
    context::{PushLine as _, Tokens, TypeContext, cluster_cfg_gate, doc_comment, to_rust_field_name},
    record::{
        FieldBounds, IdPrefix, MemberFlags, MemberPartition, RecordConstructors, RecordStructDef, ResolvedMember,
        SingleMandatoryFrom, StructAttrs,
    },
};
use crate::{
    error::Error,
    model::{FieldDef, ParsedBound, ResolvedType, StructDef, format_dec_with_underscores, parse_bound_for_type},
};

type Result<T, E = Error> = StdResult<T, E>;

pub(super) fn resolve_fields(
    parent_name: &str,
    fields: &[FieldDef],
    in_submodule: bool,
    ctx: &TypeContext<'_>,
) -> Result<Vec<ResolvedMember>> {
    fields
        .iter()
        .map(|field| {
            let FieldDef {
                field_id,
                name,
                type_str,
                entry_type,
                is_array,
                is_nullable,
                is_optional,
                min,
                max,
                length,
                ..
            } = field;
            let resolved = ctx.resolve_type(parent_name, name, type_str, entry_type.as_deref(), *is_array)?;
            let flags = MemberFlags {
                is_optional: *is_optional,
                is_nullable: *is_nullable,
                is_array: *is_array,
                is_semantic_bool: false,
            };
            let bounds = FieldBounds {
                min: min.clone(),
                max: max.clone(),
                length: *length,
            };
            Ok(ResolvedMember::for_field(
                name.clone(),
                *field_id,
                flags,
                bounds,
                resolved,
                in_submodule,
            ))
        })
        .collect()
}

pub(super) fn emit_fields_into_vec(
    fields: &[ResolvedMember],
    receiver: &str,
    const_prefix: &str,
    has_both: bool,
) -> Tokens {
    let all_mandatory = fields.iter().all(|field| !field.is_optional);
    if all_mandatory {
        if fields.is_empty() {
            return quote!(Vec::new());
        }
        quote! {
            alloc::vec![
                $(for field in fields join ($['\r']) =>
                    $(convert_field(field, receiver, const_prefix)),
                )
            ]
        }
    } else {
        let capacity = fields.len();
        quote! {
            let mut fields = Vec::with_capacity($capacity);
            $(for field in fields join ($['\r']) =>
                $(push_field(field, receiver, const_prefix, has_both))
            )
            fields
        }
    }
}

pub(super) fn emit_fields_decode_and_construct(
    fields: &[ResolvedMember],
    in_submodule: bool,
    missing_status: &str,
) -> (Tokens, Tokens) {
    let decode_loop = if fields.is_empty() {
        quote!(let _ = fields;)
    } else if let [field] = fields {
        let decoding = try_from_field_decoding_tokens(
            &field.fname,
            &field.resolved,
            &field.type_path,
            in_submodule,
            field.is_nullable,
            field.is_array,
            field.min.as_deref(),
            field.max.as_deref(),
            field.length,
        );
        let field_id = field.id;
        quote! {
            for (fid, fval) in fields {
                if fid.raw() == $field_id {
                    if $(&field.fname).is_some() {
                        return Err((Some(fid), MatterStatus::ConstraintError));
                    }
                    (|| -> Result<(), MatterStatus> {
                        $decoding
                        Ok(())
                    })().map_err(|status| (Some(fid), status))?;
                } else {
                    return Err((Some(fid), MatterStatus::ConstraintError));
                }
            }
        }
    } else {
        quote! {
            for (fid, fval) in fields {
                match fid.raw() {
                    $(for field in fields join ($['\r']) =>
                        $(field.id) => {
                            if $(&field.fname).is_some() {
                                return Err((Some(fid), MatterStatus::ConstraintError));
                            }
                            let decode_one = || -> Result<(), MatterStatus> {
                                $(try_from_field_decoding_tokens(
                                    &field.fname,
                                    &field.resolved,
                                    &field.type_path,
                                    in_submodule,
                                    field.is_nullable,
                                    field.is_array,
                                    field.min.as_deref(),
                                    field.max.as_deref(),
                                    field.length,
                                ))
                                Ok(())
                            };
                            decode_one().map_err(|st| (Some(fid), st))?;
                        }
                    )
                    _ => return Err((Some(fid), MatterStatus::ConstraintError)),
                }
            }
        }
    };

    let construct_fields = quote! {
        $(for f in fields join ($['\r']) =>
            $(if f.is_optional =>
                $(&f.fname): Optional::from($(&f.fname)),
            )
            $(if !f.is_optional =>
                $(&f.fname): $(&f.fname).ok_or((Some(FieldId::new($(f.id))), $missing_status))?,
            )
        )
    };

    (decode_loop, construct_fields)
}

pub(super) fn field_validation_tokens(
    val_expr: &str,
    resolved: &ResolvedType,
    rust_type: &str,
    min: Option<&str>,
    max: Option<&str>,
    length: Option<u32>,
) -> Tokens {
    let bound_type = bound_primitive_type(resolved, rust_type);
    let parsed_min = min.and_then(|bound| parse_bound_for_type(bound, bound_type));
    let parsed_max = max.and_then(|bound| parse_bound_for_type(bound, bound_type));
    let is_semantic = matches!(resolved, ResolvedType::SemanticType { .. });
    let val_expr = if is_semantic {
        format!("{val_expr}.raw()")
    } else {
        val_expr.to_owned()
    };
    let val_expr = val_expr.as_str();

    let is_id_type = matches!(
        resolved,
        ResolvedType::IdType {
            rust_type: _rust_type,
            attr_variant: _attr_variant,
        }
    );

    let emit_range = (!is_id_type)
        .then(|| {
            let min_v = parsed_min.filter(ParsedBound::should_emit_min);
            let max_v = parsed_max.filter(ParsedBound::should_emit_max);
            min_v
                .zip(max_v)
                .map(|(m_val, x_val)| (m_val.to_string(), x_val.to_string()))
        })
        .flatten();

    let emit_min = (!is_id_type && emit_range.is_none())
        .then(|| {
            parsed_min
                .filter(ParsedBound::should_emit_min)
                .map(|bound| bound.to_string())
        })
        .flatten();

    let emit_max = (!is_id_type && emit_range.is_none())
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
            attr_variant: _attr_variant,
            is_copy: _is_copy,
        }
    );
    let len_str = is_str_or_bytes
        .then(|| length.map(|len| format_dec_with_underscores(i128::from(len))))
        .flatten();

    quote! {
        $(if let Some((min_s, max_s)) = &emit_range =>
            ($min_s..=$max_s).contains(&$val_expr).then_some(()).ok_or(MatterStatus::ConstraintError)?;
        )
        $(if let Some(min_s) = &emit_min =>
            ($val_expr >= $min_s).then_some(()).ok_or(MatterStatus::ConstraintError)?;
        )
        $(if let Some(max_s) = &emit_max =>
            ($val_expr <= $max_s).then_some(()).ok_or(MatterStatus::ConstraintError)?;
        )
        $(if let Some(len) = &len_str =>
            ($val_expr.len() <= $len).then_some(()).ok_or(MatterStatus::ConstraintError)?;
        )
    }
}

pub(super) fn try_from_field_decoding_tokens(
    fname: &str,
    resolved: &ResolvedType,
    type_path: &str,
    in_submodule: bool,
    is_nullable: bool,
    is_array: bool,
    min: Option<&str>,
    max: Option<&str>,
    length: Option<u32>,
) -> Tokens {
    if is_nullable {
        if is_array {
            let elem_path = array_element_path(resolved, type_path, in_submodule);
            let len_str = length.map(|len| format_dec_with_underscores(i128::from(len)));
            quote! {
                let decoded = crate::traits::decode_nullable_list(fval, <$elem_path>::try_from)?;
                $(if let Some(len) = &len_str =>
                    (!matches!(&decoded, Nullable::NonNull(v) if v.len() > $len))
                        .then_some(())
                        .ok_or(MatterStatus::ConstraintError)?;
                )
                $fname = Some(decoded);
            }
        } else {
            let bound_type = bound_primitive_type(resolved, type_path);
            let parsed_min = min.and_then(|bound| parse_bound_for_type(bound, bound_type));
            let parsed_max = max.and_then(|bound| parse_bound_for_type(bound, bound_type));
            let cmp_lhs = if matches!(resolved, ResolvedType::SemanticType { .. }) {
                "v.raw()"
            } else {
                "*v"
            };
            let is_id_type = matches!(
                resolved,
                ResolvedType::IdType {
                    rust_type: _rust_type,
                    attr_variant: _attr_variant,
                }
            );
            let is_str_or_bytes = matches!(
                resolved,
                ResolvedType::Primitive {
                    rust_type: "String" | "crate::Bytes",
                    attr_variant: _attr_variant,
                    is_copy: _is_copy,
                }
            );
            let mut conditions: Vec<Tokens> = Vec::new();
            if !is_id_type {
                if let Some(min_val) = parsed_min
                    && min_val.should_emit_min()
                {
                    let min_s = min_val.to_string();
                    conditions.push(quote!($cmp_lhs < $min_s));
                }
                if let Some(max_val) = parsed_max
                    && max_val.should_emit_max()
                {
                    let max_s = max_val.to_string();
                    conditions.push(quote!($cmp_lhs > $max_s));
                }
            }
            if is_str_or_bytes && let Some(len) = length {
                let len_str = format_dec_with_underscores(i128::from(len));
                conditions.push(quote!(v.len() > $len_str));
            }

            let cond_checks: Tokens = quote! {
                $(for cond in conditions join ($['\r']) =>
                    (!matches!(&decoded, Nullable::NonNull(v) if $cond))
                        .then_some(())
                        .ok_or(MatterStatus::ConstraintError)?;
                )
            };

            quote! {
                let decoded = crate::traits::decode_nullable(fval, <$type_path>::try_from)?;
                $cond_checks
                $fname = Some(decoded);
            }
        }
    } else if is_array {
        let elem_path = array_element_path(resolved, type_path, in_submodule);
        let len_str = length.map(|len| format_dec_with_underscores(i128::from(len)));
        quote! {
            let decoded = crate::traits::decode_list(fval, <$elem_path>::try_from)?;
            $(if let Some(len) = &len_str =>
                (decoded.len() <= $len).then_some(()).ok_or(MatterStatus::ConstraintError)?;
            )
            $fname = Some(decoded);
        }
    } else {
        let validation = field_validation_tokens("decoded", resolved, type_path, min, max, length);
        quote! {
            let decoded = <$type_path>::try_from(fval)?;
            $validation
            $fname = Some(decoded);
        }
    }
}

pub(super) fn emit_structs(
    out: &mut Tokens,
    structs: &[StructDef],
    in_submodule: bool,
    ctx: &TypeContext<'_>,
    cfg_gates: Option<&BTreeMap<String, Vec<String>>>,
) -> Result<()> {
    for st in structs {
        let gate = cfg_gates.and_then(|gates| gates.get(&st.name));
        let tokens = single_struct_tokens(st, in_submodule, ctx, gate)?;
        out.append(tokens);
        out.push_blank();
    }
    Ok(())
}

fn field_expr(receiver: &str, field: &ResolvedMember, _has_both: bool) -> String {
    format!("{receiver}.{}", field.fname)
}

fn convert_field(field: &ResolvedMember, receiver: &str, const_prefix: &str) -> Tokens {
    let fid_const = format!("{const_prefix}::{}", field.field_const_ident());
    let access = field_expr(receiver, field, false);
    let is_array = matches!(&field.resolved, ResolvedType::Array { element: _ });
    if field.is_nullable {
        if is_array {
            quote! {
                ($(&fid_const), match $(&access) {
                    Nullable::NonNull(v) => AttributeValue::List(v.into_iter()
                        .map(AttributeValue::from)
                        .collect()),
                    Nullable::Null => AttributeValue::Null,
                })
            }
        } else {
            quote! {
                ($(&fid_const), match $(&access) {
                    Nullable::NonNull(v) => AttributeValue::from(v),
                    Nullable::Null => AttributeValue::Null,
                })
            }
        }
    } else if is_array {
        quote! {
            ($(&fid_const), AttributeValue::List($(&access).into_iter()
                .map(AttributeValue::from)
                .collect()))
        }
    } else {
        quote! {
            ($(&fid_const), AttributeValue::from($(&access)))
        }
    }
}

fn push_field(field: &ResolvedMember, receiver: &str, const_prefix: &str, has_both: bool) -> Tokens {
    let fid_const = format!("{const_prefix}::{}", field.field_const_ident());
    let access = field_expr(receiver, field, has_both);
    let is_array = matches!(&field.resolved, ResolvedType::Array { element: _ });
    if field.is_optional && field.is_nullable {
        if is_array {
            quote! {
                if let Optional::Present(inner) = $(&access) {
                    match inner {
                        Nullable::NonNull(v) => {
                            fields.push(($(&fid_const), AttributeValue::List(v.into_iter()
                                .map(AttributeValue::from)
                                .collect())));
                        }
                        Nullable::Null => {
                            fields.push(($(&fid_const), AttributeValue::Null));
                        }
                    }
                }
            }
        } else {
            quote! {
                if let Optional::Present(inner) = $(&access) {
                    match inner {
                        Nullable::NonNull(v) => {
                            fields.push(($(&fid_const), AttributeValue::from(v)));
                        }
                        Nullable::Null => {
                            fields.push(($(&fid_const), AttributeValue::Null));
                        }
                    }
                }
            }
        }
    } else if field.is_optional {
        if is_array {
            quote! {
                if let Optional::Present(v) = $(&access) {
                    fields.push(($(&fid_const), AttributeValue::List(v.into_iter()
                        .map(AttributeValue::from)
                        .collect())));
                }
            }
        } else {
            quote! {
                if let Optional::Present(v) = $(&access) {
                    fields.push(($(&fid_const), AttributeValue::from(v)));
                }
            }
        }
    } else if field.is_nullable {
        if is_array {
            quote! {
                match $(&access) {
                    Nullable::NonNull(v) => {
                        fields.push(($(&fid_const), AttributeValue::List(v.into_iter()
                            .map(AttributeValue::from)
                            .collect())));
                    }
                    Nullable::Null => {
                        fields.push(($(&fid_const), AttributeValue::Null));
                    }
                }
            }
        } else {
            quote! {
                match $(&access) {
                    Nullable::NonNull(v) => {
                        fields.push(($(&fid_const), AttributeValue::from(v)));
                    }
                    Nullable::Null => {
                        fields.push(($(&fid_const), AttributeValue::Null));
                    }
                }
            }
        }
    } else if is_array {
        quote! {
            fields.push(($(&fid_const), AttributeValue::List($(&access).into_iter()
                .map(AttributeValue::from)
                .collect())));
        }
    } else {
        quote! {
            fields.push(($(&fid_const), AttributeValue::from($(&access))));
        }
    }
}

fn array_element_path(resolved: &ResolvedType, type_path: &str, in_submodule: bool) -> String {
    match resolved {
        ResolvedType::Array { element } => element.rust_type_path(in_submodule),
        ResolvedType::Primitive {
            rust_type: _,
            attr_variant: _,
            is_copy: _,
        }
        | ResolvedType::IdType {
            rust_type: _,
            attr_variant: _,
        }
        | ResolvedType::SemanticType {
            rust_type: _,
            attr_variant: _,
            is_copy: _,
        }
        | ResolvedType::Enum {
            name: _,
            is_16bit: _,
            is_global: _,
        }
        | ResolvedType::Bitmap {
            name: _,
            type_str: _,
            is_global: _,
        }
        | ResolvedType::Struct { name: _, is_global: _ } => type_path.to_owned(),
    }
}

fn single_struct_tokens(
    st: &StructDef,
    in_submodule: bool,
    ctx: &TypeContext<'_>,
    gate: Option<&Vec<String>>,
) -> Result<Tokens> {
    let StructDef {
        name,
        description,
        is_fabric_scoped,
        fields,
    } = st;

    let resolved_fields = resolve_fields(name, fields, in_submodule, ctx)?;
    let partition = MemberPartition::new(&resolved_fields);
    let can_derive_eq_hash = ctx.eq_structs.contains(name);

    let (decode_loop, construct_fields) =
        emit_fields_decode_and_construct(&resolved_fields, in_submodule, "MatterStatus::ConstraintError");
    let encode_vec_body = emit_fields_into_vec(&resolved_fields, "self", "Self", false);

    let attrs = if partition.is_empty() {
        StructAttrs::unit(true)
    } else {
        StructAttrs::model(can_derive_eq_hash, true)
    };
    let struct_doc = quote! {
        $(if *is_fabric_scoped =>
            $("/// Fabric-scoped data structure.")
        )
        $(doc_comment(description))
    };
    let struct_def = quote! {
        $(RecordStructDef {
            doc: struct_doc,
            name,
            partition: &partition,
            attrs,
            cfg_gate: gate,
        })
    };
    let methods_tokens = quote! {
        $(RecordConstructors {
            partition: &partition,
            prefix: IdPrefix::Field,
            too_many_args_reason: "Struct has large mandatory argument set matching CSA specification",
            single_char_reason: Some("Struct field names match CSA specification"),
        })
    };
    let single_mand_from = quote!($(SingleMandatoryFrom::maybe_gated(name, &partition.mandatory, gate)));

    let short_field_count = partition.short_field_count();
    let many_single_char_reason = "Specification struct contains multiple CSA single-character field names";
    let cfg_gate = cluster_cfg_gate(gate);

    let tokens: Tokens = quote! {
        $struct_def

        $(&cfg_gate)
        impl $name {
            $methods_tokens

            $(if short_field_count >= 5 =>
                #[expect(clippy::many_single_char_names, reason = $(quoted(many_single_char_reason)))]
            )
            pub(crate) fn decode_fields_inner(
                fields: Vec<(FieldId, AttributeValue)>,
            ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
                $(for f in fields join ($['\r']) =>
                    let mut $(to_rust_field_name(&f.name)) = None;
                )
                $decode_loop
                Ok(Self {
                    $construct_fields
                })
            }

            pub(crate) fn encode_fields_inner(self) -> Vec<(FieldId, AttributeValue)> {
                $encode_vec_body
            }
        }

        $single_mand_from

        $(&cfg_gate)
        impl From<$name> for AttributeValue {
            #[inline]
            fn from(val: $name) -> Self {
                AttributeValue::Struct(val.encode_fields_inner())
            }
        }

        $(&cfg_gate)
        impl TryFrom<AttributeValue> for $name {
            type Error = MatterStatus;

            #[inline]
            fn try_from(val: AttributeValue) -> Result<Self, MatterStatus> {
                let AttributeValue::Struct(fields) = val else { return Err(MatterStatus::InvalidDataType); };
                Self::decode_fields_inner(fields).map_err(|(_, status)| status)
            }
        }
    };

    let is_mode_option = name == "ModeOptionStruct" && ctx.cluster_enums.iter().any(|e| e.name == "ModeTag");

    Ok(quote! {
        $tokens
        $(if is_mode_option =>
            impl crate::traits::ModeOptionBuilder<ModeTagStruct> for ModeOptionStruct {
                #[inline]
                fn build(
                    label: String,
                    mode: u8,
                    mode_tags: Vec<ModeTagStruct>,
                ) -> Self {
                    Self::new(label, mode, mode_tags)
                }
            }
        )
    })
}
