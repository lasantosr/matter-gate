use std::{
    collections::{BTreeMap, BTreeSet},
    result::Result as StdResult,
};

use genco::{quote, tokens::quoted};

use super::context::{PushLine as _, Tokens, cfg_feature_gate, doc_comment, format_hex};
use crate::{
    error::Error,
    model::{BitmapDef, FeatureDef, to_shouty_snake_case, to_snake_case},
};

type Result<T, E = Error> = StdResult<T, E>;

pub(super) fn emit_bitmap_struct(
    out: &mut Tokens,
    name: &str,
    width: &str,
    doc: &str,
    fields: &[(&str, u64, &str)],
    gate: Option<&Vec<String>>,
) -> Result<()> {
    let (attr_variant, width_digits) = resolve_bitmap_width_info(name, width)?;
    let display_fmt = format!("{{:#0{width_digits}x}}");
    let flags = emit_bitmap_flags(fields);
    let cfg_gate = cfg_feature_gate(gate);

    let tokens: Tokens = quote! {
        $cfg_gate
        crate::define_matter_bitmap! {
            $(doc_comment(doc))
            pub struct $name($width);
            attr_variant = $attr_variant;
            display_fmt = $(quoted(display_fmt));
            flags {
                $flags
            }
        }
    };
    out.append(tokens);
    out.push_blank();
    Ok(())
}

pub(super) fn emit_feature_map(out: &mut Tokens, features: &[FeatureDef]) -> Result<()> {
    let mut fields: Vec<(&str, u64, &str)> = Vec::new();
    let mut seen_const_names = BTreeSet::new();
    for feature in features {
        let FeatureDef {
            bit,
            code,
            name,
            summary,
        } = feature;
        if *bit >= 32 {
            return Err(Error::Parse(format!(
                "Feature '{name}' bit {bit} exceeds 32-bit FeatureMap"
            )));
        }
        let mask = 1_u64 << *bit;
        let name_shouty = to_shouty_snake_case(name);
        if !name_shouty.is_empty() && seen_const_names.insert(name_shouty) {
            fields.push((name.as_str(), mask, summary.as_str()));
        }
        if !code.is_empty() {
            let code_shouty = to_shouty_snake_case(code);
            if !code_shouty.is_empty() && seen_const_names.insert(code_shouty) {
                fields.push((code.as_str(), mask, summary.as_str()));
            }
        }
    }
    emit_bitmap_struct(
        out,
        "FeatureMap",
        "u32",
        "Strongly-typed 32-bit FeatureMap bitmap for this cluster.",
        &fields,
        None,
    )
}

pub(super) fn emit_bitmaps(
    out: &mut Tokens,
    bitmaps: &[BitmapDef],
    feature_gates: Option<&BTreeMap<String, Vec<String>>>,
) -> Result<()> {
    for bitmap in bitmaps {
        let BitmapDef { name, type_str, fields } = bitmap;
        let width = match type_str.to_lowercase().as_str() {
            "bitmap8" => "u8",
            "bitmap16" => "u16",
            "bitmap24" | "bitmap32" => "u32",
            "bitmap64" => "u64",
            other => {
                return Err(Error::UnsupportedBitmapType {
                    name: name.to_owned(),
                    type_str: other.to_owned(),
                });
            }
        };
        let doc = format!("{name} bitmap.");
        let fields: Vec<(&str, u64, &str)> = fields
            .iter()
            .map(|(f_name, mask)| (f_name.as_str(), *mask, ""))
            .collect();
        let gate = feature_gates.and_then(|gates| gates.get(name));
        emit_bitmap_struct(out, name, width, &doc, &fields, gate)?;
    }
    Ok(())
}

fn resolve_bitmap_width_info(name: &str, width: &str) -> Result<(&'static str, usize)> {
    let (variant, digits) = match width {
        "u8" => ("U8", 4),
        "u16" => ("U16", 6),
        "u32" => ("U32", 10),
        "u64" => ("U64", 18),
        other => {
            return Err(Error::UnsupportedBitmapWidth {
                name: name.to_owned(),
                width: other.to_owned(),
            });
        }
    };
    Ok((variant, digits))
}

fn emit_bitmap_flags(fields: &[(&str, u64, &str)]) -> Tokens {
    let mut sorted_fields: Vec<(&str, u64, &str, String)> = fields
        .iter()
        .map(|(f_name, mask, comment)| (*f_name, *mask, *comment, to_shouty_snake_case(f_name)))
        .collect();
    sorted_fields.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.3.cmp(&b.3)));

    quote! {
        $(for (f_name, mask, comment, const_name) in &sorted_fields join ($['\r']) =>
            $(if comment.trim().is_empty() {
                $(doc_comment(&format!("Bitmask flag for `{f_name}` (`{}`).", format_hex(*mask, 0))))
            } else {
                $(doc_comment(comment))
            })
            $(format!("{const_name} : is_{} = {};", to_snake_case(f_name), format_hex(*mask, 0)))
        )
    }
}
