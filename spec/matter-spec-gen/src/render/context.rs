use std::{
    collections::{BTreeMap, BTreeSet},
    result::Result as StdResult,
};

use genco::{
    lang::Rust,
    quote,
    tokens::{FormatInto, from_fn},
};

use crate::{
    error::Error,
    model::{
        AttributeDef, BitmapDef, ClusterDef, EnumDef, GlobalDefs, ResolvedType, StructDef, resolve_type,
        sanitize_identifier, to_snake_case,
    },
};

pub(crate) type Tokens = genco::Tokens<Rust>;

type Result<T, E = Error> = StdResult<T, E>;

pub(crate) fn to_rust_field_name(input: &str) -> String {
    let snake = to_snake_case(input);
    match snake.as_str() {
        "self" | "super" | "crate" => format!("{snake}_"),
        "type" | "match" | "move" | "fn" | "mod" | "for" | "in" | "where" | "ref" | "use" | "yield" | "loop"
        | "static" | "const" | "mut" | "as" | "box" | "pub" | "enum" | "struct" | "trait" | "impl" | "async"
        | "await" | "break" | "continue" | "return" | "else" | "if" | "gen" | "dyn" | "try" | "extern" | "unsafe"
        | "macro" | "let" | "do" | "abstract" | "become" | "final" | "override" | "priv" | "typeof" | "unsized"
        | "virtual" => format!("r#{snake}"),
        _ => snake,
    }
}

pub(crate) fn with_method_name(field_name: &str) -> String {
    format!("with_{}", field_name.strip_prefix("r#").unwrap_or(field_name))
}

/// Renders a token stream into Rust source text.
pub(crate) fn render(tokens: Tokens) -> Result<String> {
    tokens.to_string().map_err(|err| Error::Render(err.to_string()))
}

/// Returns `true` if `ty` is one of Rust's fixed-width integer primitive names (`u8`..`i64`).
pub(super) fn is_primitive_int(ty: &str) -> bool {
    matches!(ty, "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64")
}

pub(super) fn format_hex(val: u64, min_digits: usize) -> String {
    let hex = format!("{val:0min_digits$X}");
    if hex.len() <= 4 {
        format!("0x{hex}")
    } else {
        let mut chunks = Vec::new();
        let bytes = hex.as_bytes();
        let mut end = bytes.len();
        while end > 0 {
            let start = end.saturating_sub(4);
            chunks.push(&hex[start..end]);
            end = start;
        }
        chunks.reverse();
        format!("0x{}", chunks.join("_"))
    }
}

pub(super) fn format_doc_comment(raw: &str) -> String {
    let clean = raw.trim().replace('\r', "");
    if clean.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let paragraphs: Vec<&str> = clean.split("\n\n").collect();
    for (idx, paragraph) in paragraphs.iter().enumerate() {
        let trimmed = wrap_bare_urls(&paragraph.trim().replace('\n', " "));
        if trimmed.is_empty() {
            continue;
        }
        if idx > 0 {
            out.push('\n');
            out.push('\n');
        }
        out.push_str(&trimmed);
        if !trimmed.ends_with('.') && !trimmed.ends_with('!') && !trimmed.ends_with('?') && !trimmed.ends_with(':') {
            out.push('.');
        }
    }
    out
}

pub(super) fn cluster_cfg_gate(cfgs: Option<&Vec<String>>) -> Tokens {
    let Some(gates) = cfgs.filter(|g| !g.is_empty()) else {
        return Tokens::new();
    };
    if let [single] = gates.as_slice() {
        return quote! {
            #[cfg($(single.as_str()))]
            $['\r']
        };
    }
    quote! {
        #[cfg(any(
            $(for gate in gates join (, ) => $(gate.as_str()))
        ))]
        $['\r']
    }
}

pub(super) fn compute_field_type_string(type_path: &str, is_optional: bool, is_nullable: bool) -> String {
    match (is_optional, is_nullable) {
        (false, false) => type_path.to_owned(),
        (false, true) => format!("Nullable<{type_path}>"),
        (true, false) => format!("Optional<{type_path}>"),
        (true, true) => format!("Optional<Nullable<{type_path}>>"),
    }
}

/// Lookup collections shared by every type-resolution call-site.
pub(super) struct TypeContext<'a> {
    pub(super) cluster_attrs: &'a [AttributeDef],
    pub(super) cluster_enums: &'a [EnumDef],
    pub(super) cluster_bitmaps: &'a [BitmapDef],
    pub(super) cluster_structs: &'a [StructDef],
    pub(super) global_defs: &'a GlobalDefs,
    pub(super) all_enums: &'a BTreeMap<String, EnumDef>,
    pub(super) all_bitmaps: &'a BTreeMap<String, BitmapDef>,
    pub(super) all_structs: &'a BTreeMap<String, StructDef>,
    pub(super) eq_structs: BTreeSet<String>,
}

impl<'a> TypeContext<'a> {
    pub(super) fn new(
        cluster_attrs: &'a [AttributeDef],
        cluster_enums: &'a [EnumDef],
        cluster_bitmaps: &'a [BitmapDef],
        cluster_structs: &'a [StructDef],
        global_defs: &'a GlobalDefs,
        all_enums: &'a BTreeMap<String, EnumDef>,
        all_bitmaps: &'a BTreeMap<String, BitmapDef>,
        all_structs: &'a BTreeMap<String, StructDef>,
    ) -> Self {
        let eq_structs = compute_eq_capable_structs(
            cluster_enums,
            cluster_bitmaps,
            cluster_structs,
            global_defs,
            all_enums,
            all_bitmaps,
            all_structs,
        );
        Self {
            cluster_attrs,
            cluster_enums,
            cluster_bitmaps,
            cluster_structs,
            global_defs,
            all_enums,
            all_bitmaps,
            all_structs,
            eq_structs,
        }
    }

    pub(super) fn resolve_type(
        &self,
        cluster_name: &str,
        context_name: &str,
        type_str: &str,
        entry_type: Option<&str>,
        is_array: bool,
    ) -> Result<ResolvedType> {
        resolve_type(
            cluster_name,
            context_name,
            type_str,
            entry_type,
            is_array,
            self.cluster_attrs,
            self.cluster_enums,
            self.cluster_bitmaps,
            self.cluster_structs,
            self.global_defs,
            self.all_enums,
            self.all_bitmaps,
            self.all_structs,
        )
    }

    pub(super) fn can_derive_eq_hash(&self, resolved: &ResolvedType) -> bool {
        resolved.can_derive_eq_hash(Some(&self.eq_structs))
    }
}

pub(super) fn resolve_cluster_id_expr(
    raw_name: &str,
    available_clusters: &[&str],
    cluster_defs: &[ClusterDef],
) -> Option<String> {
    let clean = sanitize_identifier(raw_name);
    if available_clusters.iter().any(|name| *name == clean) {
        Some(format!("{clean}Cluster::CLUSTER_ID"))
    } else if let Some(matching_cluster) = cluster_defs
        .iter()
        .find(|cluster| cluster.name.eq_ignore_ascii_case(&clean))
    {
        let hex = format_hex(u64::from(matching_cluster.code), 4);
        Some(format!("ClusterId::new({hex})"))
    } else {
        None
    }
}

/// Emits `/// ...` outer doc-comment lines for `raw`, or nothing when `raw` is blank.
pub(super) fn doc_comment(raw: &str) -> impl FormatInto<Rust> + '_ {
    from_fn(move |out| {
        for line in format_doc_comment(raw).lines() {
            out.push();
            if line.is_empty() {
                out.append("///");
            } else {
                out.append(format!("/// {line}"));
            }
        }
    })
}

/// Emits `//! ...` inner doc-comment lines for `raw`, or nothing when `raw` is blank.
pub(super) fn inner_doc_comment(raw: &str) -> impl FormatInto<Rust> + '_ {
    from_fn(move |out| {
        for line in format_doc_comment(raw).lines() {
            out.push();
            if line.is_empty() {
                out.append("//!");
            } else {
                out.append(format!("//! {line}"));
            }
        }
    })
}

/// Line-oriented appends onto a genco token stream.
pub(super) trait PushLine {
    /// Appends an empty source line.
    fn push_blank(&mut self);
}

fn wrap_bare_urls(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut words = text.split(' ').peekable();
    while let Some(word) = words.next() {
        let (prefix, rest) = if let Some(stripped) = word.strip_prefix('(') {
            ("(", stripped)
        } else {
            ("", word)
        };
        if (rest.starts_with("http://") || rest.starts_with("https://")) && !rest.ends_with('>') {
            let (url, suffix) =
                if let Some(stripped) = rest.strip_suffix(|c| c == '.' || c == ',' || c == ';' || c == ')') {
                    let last_chars = &rest[stripped.len()..];
                    (stripped, last_chars)
                } else {
                    (rest, "")
                };
            out.push_str(prefix);
            out.push('<');
            out.push_str(url);
            out.push('>');
            out.push_str(suffix);
        } else {
            out.push_str(word);
        }
        if words.peek().is_some() {
            out.push(' ');
        }
    }
    out
}

fn are_all_fields_eq_capable(
    strct: &StructDef,
    current_eq: &BTreeSet<String>,
    cluster_enums: &[EnumDef],
    cluster_bitmaps: &[BitmapDef],
    cluster_structs: &[StructDef],
    global_defs: &GlobalDefs,
    all_enums: &BTreeMap<String, EnumDef>,
    all_bitmaps: &BTreeMap<String, BitmapDef>,
    all_structs: &BTreeMap<String, StructDef>,
) -> bool {
    for field in &strct.fields {
        let Ok(resolved) = resolve_type(
            &strct.name,
            &field.name,
            &field.type_str,
            field.entry_type.as_deref(),
            field.is_array,
            &[],
            cluster_enums,
            cluster_bitmaps,
            cluster_structs,
            global_defs,
            all_enums,
            all_bitmaps,
            all_structs,
        ) else {
            return false;
        };
        if !resolved.can_derive_eq_hash(Some(current_eq)) {
            return false;
        }
    }
    true
}

fn compute_eq_capable_structs(
    cluster_enums: &[EnumDef],
    cluster_bitmaps: &[BitmapDef],
    cluster_structs: &[StructDef],
    global_defs: &GlobalDefs,
    all_enums: &BTreeMap<String, EnumDef>,
    all_bitmaps: &BTreeMap<String, BitmapDef>,
    all_structs: &BTreeMap<String, StructDef>,
) -> BTreeSet<String> {
    // Collect all candidate struct defs (local, global, all)
    let mut candidate_structs: BTreeMap<String, &StructDef> = BTreeMap::new();
    for strct in &global_defs.structs {
        candidate_structs.insert(strct.name.clone(), strct);
    }
    for (name, strct) in all_structs {
        candidate_structs.insert(name.clone(), strct);
    }
    for strct in cluster_structs {
        candidate_structs.insert(strct.name.clone(), strct);
    }

    // Fixed-point iteration to determine which structs can derive Eq and Hash
    let mut eq_structs: BTreeSet<String> = candidate_structs.keys().cloned().collect();
    let mut changed = true;
    while changed {
        changed = false;
        let current_eq = eq_structs.clone();
        for (name, strct) in &candidate_structs {
            if !eq_structs.contains(name) {
                continue;
            }
            if !are_all_fields_eq_capable(
                strct,
                &current_eq,
                cluster_enums,
                cluster_bitmaps,
                cluster_structs,
                global_defs,
                all_enums,
                all_bitmaps,
                all_structs,
            ) {
                eq_structs.remove(name);
                changed = true;
            }
        }
    }
    eq_structs
}

impl PushLine for Tokens {
    fn push_blank(&mut self) {
        self.line();
    }
}
