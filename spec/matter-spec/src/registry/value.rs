//! Wire [`AttributeValue`] type, bounds, list, struct-payload, and global-attribute validation helpers.

use super::{
    IntegerMode, MAX_STACK_MEMBERS, StackBitset,
    conformance::eval_conformance,
    descriptors::{ConformanceResult, FieldDescriptor, ValueBounds, ValueConstraint, ValueType},
};
use crate::model::{AttributeId, AttributeValue, Cluster, FieldId, MatterStatus};

/// Validates a wire [`AttributeValue`] against a [`ValueConstraint`], returning the innermost
/// failing [`FieldId`] and normative [`MatterStatus`].
#[cfg(test)]
pub(crate) fn validate_value(
    value: &AttributeValue,
    constraint: &ValueConstraint,
) -> Result<(), (Option<FieldId>, MatterStatus)> {
    validate_value_with_ctx(value, constraint, 0, None, IntegerMode::Widening)
}

/// Validates `value` against `constraint` using cluster feature/attribute context and `int_mode` integer rules.
pub(super) fn validate_value_with_ctx(
    value: &AttributeValue,
    constraint: &ValueConstraint,
    feature_map: u32,
    cluster: Option<&Cluster>,
    int_mode: IntegerMode,
) -> Result<(), (Option<FieldId>, MatterStatus)> {
    if matches!(value, AttributeValue::Null) {
        return if constraint.nullable {
            Ok(())
        } else {
            Err((None, MatterStatus::InvalidDataType))
        };
    }

    match constraint.value_type {
        #[cfg(test)]
        ValueType::Any => Ok(()),
        ValueType::Bool => match value {
            AttributeValue::Bool(_) => Ok(()),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::U8 => check_unsigned(validate_unsigned_int::<u8>(value, 8, int_mode)?, constraint.bounds),
        ValueType::U16 => check_unsigned(validate_unsigned_int::<u16>(value, 16, int_mode)?, constraint.bounds),
        ValueType::U32 => check_unsigned(validate_unsigned_int::<u32>(value, 32, int_mode)?, constraint.bounds),
        ValueType::U64 => check_unsigned(validate_unsigned_int::<u64>(value, 64, int_mode)?, constraint.bounds),
        ValueType::I8 => check_signed(validate_signed_int::<i8>(value, 8, int_mode)?, constraint.bounds),
        ValueType::I16 => check_signed(validate_signed_int::<i16>(value, 16, int_mode)?, constraint.bounds),
        ValueType::I32 => check_signed(validate_signed_int::<i32>(value, 32, int_mode)?, constraint.bounds),
        ValueType::I64 => check_signed(validate_signed_int::<i64>(value, 64, int_mode)?, constraint.bounds),
        ValueType::F32 => match value {
            AttributeValue::F32(_) => Ok(()),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::F64 => match value {
            AttributeValue::F64(_) => Ok(()),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::String => match value {
            AttributeValue::String(s) => match constraint.bounds {
                ValueBounds::None => Ok(()),
                ValueBounds::StringLen { min, max } => {
                    if s.len() < min || s.len() > max {
                        Err((None, MatterStatus::ConstraintError))
                    } else {
                        Ok(())
                    }
                }
                _ => Err((None, MatterStatus::ConstraintError)),
            },
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::OctetString => match value {
            AttributeValue::Bytes(b) => match constraint.bounds {
                ValueBounds::None => Ok(()),
                ValueBounds::BytesLen { min, max } => {
                    if b.len() < min || b.len() > max {
                        Err((None, MatterStatus::ConstraintError))
                    } else {
                        Ok(())
                    }
                }
                _ => Err((None, MatterStatus::ConstraintError)),
            },
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::List(list_len) => match value {
            AttributeValue::List(items) => {
                if items.len() < list_len.min || items.len() > list_len.max {
                    return Err((None, MatterStatus::ConstraintError));
                }
                for item in items {
                    validate_value_with_ctx(item, list_len.item_constraint, feature_map, cluster, int_mode)?;
                }
                Ok(())
            }
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::Struct(struct_desc) => {
            validate_struct_payload(value, struct_desc.fields, false, feature_map, cluster, int_mode)
        }
        ValueType::Enum { bits, valid_values } => {
            let raw = validate_unsigned_bits(value, bits, int_mode)?;
            if valid_values.contains(&raw) {
                Ok(())
            } else {
                Err((None, MatterStatus::ConstraintError))
            }
        }
        ValueType::Bitmap { bits, valid_mask } => {
            let raw = validate_unsigned_bits(value, bits, int_mode)?;
            if (raw & !valid_mask) == 0 {
                Ok(())
            } else {
                Err((None, MatterStatus::ConstraintError))
            }
        }
    }
}

/// Validates struct/command/event `entries` against `fields` descriptors, enforcing uniqueness and conformance.
pub(super) fn validate_struct_fields(
    entries: &[(FieldId, AttributeValue)],
    fields: &[FieldDescriptor],
    is_command: bool,
    feature_map: u32,
    cluster: Option<&Cluster>,
    int_mode: IntegerMode,
) -> Result<(), (Option<FieldId>, MatterStatus)> {
    let mut seen = StackBitset::new();
    for (idx, (fid, fval)) in entries.iter().enumerate() {
        let Some((desc_idx, field_desc)) = fields.iter().enumerate().find(|(_, f)| f.id == *fid) else {
            return Err((Some(*fid), MatterStatus::ConstraintError));
        };

        check_field_duplicate(&mut seen, entries, idx, desc_idx, *fid)?;

        let conf = eval_conformance(field_desc.conformance, feature_map, cluster);
        if conf == ConformanceResult::Disallowed {
            return Err((Some(*fid), MatterStatus::ConstraintError));
        }

        if let Err((inner_fid, status)) =
            validate_value_with_ctx(fval, &field_desc.constraint, feature_map, cluster, int_mode)
        {
            return Err((inner_fid.or(Some(*fid)), status));
        }
    }

    let missing_status = if is_command {
        MatterStatus::InvalidCommand
    } else {
        MatterStatus::ConstraintError
    };
    for field in fields {
        let conf = eval_conformance(field.conformance, feature_map, cluster);
        if conf == ConformanceResult::Mandatory && !entries.iter().any(|(fid, _)| *fid == field.id) {
            return Err((Some(field.id), missing_status));
        }
    }

    Ok(())
}

/// Extracts any integer [`AttributeValue`] fitting into `i64` for numeric conformance comparisons.
pub(super) fn extract_signed(value: &AttributeValue) -> Option<i64> {
    match value {
        AttributeValue::U8(v) => Some(i64::from(*v)),
        AttributeValue::U16(v) => Some(i64::from(*v)),
        AttributeValue::U32(v) => Some(i64::from(*v)),
        AttributeValue::U64(v) => i64::try_from(*v).ok(),
        AttributeValue::I8(v) => Some(i64::from(*v)),
        AttributeValue::I16(v) => Some(i64::from(*v)),
        AttributeValue::I32(v) => Some(i64::from(*v)),
        AttributeValue::I64(v) => Some(*v),
        _ => None,
    }
}

/// Decodes a `u8` wire value according to `int_mode` (`Strict`, `Widening`, or `Fit`).
pub(super) fn decode_u8_with_mode(value: &AttributeValue, int_mode: IntegerMode) -> Result<u8, MatterStatus> {
    let raw = validate_unsigned_int::<u8>(value, 8, int_mode).map_err(|(_, status)| status)?;
    u8::try_from(raw).map_err(|_| MatterStatus::ConstraintError)
}

/// Decodes a `u16` wire value according to `int_mode` (`Strict`, `Widening`, or `Fit`).
pub(super) fn decode_u16_with_mode(value: &AttributeValue, int_mode: IntegerMode) -> Result<u16, MatterStatus> {
    let raw = validate_unsigned_int::<u16>(value, 16, int_mode).map_err(|(_, status)| status)?;
    u16::try_from(raw).map_err(|_| MatterStatus::ConstraintError)
}

/// Decodes a `u32` wire value according to `int_mode` (`Strict`, `Widening`, or `Fit`).
pub(super) fn decode_u32_with_mode(value: &AttributeValue, int_mode: IntegerMode) -> Result<u32, MatterStatus> {
    let raw = validate_unsigned_int::<u32>(value, 32, int_mode).map_err(|(_, status)| status)?;
    u32::try_from(raw).map_err(|_| MatterStatus::ConstraintError)
}

/// Validates global attributes (`0xFFF8..=0xFFFD`) against their spec wire types under `int_mode`.
pub(super) fn validate_global_attribute_value(
    id: AttributeId,
    value: &AttributeValue,
    int_mode: IntegerMode,
) -> Result<(), MatterStatus> {
    match id {
        AttributeId::GENERATED_COMMAND_LIST
        | AttributeId::ACCEPTED_COMMAND_LIST
        | AttributeId::EVENT_LIST
        | AttributeId::ATTRIBUTE_LIST => {
            let AttributeValue::List(items) = value else {
                return Err(MatterStatus::InvalidDataType);
            };
            for item in items {
                let _: u32 = decode_u32_with_mode(item, int_mode)?;
            }
            Ok(())
        }
        AttributeId::FEATURE_MAP => {
            let _: u32 = decode_u32_with_mode(value, int_mode)?;
            Ok(())
        }
        AttributeId::CLUSTER_REVISION => {
            let _: u16 = decode_u16_with_mode(value, int_mode)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_unsigned_int<T: TryFrom<u64>>(
    value: &AttributeValue,
    target_bits: u8,
    int_mode: IntegerMode,
) -> Result<u64, (Option<FieldId>, MatterStatus)> {
    let (raw, wire_bits) = match value {
        AttributeValue::U8(v) => (u64::from(*v), 8_u8),
        AttributeValue::U16(v) => (u64::from(*v), 16_u8),
        AttributeValue::U32(v) => (u64::from(*v), 32_u8),
        AttributeValue::U64(v) => (*v, 64_u8),
        _ => return Err((None, MatterStatus::InvalidDataType)),
    };
    let ok = match int_mode {
        IntegerMode::Strict => wire_bits == target_bits,
        IntegerMode::Widening => wire_bits <= target_bits,
        IntegerMode::Fit => {
            return T::try_from(raw)
                .map(|_| raw)
                .map_err(|_| (None, MatterStatus::ConstraintError));
        }
    };
    if ok {
        Ok(raw)
    } else {
        Err((None, MatterStatus::InvalidDataType))
    }
}

fn validate_signed_int<T: TryFrom<i64>>(
    value: &AttributeValue,
    target_bits: u8,
    int_mode: IntegerMode,
) -> Result<i64, (Option<FieldId>, MatterStatus)> {
    let (raw, wire_bits) = match value {
        AttributeValue::I8(v) => (i64::from(*v), 8_u8),
        AttributeValue::I16(v) => (i64::from(*v), 16_u8),
        AttributeValue::I32(v) => (i64::from(*v), 32_u8),
        AttributeValue::I64(v) => (*v, 64_u8),
        _ => return Err((None, MatterStatus::InvalidDataType)),
    };
    let ok = match int_mode {
        IntegerMode::Strict => wire_bits == target_bits,
        IntegerMode::Widening => wire_bits <= target_bits,
        IntegerMode::Fit => {
            return T::try_from(raw)
                .map(|_| raw)
                .map_err(|_| (None, MatterStatus::ConstraintError));
        }
    };
    if ok {
        Ok(raw)
    } else {
        Err((None, MatterStatus::InvalidDataType))
    }
}

fn validate_struct_payload(
    value: &AttributeValue,
    fields: &[FieldDescriptor],
    is_command: bool,
    feature_map: u32,
    cluster: Option<&Cluster>,
    int_mode: IntegerMode,
) -> Result<(), (Option<FieldId>, MatterStatus)> {
    let AttributeValue::Struct(entries) = value else {
        return Err((None, MatterStatus::InvalidDataType));
    };
    validate_struct_fields(entries, fields, is_command, feature_map, cluster, int_mode)
}

fn check_unsigned(v: u64, bounds: ValueBounds) -> Result<(), (Option<FieldId>, MatterStatus)> {
    match bounds {
        ValueBounds::None => Ok(()),
        ValueBounds::Unsigned { min, max } => {
            if v < min || v > max {
                Err((None, MatterStatus::ConstraintError))
            } else {
                Ok(())
            }
        }
        _ => Err((None, MatterStatus::ConstraintError)),
    }
}

fn check_signed(v: i64, bounds: ValueBounds) -> Result<(), (Option<FieldId>, MatterStatus)> {
    match bounds {
        ValueBounds::None => Ok(()),
        ValueBounds::Signed { min, max } => {
            if v < min || v > max {
                Err((None, MatterStatus::ConstraintError))
            } else {
                Ok(())
            }
        }
        _ => Err((None, MatterStatus::ConstraintError)),
    }
}

fn validate_unsigned_bits(
    value: &AttributeValue,
    bits: u8,
    int_mode: IntegerMode,
) -> Result<u64, (Option<FieldId>, MatterStatus)> {
    match bits {
        8 => validate_unsigned_int::<u8>(value, 8, int_mode),
        16 => validate_unsigned_int::<u16>(value, 16, int_mode),
        32 => validate_unsigned_int::<u32>(value, 32, int_mode),
        _ => validate_unsigned_int::<u64>(value, 64, int_mode),
    }
}

/// Rejects duplicate struct `FieldId`s using `seen` (with linear fallback above `MAX_STACK_MEMBERS`).
fn check_field_duplicate(
    seen: &mut StackBitset,
    entries: &[(FieldId, AttributeValue)],
    idx: usize,
    desc_idx: usize,
    fid: FieldId,
) -> Result<(), (Option<FieldId>, MatterStatus)> {
    let is_dup = if desc_idx < MAX_STACK_MEMBERS {
        !seen.insert(desc_idx)
    } else {
        entries
            .get(..idx)
            .is_some_and(|prevs| prevs.iter().any(|(prev_id, _)| *prev_id == fid))
    };
    if is_dup {
        return Err((Some(fid), MatterStatus::ConstraintError));
    }
    Ok(())
}
