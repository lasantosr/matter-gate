use std::fmt;

use heck::{ToKebabCase as _, ToShoutySnakeCase as _, ToSnakeCase as _, ToUpperCamelCase as _};

use crate::{error::Error, model::Result};

pub(crate) fn sanitize_identifier(input: &str) -> String {
    let clean: String = input
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch == '/' || ch == '-' || ch == ' ' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if clean.is_empty() {
        return "Unknown".to_owned();
    }
    let pascal = clean.to_upper_camel_case();
    if pascal.is_empty() {
        return "Unknown".to_owned();
    }
    let mut chars = pascal.chars();
    let first = chars.next().unwrap_or('_');
    if first.is_ascii_digit() {
        format!("V{pascal}")
    } else {
        pascal
    }
}

pub(crate) fn to_snake_case(input: &str) -> String {
    input.to_snake_case()
}

pub(crate) fn to_kebab_case(input: &str) -> String {
    input.to_kebab_case()
}

pub(crate) fn to_shouty_snake_case(input: &str) -> String {
    input.to_shouty_snake_case()
}

pub(crate) fn format_dec_with_underscores(val: i128) -> String {
    let val_str = val.to_string();
    let is_negative = val_str.starts_with('-');
    let digits = if is_negative { &val_str[1..] } else { &val_str[..] };
    let mut result = String::new();
    let len = digits.len();
    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (len.saturating_sub(idx)) % 3 == 0 {
            result.push('_');
        }
        result.push(ch);
    }
    if is_negative { format!("-{result}") } else { result }
}

pub(crate) fn parse_hex_or_dec(raw: &str) -> Option<u32> {
    let trimmed = raw.trim();
    if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        trimmed.parse::<u32>().ok()
    }
}

pub(crate) fn parse_hex_or_dec_u64(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        trimmed.parse::<u64>().ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ParsedBound {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl ParsedBound {
    pub(crate) fn should_emit_min(&self) -> bool {
        match self {
            Self::U8(v) => *v > 0,
            Self::U16(v) => *v > 0,
            Self::U32(v) => *v > 0,
            Self::U64(v) => *v > 0,
            Self::I8(v) => *v > i8::MIN,
            Self::I16(v) => *v > i16::MIN,
            Self::I32(v) => *v > i32::MIN,
            Self::I64(v) => *v > i64::MIN,
            Self::F32(v) => v.is_finite(),
            Self::F64(v) => v.is_finite(),
        }
    }

    pub(crate) fn should_emit_max(&self) -> bool {
        match self {
            Self::U8(v) => *v < u8::MAX,
            Self::U16(v) => *v < u16::MAX,
            Self::U32(v) => *v < u32::MAX,
            Self::U64(v) => *v < u64::MAX,
            Self::I8(v) => *v < i8::MAX,
            Self::I16(v) => *v < i16::MAX,
            Self::I32(v) => *v < i32::MAX,
            Self::I64(v) => *v < i64::MAX,
            Self::F32(v) => v.is_finite(),
            Self::F64(v) => v.is_finite(),
        }
    }
}

impl fmt::Display for ParsedBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(v) => write!(f, "{}", format_bound_num(i128::from(*v), "u8")),
            Self::U16(v) => write!(f, "{}", format_bound_num(i128::from(*v), "u16")),
            Self::U32(v) => write!(f, "{}", format_bound_num(i128::from(*v), "u32")),
            Self::U64(v) => write!(f, "{}", format_bound_num(i128::from(*v), "u64")),
            Self::I8(v) => write!(f, "{}", format_bound_num(i128::from(*v), "i8")),
            Self::I16(v) => write!(f, "{}", format_bound_num(i128::from(*v), "i16")),
            Self::I32(v) => write!(f, "{}", format_bound_num(i128::from(*v), "i32")),
            Self::I64(v) => write!(f, "{}", format_bound_num(i128::from(*v), "i64")),
            Self::F32(v) => write!(f, "{v}_f32"),
            Self::F64(v) => write!(f, "{v}_f64"),
        }
    }
}

pub(crate) fn parse_bool(raw: &str, context: &str) -> Result<bool> {
    match raw.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(Error::Parse(format!("Invalid boolean value '{other}' for {context}"))),
    }
}

pub(crate) fn parse_opt_bool(raw: Option<&str>, context: &str) -> Result<bool> {
    match raw {
        Some(val) => parse_bool(val, context),
        None => Ok(false),
    }
}

pub(crate) fn parse_bound_for_type(raw: &str, rust_type: &str) -> Option<ParsedBound> {
    let trimmed = raw.trim();
    if let Some(magnitude) = trimmed.strip_prefix("-0x").or_else(|| trimmed.strip_prefix("-0X")) {
        let negated = i128::try_from(u128::from_str_radix(magnitude, 16).ok()?)
            .ok()?
            .checked_neg()?;
        return match rust_type {
            "i8" => i8::try_from(negated).ok().map(ParsedBound::I8),
            "i16" => i16::try_from(negated).ok().map(ParsedBound::I16),
            "i32" => i32::try_from(negated).ok().map(ParsedBound::I32),
            "i64" => i64::try_from(negated).ok().map(ParsedBound::I64),
            _ => None,
        };
    }
    let is_hex = trimmed.starts_with("0x") || trimmed.starts_with("0X");
    let hex_part = if is_hex { &trimmed[2..] } else { trimmed };

    match rust_type {
        "u8" => {
            let val = if is_hex {
                u8::from_str_radix(hex_part, 16).ok()?
            } else {
                trimmed.parse::<u8>().ok()?
            };
            Some(ParsedBound::U8(val))
        }
        "u16" => {
            let val = if is_hex {
                u16::from_str_radix(hex_part, 16).ok()?
            } else {
                trimmed.parse::<u16>().ok()?
            };
            Some(ParsedBound::U16(val))
        }
        "u32" => {
            let val = if is_hex {
                u32::from_str_radix(hex_part, 16).ok()?
            } else {
                trimmed.parse::<u32>().ok()?
            };
            Some(ParsedBound::U32(val))
        }
        "u64" => {
            let val = if is_hex {
                u64::from_str_radix(hex_part, 16).ok()?
            } else {
                trimmed.parse::<u64>().ok()?
            };
            Some(ParsedBound::U64(val))
        }
        "i8" => {
            let val = if is_hex {
                let unsigned = u8::from_str_radix(hex_part, 16).ok()?;
                i8::from_be_bytes(unsigned.to_be_bytes())
            } else {
                trimmed.parse::<i8>().ok()?
            };
            Some(ParsedBound::I8(val))
        }
        "i16" => {
            let val = if is_hex {
                let unsigned = u16::from_str_radix(hex_part, 16).ok()?;
                i16::from_be_bytes(unsigned.to_be_bytes())
            } else {
                trimmed.parse::<i16>().ok()?
            };
            Some(ParsedBound::I16(val))
        }
        "i32" => {
            let val = if is_hex {
                let unsigned = u32::from_str_radix(hex_part, 16).ok()?;
                i32::from_be_bytes(unsigned.to_be_bytes())
            } else {
                trimmed.parse::<i32>().ok()?
            };
            Some(ParsedBound::I32(val))
        }
        "i64" => {
            let val = if is_hex {
                let unsigned = u64::from_str_radix(hex_part, 16).ok()?;
                i64::from_be_bytes(unsigned.to_be_bytes())
            } else {
                trimmed.parse::<i64>().ok()?
            };
            Some(ParsedBound::I64(val))
        }
        "f32" => {
            let val = trimmed.parse::<f32>().ok().filter(|v| v.is_finite())?;
            Some(ParsedBound::F32(val))
        }
        "f64" => {
            let val = trimmed.parse::<f64>().ok().filter(|v| v.is_finite())?;
            Some(ParsedBound::F64(val))
        }
        _ => None,
    }
}

fn format_bound_num(val: i128, suffix: &str) -> String {
    let abs_val = val.unsigned_abs();
    let is_neg = val < 0;
    let sign = if is_neg { "-" } else { "" };

    let is_hex_candidate = match abs_val {
        0x7FFE
        | 0x7FFF
        | 0xFFFD
        | 0xFFFE
        | 0xFFFF
        | 0x7FFF_FFFE
        | 0x7FFF_FFFF
        | 0xFFFF_FFFD
        | 0xFFFF_FFFE
        | 0xFFFF_FFFF
        | 0x3FFF_FFFF_FFFF_FFFF
        | 0x7FFF_FFFF_FFFF_FFFE
        | 0x7FFF_FFFF_FFFF_FFFF
        | 0xFFFF_FFFF_FFFF_FFFD
        | 0xFFFF_FFFF_FFFF_FFFE
        | 0xFFFF_FFFF_FFFF_FFFF => true,
        _ if abs_val >= 0x1000
            && (abs_val.is_power_of_two()
                || abs_val.checked_add(1).is_some_and(u128::is_power_of_two)
                || abs_val.checked_add(2).is_some_and(u128::is_power_of_two)) =>
        {
            true
        }
        _ => false,
    };

    if is_hex_candidate {
        let hex_str = format!("{abs_val:X}");
        let mut grouped_hex = String::new();
        let len = hex_str.len();
        for (idx, ch) in hex_str.chars().enumerate() {
            if idx > 0 && (len.saturating_sub(idx)) % 4 == 0 {
                grouped_hex.push('_');
            }
            grouped_hex.push(ch);
        }
        format!("{sign}0x{grouped_hex}_{suffix}")
    } else if abs_val >= 10_000 {
        let dec = format_dec_with_underscores(val);
        format!("{dec}_{suffix}")
    } else {
        format!("{val}_{suffix}")
    }
}
