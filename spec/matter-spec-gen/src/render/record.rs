use genco::{
    lang::Rust,
    quote_in,
    tokens::{FormatInto, quoted},
};

use super::context::{
    Tokens, TypeContext, cfg_feature_gate, compute_field_type_string, format_hex, to_rust_field_name, with_method_name,
};
use crate::model::{ResolvedType, to_shouty_snake_case};

/// Deterministic derivation and feature attributes for specification structs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StructAttrs {
    pub(super) copy: bool,
    pub(super) eq: bool,
    pub(super) hash: bool,
    pub(super) has_fields: bool,
}

impl StructAttrs {
    /// Attributes for unit/empty ZST structs: `Copy`, `Eq`, `Hash` (optional), `Default`, Serde, Schemars.
    pub(super) const fn unit(hash: bool) -> Self {
        Self {
            copy: true,
            eq: true,
            hash,
            has_fields: false,
        }
    }

    /// Attributes for standard data model structs: `Clone`, `PartialEq`, `Default`, Serde, Schemars.
    pub(super) const fn model(eq_hash: bool, has_fields: bool) -> Self {
        Self {
            copy: false,
            eq: eq_hash,
            hash: eq_hash,
            has_fields,
        }
    }
}

impl FormatInto<Rust> for StructAttrs {
    fn format_into(self, tokens: &mut Tokens) {
        let mut derive_names = vec!["Debug", "Clone"];
        if self.copy {
            derive_names.push("Copy");
        }
        derive_names.push("PartialEq");
        if self.eq {
            derive_names.push("Eq");
        }
        if self.hash {
            derive_names.push("Hash");
        }
        derive_names.push("Default");

        let derives_joined = derive_names.join(", ");

        quote_in! { *tokens =>
            #[derive($derives_joined)]
            #[cfg_attr(feature = $(quoted("serde")), derive(Serialize, Deserialize))]
            #[cfg_attr(feature = $(quoted("schemars")), derive(schemars::JsonSchema))]
            $(if self.has_fields =>
                #[cfg_attr(
                    all(feature = $(quoted("serde")), feature = $(quoted("deny-unknown-fields"))),
                    serde(deny_unknown_fields)
                )]
                #[cfg_attr(
                    all(feature = $(quoted("schemars")), feature = $(quoted("deny-unknown-fields"))),
                    schemars(deny_unknown_fields)
                )]
            )
        };
    }
}

/// Member presence and encoding modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct MemberFlags {
    pub(super) is_optional: bool,
    pub(super) is_nullable: bool,
    pub(super) is_array: bool,
    pub(super) is_semantic_bool: bool,
}

/// Value constraints and bound definitions for record fields.
#[derive(Debug, Clone, Default)]
pub(super) struct FieldBounds {
    pub(super) min: Option<String>,
    pub(super) max: Option<String>,
    pub(super) length: Option<u32>,
}

/// Unified domain representation of a resolved record member (cluster attribute or struct/command field).
#[derive(Debug, Clone)]
pub(super) struct ResolvedMember {
    pub(super) name: String,
    pub(super) fname: String,
    pub(super) const_name: String,
    pub(super) id: u32,
    pub(super) id_hex: String,
    pub(super) type_path: String,
    pub(super) inner_type: String,
    pub(super) field_type: String,
    pub(super) is_optional: bool,
    pub(super) is_nullable: bool,
    pub(super) is_semantic_bool: bool,
    pub(super) is_array: bool,
    pub(super) min: Option<String>,
    pub(super) max: Option<String>,
    pub(super) length: Option<u32>,
    pub(super) resolved: ResolvedType,
}

impl ResolvedMember {
    pub(super) fn for_attribute(name: String, id: u32, flags: MemberFlags, resolved: ResolvedType) -> Self {
        let fname = to_rust_field_name(&name);
        let const_name = to_shouty_snake_case(&name);
        let id_hex = format_hex(u64::from(id), 4);
        let type_path = resolved.rust_type_path(false);
        let inner_type = compute_field_type_string(&type_path, false, flags.is_nullable);
        let field_type = compute_field_type_string(&type_path, flags.is_optional, flags.is_nullable);

        Self {
            name,
            fname,
            const_name,
            id,
            id_hex,
            type_path,
            inner_type,
            field_type,
            is_optional: flags.is_optional,
            is_nullable: flags.is_nullable,
            is_semantic_bool: flags.is_semantic_bool,
            is_array: flags.is_array,
            min: None,
            max: None,
            length: None,
            resolved,
        }
    }

    pub(super) fn for_field(
        name: String,
        id: u32,
        flags: MemberFlags,
        bounds: FieldBounds,
        resolved: ResolvedType,
        in_submodule: bool,
    ) -> Self {
        let fname = to_rust_field_name(&name);
        let const_name = to_shouty_snake_case(&name);
        let id_hex = format_hex(u64::from(id), 0);
        let type_path = resolved.rust_type_path(in_submodule);
        let inner_type = compute_field_type_string(&type_path, false, flags.is_nullable);
        let field_type = compute_field_type_string(&type_path, flags.is_optional, flags.is_nullable);

        Self {
            name,
            fname,
            const_name,
            id,
            id_hex,
            type_path,
            inner_type,
            field_type,
            is_optional: flags.is_optional,
            is_nullable: flags.is_nullable,
            is_semantic_bool: false,
            is_array: flags.is_array,
            min: bounds.min,
            max: bounds.max,
            length: bounds.length,
            resolved,
        }
    }

    pub(super) fn const_ident(&self, prefix: &str) -> String {
        format!("{prefix}_{}", self.const_name)
    }

    pub(super) fn attr_const_ident(&self) -> String {
        self.const_ident("ATTR")
    }

    pub(super) fn field_const_ident(&self) -> String {
        self.const_ident("FIELD")
    }

    pub(super) fn with_method_name(&self) -> String {
        with_method_name(&self.fname)
    }

    pub(super) fn is_copy(&self) -> bool {
        self.resolved.is_copy()
    }

    pub(super) fn schemars_with_type(&self) -> Option<&'static str> {
        match self.type_path.as_str() {
            "crate::Bytes" => Some(if self.is_nullable { "Option<Vec<u8>>" } else { "Vec<u8>" }),
            "Vec<crate::Bytes>" => Some(if self.is_nullable {
                "Option<Vec<Vec<u8>>>"
            } else {
                "Vec<Vec<u8>>"
            }),
            _ => None,
        }
    }
}

/// Partition of record members into mandatory and optional sets.
#[derive(Debug)]
pub(super) struct MemberPartition<'a> {
    pub(super) mandatory: Vec<&'a ResolvedMember>,
    pub(super) optional: Vec<&'a ResolvedMember>,
    pub(super) all: &'a [ResolvedMember],
}

impl<'a> MemberPartition<'a> {
    pub(super) fn new(members: &'a [ResolvedMember]) -> Self {
        let mandatory: Vec<&'a ResolvedMember> = members.iter().filter(|m| !m.is_optional).collect();
        let optional: Vec<&'a ResolvedMember> = members.iter().filter(|m| m.is_optional).collect();
        Self {
            mandatory,
            optional,
            all: members,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.all.is_empty()
    }

    pub(super) fn can_derive_eq_hash(&self, ctx: &TypeContext<'_>) -> bool {
        self.all.iter().all(|m| ctx.can_derive_eq_hash(&m.resolved))
    }

    pub(super) fn short_field_count(&self) -> usize {
        self.all.iter().filter(|m| m.fname.len() == 1).count()
    }

    pub(super) fn has_many_single_chars(&self) -> bool {
        self.short_field_count() >= 5
    }

    pub(super) fn has_too_many_args(&self) -> bool {
        self.mandatory.len() > 15
    }
}

/// Prefix and type used for associated constant identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IdPrefix {
    Attr,
    Field,
}

impl IdPrefix {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Attr => "ATTR",
            Self::Field => "FIELD",
        }
    }

    pub(super) fn type_name(self) -> &'static str {
        match self {
            Self::Attr => "AttributeId",
            Self::Field => "FieldId",
        }
    }
}

/// Renders the main struct definition (Option A flat layout: mandatory + `Optional<T>` fields directly on the struct).
pub(super) struct RecordStructDef<'a> {
    pub(super) doc: Tokens,
    pub(super) name: &'a str,
    pub(super) partition: &'a MemberPartition<'a>,
    pub(super) attrs: StructAttrs,
    pub(super) cfg_gate: Option<&'a Vec<String>>,
}

impl FormatInto<Rust> for RecordStructDef<'_> {
    fn format_into(self, tokens: &mut Tokens) {
        let name = self.name;
        let partition = self.partition;
        let doc = self.doc;
        let attrs = self.attrs;
        let gate = cfg_feature_gate(self.cfg_gate);

        if partition.is_empty() {
            quote_in! { *tokens =>
                $doc
                $gate
                $attrs
                pub struct $name;
            };
        } else {
            quote_in! { *tokens =>
                $doc
                $gate
                $attrs
                pub struct $name {
                    $(for mand in &partition.mandatory join ($['\r']) =>
                        $(if let Some(with_type) = mand.schemars_with_type() =>
                            #[cfg_attr(feature = $(quoted("schemars")), schemars(with = $(quoted(with_type))))]
                        )
                        pub $(&mand.fname): $(&mand.field_type),
                    )
                    $(for opt in &partition.optional join ($['\r']) =>
                        $(if let Some(with_type) = opt.schemars_with_type() =>
                            #[cfg_attr(feature = $(quoted("schemars")), schemars(with = $(quoted(with_type))))]
                        )
                        $(OptionalSerdeAttr(opt))
                        pub $(&opt.fname): $(&opt.field_type),
                    )
                }
            };
        }
    }
}

/// Renders standard associated items on the record struct:
/// ID constants, `NONE`, `new(...)`, and `with_<field>` builder methods.
pub(super) struct RecordConstructors<'a> {
    pub(super) partition: &'a MemberPartition<'a>,
    pub(super) prefix: IdPrefix,
    pub(super) too_many_args_reason: &'static str,
    pub(super) single_char_reason: Option<&'static str>,
}

impl FormatInto<Rust> for RecordConstructors<'_> {
    fn format_into(self, tokens: &mut Tokens) {
        let partition = self.partition;
        let prefix = self.prefix;
        let too_many_args_reason = self.too_many_args_reason;
        let type_name = prefix.type_name();
        let prefix_str = prefix.as_str();

        quote_in! { *tokens =>
            $(for member in partition.all join ($['\r']) =>
                pub const $(member.const_ident(prefix_str)): $type_name = $type_name::new($(&member.id_hex));
            )

            $(if partition.mandatory.is_empty() && !partition.optional.is_empty() =>
                pub const NONE: Self = Self {
                    $(for opt in &partition.optional join ($['\r']) =>
                        $(&opt.fname): Optional::Omitted,
                    )
                };
            )
            $(if partition.mandatory.is_empty() && partition.optional.is_empty() =>
                pub const NONE: Self = Self;
            )

            $(if partition.mandatory.is_empty() =>
                #[must_use]
                pub $(if partition.optional.is_empty() => const) fn new() -> Self {
                    $(if partition.optional.is_empty() => Self)
                    $(if !partition.optional.is_empty() => Self::default())
                }
            )

            $(if !partition.mandatory.is_empty() =>
                #[must_use]
                $(if let Some(reason) = self.single_char_reason =>
                    $(if partition.has_many_single_chars() =>
                        #[expect(clippy::many_single_char_names, reason = $(quoted(reason)))]
                    )
                )
                $(if partition.has_too_many_args() =>
                    #[expect(clippy::too_many_arguments, reason = $(quoted(too_many_args_reason)))]
                )
                pub fn new(
                    $(for mand in &partition.mandatory join ($['\r']) =>
                        $(&mand.fname): impl Into<$(&mand.field_type)>,
                    )
                ) -> Self {
                    Self {
                        $(for mand in &partition.mandatory join ($['\r']) =>
                            $(&mand.fname): $(&mand.fname).into(),
                        )
                        $(for opt in &partition.optional join ($['\r']) =>
                            $(&opt.fname): Optional::Omitted,
                        )
                    }
                }
            )

            $(for opt in &partition.optional join ($['\r']) =>
                #[must_use]
                pub fn $(opt.with_method_name())(
                    mut self,
                    $(&opt.fname): impl Into<$(&opt.inner_type)>,
                ) -> Self {
                    self.$(&opt.fname) = Optional::Present($(&opt.fname).into());
                    self
                }
            )
        };
    }
}

/// Renders single-mandatory `From` trait implementations when a record has exactly 1 mandatory field.
pub(super) struct SingleMandatoryFrom<'a> {
    pub(super) target_type: &'a str,
    pub(super) mand: &'a ResolvedMember,
    pub(super) cfg_gate: Option<&'a Vec<String>>,
}

impl<'a> SingleMandatoryFrom<'a> {
    pub(super) fn maybe(target_type: &'a str, mandatory: &'a [&'a ResolvedMember]) -> Option<Self> {
        Self::maybe_gated(target_type, mandatory, None)
    }

    pub(super) fn maybe_gated(
        target_type: &'a str,
        mandatory: &'a [&'a ResolvedMember],
        cfg_gate: Option<&'a Vec<String>>,
    ) -> Option<Self> {
        match mandatory {
            [single] => Some(Self {
                target_type,
                mand: single,
                cfg_gate,
            }),
            _ => None,
        }
    }
}

impl FormatInto<Rust> for SingleMandatoryFrom<'_> {
    fn format_into(self, tokens: &mut Tokens) {
        let target = self.target_type;
        let mand_type = &self.mand.field_type;
        let mand_path = &self.mand.type_path;
        let gate = cfg_feature_gate(self.cfg_gate);

        quote_in! { *tokens =>
            $(&gate)
            impl From<$mand_type> for $target {
                #[inline]
                fn from(val: $mand_type) -> Self {
                    Self::new(val)
                }
            }

            $(if self.mand.is_nullable =>
                $(&gate)
                impl From<$mand_path> for $target {
                    #[inline]
                    fn from(val: $mand_path) -> Self {
                        Self::new(val)
                    }
                }

                $(&gate)
                impl From<Option<$mand_path>> for $target {
                    #[inline]
                    fn from(val: Option<$mand_path>) -> Self {
                        Self::new(val)
                    }
                }
            )

            $(if mand_path == "String" =>
                $(&gate)
                impl From<&str> for $target {
                    #[inline]
                    fn from(val: &str) -> Self {
                        Self::new(val)
                    }
                }
            )

            $(if self.mand.is_semantic_bool =>
                impl From<bool> for $target {
                    #[inline]
                    fn from(val: bool) -> Self {
                        Self::new($mand_path::from(val))
                    }
                }
            )
        };
    }
}

/// Renders `serde` and `schemars(default)` attributes for an `Optional<...>` struct field.
struct OptionalSerdeAttr<'a>(&'a ResolvedMember);

impl FormatInto<Rust> for OptionalSerdeAttr<'_> {
    fn format_into(self, tokens: &mut Tokens) {
        if self.0.is_nullable {
            quote_in! { *tokens =>
                #[cfg_attr(
                    feature = $(quoted("serde")),
                    serde(
                        default,
                        skip_serializing_if = $(quoted("Optional::is_omitted")),
                        deserialize_with = $(quoted("crate::model::serde::deserialize_optional_nullable"))
                    )
                )]
                #[cfg_attr(all(not(feature = $(quoted("serde"))), feature = $(quoted("schemars"))), schemars(default))]
            };
        } else {
            quote_in! { *tokens =>
                #[cfg_attr(
                    feature = $(quoted("serde")),
                    serde(default, skip_serializing_if = $(quoted("Optional::is_omitted")))
                )]
                #[cfg_attr(all(not(feature = $(quoted("serde"))), feature = $(quoted("schemars"))), schemars(default))]
            };
        }
    }
}
