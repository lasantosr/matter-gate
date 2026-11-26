//! Conservative Rust struct layout and niche estimation for `> 64B` dispatch variant boxing.

use std::collections::BTreeSet;

use super::SharedClusterContext;
use crate::model::{ClusterDef, ResolvedType, resolve_type};

/// Estimated Rust `(size, align, niches)` used to decide whether generated `dispatch` enum variants are boxed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TypeLayout {
    pub(crate) size: usize,
    pub(crate) align: usize,
    pub(crate) niches: usize,
}

impl TypeLayout {
    const ZST: Self = Self {
        size: 0,
        align: 1,
        niches: 0,
    };
}

/// Adjusts `inner` layout when wrapped in a 2-variant enum (`Optional` or `Nullable`), consuming a niche if available.
pub(super) fn wrap_enum_tag(inner: TypeLayout) -> TypeLayout {
    if inner.niches >= 1 {
        TypeLayout {
            size: inner.size,
            align: inner.align,
            niches: inner.niches.saturating_sub(1),
        }
    } else {
        TypeLayout {
            size: align_up(inner.size.saturating_add(1), inner.align),
            align: inner.align,
            niches: 254,
        }
    }
}

/// Computes `repr(Rust)`-style field-reordered struct size and alignment from field layouts.
pub(super) fn struct_layout(mut fields: Vec<TypeLayout>) -> TypeLayout {
    if fields.is_empty() {
        return TypeLayout::ZST;
    }
    fields.sort_by(|a, b| b.align.cmp(&a.align).then_with(|| b.size.cmp(&a.size)));
    let mut offset = 0_usize;
    let mut max_align = 1_usize;
    let mut max_niches = 0_usize;
    for f in fields {
        offset = align_up(offset, f.align);
        offset = offset.saturating_add(f.size);
        max_align = max_align.max(f.align);
        max_niches = max_niches.max(f.niches);
    }
    TypeLayout {
        size: align_up(offset, max_align),
        align: max_align,
        niches: max_niches,
    }
}

/// Estimates the Rust layout of a resolved Matter type, breaking cyclic struct references defensively.
pub(super) fn resolved_type_layout(
    resolved: &ResolvedType,
    cluster_def: &ClusterDef,
    shared: &SharedClusterContext,
    visiting: &mut BTreeSet<String>,
) -> TypeLayout {
    match resolved {
        ResolvedType::Primitive { rust_type, .. } => primitive_type_layout(rust_type),
        ResolvedType::IdType { rust_type, .. } => id_type_layout(rust_type),
        ResolvedType::SemanticType { rust_type, .. } => semantic_type_layout(rust_type),
        ResolvedType::Enum { is_16bit, .. } => {
            if *is_16bit {
                TypeLayout {
                    size: 2,
                    align: 2,
                    niches: 256,
                }
            } else {
                TypeLayout {
                    size: 1,
                    align: 1,
                    niches: 16,
                }
            }
        }
        ResolvedType::Bitmap { type_str, .. } => match type_str.to_ascii_lowercase().as_str() {
            "bitmap8" | "int8u" | "u8" => TypeLayout {
                size: 1,
                align: 1,
                niches: 0,
            },
            "bitmap16" | "int16u" | "u16" => TypeLayout {
                size: 2,
                align: 2,
                niches: 0,
            },
            "bitmap64" | "int64u" | "u64" => TypeLayout {
                size: 8,
                align: 8,
                niches: 0,
            },
            _ => TypeLayout {
                size: 4,
                align: 4,
                niches: 0,
            },
        },
        ResolvedType::Array { .. } => TypeLayout {
            size: 24,
            align: 8,
            niches: 1,
        },
        ResolvedType::Struct { name, .. } => struct_type_layout(name, cluster_def, shared, visiting),
    }
}

const fn align_up(val: usize, align: usize) -> usize {
    if align <= 1 {
        val
    } else if let Some(rem) = val.checked_rem(align)
        && rem != 0
    {
        val.saturating_add(align.saturating_sub(rem))
    } else {
        val
    }
}

fn primitive_type_layout(rust_type: &str) -> TypeLayout {
    match rust_type {
        "bool" => TypeLayout {
            size: 1,
            align: 1,
            niches: 254,
        },
        "u8" | "i8" => TypeLayout {
            size: 1,
            align: 1,
            niches: 0,
        },
        "u16" | "i16" => TypeLayout {
            size: 2,
            align: 2,
            niches: 0,
        },
        "u32" | "i32" | "f32" => TypeLayout {
            size: 4,
            align: 4,
            niches: 0,
        },
        "u64" | "i64" | "f64" => TypeLayout {
            size: 8,
            align: 8,
            niches: 0,
        },
        "()" => TypeLayout::ZST,
        "crate::Bytes" => TypeLayout {
            size: 32,
            align: 8,
            niches: 1,
        },
        _ => TypeLayout {
            size: 24,
            align: 8,
            niches: 1,
        },
    }
}

fn id_type_layout(rust_type: &str) -> TypeLayout {
    match rust_type {
        "FabricIndex" | "FieldId" | "ActionId" => TypeLayout {
            size: 1,
            align: 1,
            niches: 0,
        },
        "GroupId" | "EndpointId" | "VendorId" => TypeLayout {
            size: 2,
            align: 2,
            niches: 0,
        },
        "NodeId" | "FabricId" | "EventNumber" => TypeLayout {
            size: 8,
            align: 8,
            niches: 0,
        },
        _ => TypeLayout {
            size: 4,
            align: 4,
            niches: 0,
        },
    }
}

fn semantic_type_layout(rust_type: &str) -> TypeLayout {
    match rust_type {
        "types::Percent" => TypeLayout {
            size: 1,
            align: 1,
            niches: 0,
        },
        "types::Percent100ths" | "types::Temperature" | "types::TemperatureDifference" | "types::SignedTemperature" => {
            TypeLayout {
                size: 2,
                align: 2,
                niches: 0,
            }
        }
        "types::Ipv4Address" => TypeLayout {
            size: 4,
            align: 1,
            niches: 0,
        },
        "types::Ipv6Address" => TypeLayout {
            size: 16,
            align: 1,
            niches: 0,
        },
        "types::IpAddress" | "types::Ipv6Prefix" | "types::HardwareAddress" => TypeLayout {
            size: 32,
            align: 8,
            niches: 1,
        },
        "types::TimeOfDay" | "types::MatterDate" | "types::EpochS" | "types::ElapsedS" => TypeLayout {
            size: 4,
            align: 4,
            niches: 0,
        },
        _ => TypeLayout {
            size: 8,
            align: 8,
            niches: 0,
        },
    }
}

fn struct_type_layout(
    name: &str,
    cluster_def: &ClusterDef,
    shared: &SharedClusterContext,
    visiting: &mut BTreeSet<String>,
) -> TypeLayout {
    if !visiting.insert(name.to_owned()) {
        return TypeLayout {
            size: 24,
            align: 8,
            niches: 1,
        };
    }
    let st_opt = cluster_def
        .structs
        .iter()
        .chain(shared.global_defs.structs.iter())
        .chain(shared.all_structs.values())
        .find(|s| s.name.eq_ignore_ascii_case(name));
    let layout = if let Some(st) = st_opt {
        let field_layouts = st
            .fields
            .iter()
            .map(|f| {
                let base = resolve_type(
                    name,
                    &f.name,
                    &f.type_str,
                    f.entry_type.as_deref(),
                    f.is_array,
                    &cluster_def.attributes,
                    &cluster_def.enums,
                    &cluster_def.bitmaps,
                    &cluster_def.structs,
                    &shared.global_defs,
                    &shared.all_enums,
                    &shared.all_bitmaps,
                    &shared.all_structs,
                )
                .map_or(
                    TypeLayout {
                        size: 8,
                        align: 8,
                        niches: 0,
                    },
                    |r| resolved_type_layout(&r, cluster_def, shared, visiting),
                );
                let mut fl = if f.is_nullable { wrap_enum_tag(base) } else { base };
                if f.is_optional {
                    fl = wrap_enum_tag(fl);
                }
                fl
            })
            .collect();
        struct_layout(field_layouts)
    } else {
        TypeLayout {
            size: 24,
            align: 8,
            niches: 1,
        }
    };
    visiting.remove(name);
    layout
}
