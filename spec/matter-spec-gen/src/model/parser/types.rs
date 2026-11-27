use std::collections::BTreeSet;

use super::{
    AttributeDef, BitmapDef, CommandArgDef, CommandDef, EnumDef, EventDef, EventFieldDef, StructDef, StructFieldDef,
    identifiers::parse_hex_or_dec_u64, parse_hex_or_dec, sanitize_identifier,
};
use crate::{error::Error, model::Result, xml::*};

pub(crate) fn parse_attribute_xml(attr_xml: AttributeXml) -> Option<AttributeDef> {
    let AttributeXml {
        code,
        name_attr,
        define,
        type_attr,
        // Ignored: `matter-spec` models server-side clusters; `side="client"` only appears on a few legacy ZCL
        // attributes.
        side: _side,
        optional,
        writable,
        readable,
        reportable,
        default,
        min,
        max,
        min_length,
        length,
        entry_type,
        is_nullable,
        is_fabric_sensitive,
        must_use_timed_write,
        must_use_atomic_write,
        // Ignored: `matter-spec` organizes schemas into per-release modules (`v1_0`..`v1_6`) with exact
        // `ClusterRevision`.
        introduced_in: _introduced_in,
        // Ignored: provisionality is tracked via `ConformanceOp::Provisional` rather than XML `apiMaturity`.
        api_maturity: _api_maturity,
        items,
    } = attr_xml;

    let mut text_name = None;
    let mut description = String::new();
    let mut access_items = Vec::new();
    let mut quality = super::QualityFlags {
        atomic_write: must_use_atomic_write.as_deref() == Some("true"),
        reportable: reportable.as_deref() == Some("true"),
        ..Default::default()
    };
    let mut conformance = Vec::new();
    let mut choice = None;
    for item in items {
        match item {
            AttributeItem::Description(SimpleTextXml { text }) => {
                description = text;
            }
            AttributeItem::Access(ax) => {
                access_items.push(ax);
            }
            AttributeItem::Quality(QualityXml {
                change_omitted,
                large_message,
            }) => {
                if change_omitted.as_deref() == Some("true") {
                    quality.change_omitted = true;
                }
                if large_message.as_deref() == Some("true") {
                    quality.large_message = true;
                }
            }
            AttributeItem::MandatoryConform(c) => {
                if choice.is_none() {
                    choice = c.choice.clone();
                }
                super::conformance::compile_conform_xml(&c, &mut conformance);
                conformance.push(super::ConformanceOp::Mandatory);
            }
            AttributeItem::OptionalConform(c) => {
                if choice.is_none() {
                    choice = c.choice.clone();
                }
                super::conformance::compile_conform_xml(&c, &mut conformance);
                conformance.push(super::ConformanceOp::Optional);
            }
            AttributeItem::OtherwiseConform(o) => {
                super::conformance::compile_otherwise_conform(&o, &mut conformance);
            }
            AttributeItem::DeprecateConform(EmptyConformXml) => {
                conformance.push(super::ConformanceOp::Deprecated);
            }
            AttributeItem::DisallowConform(EmptyConformXml) => {
                conformance.push(super::ConformanceOp::Disallow);
            }
            AttributeItem::DescribedConform(EmptyConformXml) => {}
            AttributeItem::GreaterOrEqualTerm(GreaterOrEqualTermXml { items: term_items }) => {
                super::conformance::compile_greater_or_equal_term(&term_items, &mut conformance);
            }
            AttributeItem::Text(raw_text) => {
                let trimmed = raw_text.trim();
                if !trimmed.is_empty() {
                    text_name = Some(trimmed.to_owned());
                }
            }
        }
    }

    let attr_code = parse_hex_or_dec(&code)?;
    let raw_name = name_attr.or(text_name).or(define.clone()).unwrap_or_default();
    let name = sanitize_identifier(&raw_name);
    if name.is_empty() {
        return None;
    }

    let type_str = type_attr.unwrap_or_default();
    let is_array = type_str == "array";
    let is_nullable = is_nullable.as_deref() == Some("true");
    let is_writable = writable.as_deref() == Some("true");
    let is_optional = optional.as_deref() == Some("true");
    let length_val = length.as_deref().and_then(parse_hex_or_dec);
    let min_length_val = min_length.as_deref().and_then(parse_hex_or_dec);
    let list_length = (is_array || min_length_val.is_some() || length_val.is_some()).then(|| super::ListLength {
        min: min_length_val,
        max: length_val,
        item_constraint: entry_type.clone(),
    });
    let default_read = readable.as_deref() != Some("false");
    let mut access = super::conformance::parse_access_flags(&access_items, default_read, is_writable);
    if is_fabric_sensitive.as_deref() == Some("true") {
        access.fabric_sensitive = true;
    }
    if must_use_timed_write.as_deref() == Some("true") {
        access.timed = true;
    }

    Some(AttributeDef {
        name,
        code: attr_code,
        type_str,
        entry_type,
        is_array,
        is_nullable,
        is_writable,
        is_optional,
        min,
        max,
        length: length_val,
        min_length: min_length_val,
        list_length,
        default,
        define,
        access,
        quality,
        conformance,
        choice,
        description,
    })
}

pub(crate) fn parse_command_xml(cmd_xml: CommandXml) -> Result<Option<CommandDef>> {
    let CommandXml {
        code,
        name,
        source,
        optional: _optional,
        response,
        must_use_timed_invoke,
        is_fabric_scoped,
        disable_default_response: _disable_default_response,
        cli: _cli,
        cli_function_name: _cli_function_name,
        no_default_implementation: _no_default_implementation,
        manufacturer_code: _manufacturer_code,
        api_maturity: _api_maturity,
        items,
    } = cmd_xml;

    let cmd_name = sanitize_identifier(&name);
    let cmd_code = parse_hex_or_dec(&code)
        .ok_or_else(|| Error::Parse(format!("Invalid command code '{code}' on command '{cmd_name}'")))?;
    let source_str = source.unwrap_or_else(|| "client".to_owned());
    let response_opt = response.as_deref().map(sanitize_identifier);

    let mut description = String::new();
    let mut args = Vec::new();
    let mut seen_arg_ids = BTreeSet::new();
    let mut next_arg_id = 0;
    let mut access_items = Vec::new();
    let mut quality = super::QualityFlags::default();
    let mut conformance = Vec::new();
    let mut choice = None;

    for item in items {
        match item {
            CommandItem::Description(SimpleTextXml { text }) => {
                description = text;
            }
            CommandItem::Access(ax) => {
                access_items.push(ax);
            }
            CommandItem::Quality(QualityXml {
                change_omitted,
                large_message,
            }) => {
                if change_omitted.as_deref() == Some("true") {
                    quality.change_omitted = true;
                }
                if large_message.as_deref() == Some("true") {
                    quality.large_message = true;
                }
            }
            CommandItem::DescribedConform(EmptyConformXml) | CommandItem::Text(_) => {}
            CommandItem::Arg(arg) => {
                if let Some(parsed_arg) = parse_command_arg_xml(*arg, &mut seen_arg_ids, &mut next_arg_id)? {
                    args.push(parsed_arg);
                }
            }
            CommandItem::MandatoryConform(c) => {
                if choice.is_none() {
                    choice = c.choice.clone();
                }
                super::conformance::compile_conform_xml(&c, &mut conformance);
                conformance.push(super::ConformanceOp::Mandatory);
            }
            CommandItem::OptionalConform(c) => {
                if choice.is_none() {
                    choice = c.choice.clone();
                }
                super::conformance::compile_conform_xml(&c, &mut conformance);
                conformance.push(super::ConformanceOp::Optional);
            }
            CommandItem::OtherwiseConform(o) => {
                super::conformance::compile_otherwise_conform(&o, &mut conformance);
            }
            CommandItem::DisallowConform(EmptyConformXml) => {
                conformance.push(super::ConformanceOp::Disallow);
            }
        }
    }

    let direction = super::conformance::parse_command_direction(&cmd_name, &source_str)?;
    let mut access = super::conformance::parse_access_flags(&access_items, false, false);
    if direction == super::CommandDirection::ClientToServer {
        access.invoke = true;
    }
    if is_fabric_scoped.as_deref() == Some("true") {
        access.fabric_scoped = true;
    }
    if must_use_timed_invoke.as_deref() == Some("true") {
        access.timed = true;
    }

    Ok(Some(CommandDef {
        name: cmd_name,
        code: cmd_code,
        source: source_str,
        direction,
        response: response_opt,
        args,
        access,
        quality,
        conformance,
        choice,
        description,
    }))
}

pub(crate) fn parse_event_xml(event_xml: EventXml) -> Result<Option<EventDef>> {
    let EventXml {
        code,
        name,
        priority,
        side: _side,
        optional: _optional,
        is_fabric_sensitive,
        api_maturity: _api_maturity,
        items,
    } = event_xml;

    let Some(event_code) = parse_hex_or_dec(&code) else {
        return Ok(None);
    };
    let event_name = sanitize_identifier(&name);
    if event_name.is_empty() {
        return Ok(None);
    }
    let prio = priority.ok_or_else(|| Error::UnknownEventPriority {
        event: event_name.clone(),
        priority: String::new(),
    })?;
    match prio.trim().to_ascii_lowercase().as_str() {
        "info" | "critical" | "debug" => {}
        _ => {
            return Err(Error::UnknownEventPriority {
                event: event_name,
                priority: prio,
            });
        }
    }

    let mut description = String::new();
    let mut fields = Vec::new();
    let mut seen_field_ids = BTreeSet::new();
    let mut next_field_id = 0;
    let mut access_items = Vec::new();
    let mut conformance = Vec::new();
    let mut choice = None;

    for item in items {
        match item {
            EventItem::Description(SimpleTextXml { text }) => {
                description = text;
            }
            EventItem::Access(ax) => {
                access_items.push(ax);
            }
            EventItem::Field(field) => {
                if let Some(parsed_field) = parse_event_field_xml(*field, &mut seen_field_ids, &mut next_field_id)? {
                    fields.push(parsed_field);
                }
            }
            EventItem::MandatoryConform(c) => {
                if choice.is_none() {
                    choice = c.choice.clone();
                }
                super::conformance::compile_conform_xml(&c, &mut conformance);
                conformance.push(super::ConformanceOp::Mandatory);
            }
            EventItem::OptionalConform(c) => {
                if choice.is_none() {
                    choice = c.choice.clone();
                }
                super::conformance::compile_conform_xml(&c, &mut conformance);
                conformance.push(super::ConformanceOp::Optional);
            }
            EventItem::OtherwiseConform(o) => {
                super::conformance::compile_otherwise_conform(&o, &mut conformance);
            }
            EventItem::DisallowConform(EmptyConformXml) => {
                conformance.push(super::ConformanceOp::Disallow);
            }
            EventItem::Text(_) => {}
        }
    }

    let mut access = super::conformance::parse_access_flags(&access_items, true, false);
    if is_fabric_sensitive.as_deref() == Some("true") {
        access.fabric_sensitive = true;
    }

    Ok(Some(EventDef {
        name: event_name,
        code: event_code,
        priority: prio,
        fields,
        access,
        conformance,
        choice,
        description,
    }))
}

pub(crate) fn parse_enum_def(enm: &EnumXml) -> Result<EnumDef> {
    let EnumXml {
        name,
        type_attr,
        api_maturity: _api_maturity,
        items,
    } = enm;
    let clean_name = sanitize_identifier(name);
    let is_16bit = type_attr == "enum16";

    let mut parsed_items = Vec::new();
    let mut seen_names = BTreeSet::new();
    let mut seen_values = BTreeSet::new();
    for it in items {
        match it {
            EnumItem::Description(SimpleTextXml { text: _text }) => {}
            EnumItem::Cluster(ClusterRefXml { code: _code }) => {}
            EnumItem::Item(item_box) => {
                let ItemXml {
                    name: it_name_raw,
                    value,
                    code: _code,
                    field_id: _field_id,
                    type_attr: _type_attr,
                    is_nullable: _is_nullable,
                    is_fabric_sensitive: _is_fabric_sensitive,
                    optional: _optional,
                    writable: _writable,
                    array: _array,
                    length: _length,
                    min_length: _min_length,
                    entry_type: _entry_type,
                    min: _it_min,
                    max: _it_max,
                    default: _default,
                    defaut_typo: _defaut_typo,
                    api_maturity: _it_api_maturity,
                    items: child_items,
                } = &**item_box;
                for child in child_items {
                    match child {
                        ItemChildItem::MandatoryConform(_)
                        | ItemChildItem::OptionalConform(_)
                        | ItemChildItem::OtherwiseConform(_)
                        | ItemChildItem::DisallowConform(_)
                        | ItemChildItem::DeprecateConform(_)
                        | ItemChildItem::ProvisionalConform(_)
                        | ItemChildItem::Text(_) => {}
                    }
                }
                let it_name = sanitize_identifier(it_name_raw);
                let val = value.as_deref().and_then(parse_hex_or_dec).unwrap_or(0);
                if !it_name.is_empty() {
                    if !seen_names.insert(it_name.clone()) {
                        return Err(Error::Parse(format!(
                            "Duplicate variant name '{it_name}' in enum '{clean_name}'"
                        )));
                    }
                    if !seen_values.insert(val) {
                        return Err(Error::Parse(format!(
                            "Duplicate discriminant value {val} (variant '{it_name}') in enum '{clean_name}'"
                        )));
                    }
                    parsed_items.push((it_name, val));
                }
            }
            EnumItem::Text(_) => {}
        }
    }

    Ok(EnumDef {
        name: clean_name,
        is_16bit,
        items: parsed_items,
    })
}

pub(crate) fn parse_bitmap_def(bm: &BitmapXml) -> BitmapDef {
    let BitmapXml {
        name,
        type_attr,
        api_maturity: _api_maturity,
        items,
    } = bm;
    let clean_name = sanitize_identifier(name);
    let type_str = type_attr.clone();

    let mut fields = Vec::new();
    let mut seen_fields = BTreeSet::new();
    for it in items {
        match it {
            BitmapItem::Description(SimpleTextXml { text: _text }) => {}
            BitmapItem::Cluster(ClusterRefXml { code: _code }) => {}
            BitmapItem::Field(field_box) => {
                let FieldXml {
                    name: f_name_raw,
                    type_attr: _type_attr,
                    mask,
                    id: _id,
                    field_id: _field_id,
                    optional: _optional,
                    array: _array,
                    is_nullable: _is_nullable,
                    length: _length,
                    min_length: _min_length,
                    entry_type: _entry_type,
                    min: _f_min,
                    max: _f_max,
                    default: _default,
                    api_maturity: _f_api_maturity,
                } = &**field_box;
                let f_name = sanitize_identifier(f_name_raw);
                let mask_val = mask.as_deref().and_then(parse_hex_or_dec_u64).unwrap_or(0);
                if !f_name.is_empty() && mask_val > 0 && seen_fields.insert(f_name.clone()) {
                    fields.push((f_name, mask_val));
                }
            }
            BitmapItem::Text(_) => {}
        }
    }

    BitmapDef {
        name: clean_name,
        type_str,
        fields,
    }
}

pub(crate) fn parse_struct_def(st: &StructXml) -> Result<StructDef> {
    let StructXml {
        name,
        is_fabric_scoped,
        api_maturity: _api_maturity,
        items,
    } = st;
    let clean_name = sanitize_identifier(name);
    let is_fabric = is_fabric_scoped.as_deref() == Some("true");

    let mut description = String::new();
    let mut fields = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut next_id = 0;

    for it in items {
        match it {
            StructItem::Description(SimpleTextXml { text }) => {
                description = text.clone();
            }
            StructItem::Cluster(ClusterRefXml { code: _code }) => {}
            StructItem::Item(item_box) => {
                let ItemXml {
                    name: it_name_raw,
                    value,
                    code,
                    field_id,
                    type_attr,
                    is_nullable,
                    is_fabric_sensitive: _is_fabric_sensitive,
                    optional,
                    writable: _writable,
                    array,
                    length,
                    min_length,
                    entry_type,
                    min,
                    max,
                    default: _default,
                    defaut_typo: _defaut_typo,
                    api_maturity: _it_api_maturity,
                    items: child_items,
                } = &**item_box;
                let (field_conformance, field_choice) = parse_struct_field_conformance(child_items);
                let it_name = sanitize_identifier(it_name_raw);
                if it_name.is_empty() {
                    continue;
                }
                let fid = if let Some(parsed) = field_id
                    .as_deref()
                    .or(code.as_deref())
                    .or(value.as_deref())
                    .and_then(parse_hex_or_dec)
                {
                    if seen_ids.insert(parsed) {
                        parsed
                    } else {
                        next_free_field_id(&mut seen_ids, next_id, &format!("struct field '{it_name}'"))?
                    }
                } else {
                    next_free_field_id(&mut seen_ids, next_id, &format!("struct field '{it_name}'"))?
                };
                next_id = fid.saturating_add(1);
                let type_str = type_attr.clone().unwrap_or_default();
                let is_array = array.as_deref() == Some("true") || type_str == "array";
                let is_null = is_nullable.as_deref() == Some("true");
                let is_opt = optional.as_deref() == Some("true");
                let length_val = length.as_deref().and_then(parse_hex_or_dec);
                let min_length_val = min_length.as_deref().and_then(parse_hex_or_dec);
                let list_length =
                    (is_array || min_length_val.is_some() || length_val.is_some()).then(|| super::ListLength {
                        min: min_length_val,
                        max: length_val,
                        item_constraint: entry_type.clone(),
                    });

                fields.push(StructFieldDef {
                    field_id: fid,
                    name: it_name,
                    type_str,
                    entry_type: entry_type.clone(),
                    is_array,
                    is_nullable: is_null,
                    is_optional: is_opt,
                    min: min.clone(),
                    max: max.clone(),
                    length: length_val,
                    min_length: min_length_val,
                    list_length,
                    conformance: field_conformance,
                    choice: field_choice,
                });
            }
            StructItem::Text(_) => {}
        }
    }

    Ok(StructDef {
        name: clean_name,
        description,
        is_fabric_scoped: is_fabric,
        fields,
    })
}

pub(super) fn enum_cluster_codes(enm: &EnumXml) -> Vec<u32> {
    let mut codes = Vec::new();
    for it in &enm.items {
        match it {
            EnumItem::Cluster(ClusterRefXml { code }) => {
                if let Some(cluster_code) = parse_hex_or_dec(code) {
                    codes.push(cluster_code);
                }
            }
            EnumItem::Description(_) | EnumItem::Item(_) | EnumItem::Text(_) => {}
        }
    }
    codes
}

pub(super) fn bitmap_cluster_codes(bm: &BitmapXml) -> Vec<u32> {
    let mut codes = Vec::new();
    for it in &bm.items {
        match it {
            BitmapItem::Cluster(ClusterRefXml { code }) => {
                if let Some(cluster_code) = parse_hex_or_dec(code) {
                    codes.push(cluster_code);
                }
            }
            BitmapItem::Description(_) | BitmapItem::Field(_) | BitmapItem::Text(_) => {}
        }
    }
    codes
}

pub(super) fn struct_cluster_codes(st: &StructXml) -> Vec<u32> {
    let mut codes = Vec::new();
    for it in &st.items {
        match it {
            StructItem::Cluster(ClusterRefXml { code }) => {
                if let Some(cluster_code) = parse_hex_or_dec(code) {
                    codes.push(cluster_code);
                }
            }
            StructItem::Description(_) | StructItem::Item(_) | StructItem::Text(_) => {}
        }
    }
    codes
}

fn next_free_field_id(seen_ids: &mut BTreeSet<u32>, mut next_id: u32, context: &str) -> Result<u32> {
    while !seen_ids.insert(next_id) {
        next_id = next_id
            .checked_add(1)
            .ok_or_else(|| Error::Parse(format!("Field id overflow on {context}")))?;
    }
    Ok(next_id)
}

fn parse_command_arg_xml(
    arg: ArgXml,
    seen_arg_ids: &mut BTreeSet<u32>,
    next_arg_id: &mut u32,
) -> Result<Option<CommandArgDef>> {
    let ArgXml {
        name: arg_name_raw,
        type_attr,
        id,
        field_id,
        optional,
        array,
        is_nullable,
        length,
        lenght_typo,
        min_length,
        entry_type,
        min,
        max,
        ..
    } = arg;
    let arg_name = sanitize_identifier(&arg_name_raw);
    if arg_name.is_empty() {
        return Ok(None);
    }
    let fid = if let Some(raw_id) = field_id.as_deref().or(id.as_deref()) {
        let parsed = parse_hex_or_dec(raw_id)
            .ok_or_else(|| Error::Parse(format!("Invalid field id '{raw_id}' on command arg '{arg_name}'")))?;
        if seen_arg_ids.insert(parsed) {
            parsed
        } else {
            // Some CSA XMLs (e.g. v1.6 Thermostat AtomicRequest/AtomicResponse) repeat id="0" on all args.
            next_free_field_id(seen_arg_ids, *next_arg_id, &format!("command arg '{arg_name}'"))?
        }
    } else {
        next_free_field_id(seen_arg_ids, *next_arg_id, &format!("command arg '{arg_name}'"))?
    };
    *next_arg_id = fid.saturating_add(1);
    let is_array = super::identifiers::parse_opt_bool(array.as_deref(), &format!("command arg '{arg_name}' array"))?
        || type_attr.eq_ignore_ascii_case("array");
    let is_nullable =
        super::identifiers::parse_opt_bool(is_nullable.as_deref(), &format!("command arg '{arg_name}' isNullable"))?;
    let is_optional =
        super::identifiers::parse_opt_bool(optional.as_deref(), &format!("command arg '{arg_name}' optional"))?;
    let length_val = length.as_deref().or(lenght_typo.as_deref()).and_then(parse_hex_or_dec);
    let min_length_val = min_length.as_deref().and_then(parse_hex_or_dec);
    let list_length = (is_array || min_length_val.is_some() || length_val.is_some()).then(|| super::ListLength {
        min: min_length_val,
        max: length_val,
        item_constraint: entry_type.clone(),
    });

    Ok(Some(CommandArgDef {
        field_id: fid,
        name: arg_name,
        type_str: type_attr,
        entry_type,
        is_array,
        is_nullable,
        is_optional,
        min,
        max,
        length: length_val,
        min_length: min_length_val,
        list_length,
        conformance: Vec::new(),
        choice: None,
    }))
}

fn parse_event_field_xml(
    field: FieldXml,
    seen_field_ids: &mut BTreeSet<u32>,
    next_field_id: &mut u32,
) -> Result<Option<EventFieldDef>> {
    let FieldXml {
        name: f_name_raw,
        type_attr,
        id,
        field_id,
        optional,
        array,
        is_nullable,
        length,
        min_length,
        entry_type,
        min,
        max,
        ..
    } = field;
    let field_name = sanitize_identifier(&f_name_raw);
    if field_name.is_empty() {
        return Ok(None);
    }
    let fid = if let Some(parsed) = field_id.as_deref().or(id.as_deref()).and_then(parse_hex_or_dec) {
        if seen_field_ids.insert(parsed) {
            parsed
        } else {
            next_free_field_id(seen_field_ids, *next_field_id, &format!("event field '{field_name}'"))?
        }
    } else {
        next_free_field_id(seen_field_ids, *next_field_id, &format!("event field '{field_name}'"))?
    };
    *next_field_id = fid.saturating_add(1);
    let type_str = type_attr.unwrap_or_default();
    let is_array = array.as_deref() == Some("true") || type_str == "array";
    let is_nullable = is_nullable.as_deref() == Some("true");
    let is_optional = optional.as_deref() == Some("true");
    let length_val = length.as_deref().and_then(parse_hex_or_dec);
    let min_length_val = min_length.as_deref().and_then(parse_hex_or_dec);
    let list_length = (is_array || min_length_val.is_some() || length_val.is_some()).then(|| super::ListLength {
        min: min_length_val,
        max: length_val,
        item_constraint: entry_type.clone(),
    });

    Ok(Some(EventFieldDef {
        field_id: fid,
        name: field_name,
        type_str,
        entry_type,
        is_array,
        is_nullable,
        is_optional,
        min,
        max,
        length: length_val,
        min_length: min_length_val,
        list_length,
        conformance: Vec::new(),
        choice: None,
    }))
}

fn parse_struct_field_conformance(child_items: &[ItemChildItem]) -> (Vec<super::ConformanceOp>, Option<String>) {
    let mut conformance = Vec::new();
    let mut choice = None;
    for child in child_items {
        match child {
            ItemChildItem::MandatoryConform(c) => {
                if choice.is_none() {
                    choice.clone_from(&c.choice);
                }
                super::conformance::compile_conform_xml(c, &mut conformance);
                conformance.push(super::ConformanceOp::Mandatory);
            }
            ItemChildItem::OptionalConform(c) => {
                if choice.is_none() {
                    choice.clone_from(&c.choice);
                }
                super::conformance::compile_conform_xml(c, &mut conformance);
                conformance.push(super::ConformanceOp::Optional);
            }
            ItemChildItem::OtherwiseConform(o) => {
                super::conformance::compile_otherwise_conform(o, &mut conformance);
            }
            ItemChildItem::DisallowConform(EmptyConformXml) => {
                conformance.push(super::ConformanceOp::Disallow);
            }
            ItemChildItem::DeprecateConform(EmptyConformXml) => {
                conformance.push(super::ConformanceOp::Deprecated);
            }
            ItemChildItem::ProvisionalConform(EmptyConformXml) => {
                conformance.push(super::ConformanceOp::Provisional);
            }
            ItemChildItem::Text(_) => {}
        }
    }
    (conformance, choice)
}
