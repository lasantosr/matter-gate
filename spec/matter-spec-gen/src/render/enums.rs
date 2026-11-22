use std::{
    collections::{BTreeMap, BTreeSet},
    result::Result as StdResult,
};

use genco::quote;

use super::context::{PushLine as _, Tokens, cfg_feature_gate, format_hex};
use crate::{
    error::Error,
    model::{ClusterDef, EnumDef, is_semantic_boolean_attr, sanitize_identifier, to_upper_camel_case},
};

type Result<T, E = Error> = StdResult<T, E>;

pub(super) fn emit_enums(
    out: &mut Tokens,
    enums: &[EnumDef],
    feature_gates: Option<&BTreeMap<String, Vec<String>>>,
) -> Result<()> {
    for enm in enums {
        let gate = feature_gates.and_then(|gates| gates.get(&enm.name));
        out.append(enum_tokens(enm, gate)?);
        out.push_blank();
    }
    Ok(())
}

pub(super) fn emit_semantic_booleans(out: &mut Tokens, cluster: &ClusterDef) {
    let mut seen = BTreeSet::new();
    for attr in &cluster.attributes {
        if (attr.type_str.eq_ignore_ascii_case("boolean") || attr.type_str.eq_ignore_ascii_case("bool"))
            && let Some((false_raw, true_raw)) = is_semantic_boolean_attr(&attr.name)
        {
            let enum_name = sanitize_identifier(&attr.name);
            if seen.insert(enum_name.clone()) && !cluster.enums.iter().any(|e| e.name == enum_name) {
                let false_variant = to_upper_camel_case(false_raw);
                let true_variant = to_upper_camel_case(true_raw);
                out.append(semantic_boolean_tokens(&enum_name, &false_variant, &true_variant));
                out.push_blank();
            }
        }
    }
}

fn semantic_boolean_tokens(enum_name: &str, false_variant: &str, true_variant: &str) -> Tokens {
    let doc = format!("/// Specification 2-variant semantic boolean enum for [`{enum_name}`].");
    quote! {
        crate::define_matter_semantic_bool! {
            $doc
            pub enum $enum_name {
                $false_variant = 0,
                $true_variant = 1,
            }
        }
    }
}

fn enum_tokens(enm: &EnumDef, gate: Option<&Vec<String>>) -> Result<Tokens> {
    let EnumDef { name, is_16bit, items } = enm;
    let (repr_type, attr_variant) = if *is_16bit { ("u16", "U16") } else { ("u8", "U8") };
    let doc = format!("/// {name} exhaustive specification enum.");
    let variants: Vec<(&str, String)> = items
        .iter()
        .map(|(item_name, item_val)| (item_name.as_str(), format_hex(u64::from(*item_val), 0)))
        .collect();

    let Some((default_variant, _)) = variants.first() else {
        return Err(Error::Parse(format!("Enum '{name}' has no variants")));
    };

    let cfg_gate = cfg_feature_gate(gate);
    Ok(quote! {
        $cfg_gate
        crate::define_matter_enum! {
            $doc
            pub enum $name {
                $(for (item_name, item_hex) in &variants join ($['\r']) =>
                    $(*item_name) = $item_hex,
                )
            }
            repr = $repr_type;
            attr_variant = $attr_variant;
            default = $(*default_variant);
        }
    })
}
