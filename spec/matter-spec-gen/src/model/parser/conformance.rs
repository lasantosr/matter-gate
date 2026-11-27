use crate::{
    error::Error,
    xml::{
        AccessXml, AndTermItem, ConformItem, ConformXml, GreaterOrEqualTermItem, GreaterTermItem, NotTermItem,
        OrTermItem, OtherwiseConformItem, OtherwiseConformXml,
    },
};

/// Returns `true` when an unresolved `DeviceTypeIncludeItem` target (`requireAttribute`,
/// `requireCommand`, or `requireEvent`) is a known upstream `matter-devices.xml` quirk for `version`.
///
/// Rule for adding entries here (checked *only* after `resolve_include_target` fails):
/// 1. **Fix or alias in `resolve_include_target` instead** if the member exists in that release's cluster XML under a
///    different casing, `@define`, or clear 1:1 rename.
/// 2. **Add here (scoped to the exact affected `version`s) only** when verified in `xtask/spec-data/` that the target
///    is genuinely absent from that release's cluster XML (stale ZCL names, cluster-level placeholders, or upstream
///    typos on example device types).
pub(crate) fn is_legacy_device_include_ignored(version: &str, target: &str) -> bool {
    match target {
        "Scenes"
        | "ColorControl"
        | "BarrierControl"
        | "MoveColorTemperature"
        | "IdentifyQuery"
        | "SetCredentialStatus"
        | "SetCredentialStatusResponse" => matches!(version, "1.0" | "1.1" | "1.2" | "1.3" | "1.4" | "1.5" | "1.6"),
        "COLOR_CONTROL_COLOR_START_ENHANCED_HUE" | "COLOR_CONTROL_COLOR_STORED_ENHANCED_HUE" => {
            matches!(version, "1.1" | "1.2" | "1.3" | "1.4" | "1.5" | "1.6")
        }
        "BINDING" => matches!(version, "1.2" | "1.3" | "1.4" | "1.5" | "1.6"),
        "SCENE_NAME_SUPPORT" | "SCENE_COUNT" | "SCENE_VALID" | "CURRENT_SCENE" | "CURRENT_GROUP" => {
            matches!(version, "1.3" | "1.4" | "1.5" | "1.6")
        }
        "COLOR_CONTROL_COLOR_TEMP_PHYSICAL_MIN" | "COLOR_CONTROL_COLOR_TEMP_PHYSICAL_MAX" => version == "1.6",
        _ => false,
    }
}

/// Privilege level for attribute/command/event access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum AccessPrivilege {
    View,
    Operate,
    Manage,
    Administer,
}

impl AccessPrivilege {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "view" => Some(Self::View),
            "operate" => Some(Self::Operate),
            "manage" => Some(Self::Manage),
            "administer" | "admin" => Some(Self::Administer),
            _ => None,
        }
    }
}

/// Parsed access control flags for an attribute, command, or event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct AccessFlags {
    pub(crate) read: bool,
    pub(crate) write: bool,
    pub(crate) invoke: bool,
    pub(crate) fabric_scoped: bool,
    pub(crate) fabric_sensitive: bool,
    pub(crate) timed: bool,
    pub(crate) read_privilege: Option<AccessPrivilege>,
    pub(crate) write_privilege: Option<AccessPrivilege>,
    pub(crate) invoke_privilege: Option<AccessPrivilege>,
}

/// Parsed quality flags for an attribute or command.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct QualityFlags {
    pub(crate) change_omitted: bool,
    pub(crate) large_message: bool,
    pub(crate) atomic_write: bool,
    pub(crate) reportable: bool,
}

pub(crate) fn parse_access_flags(
    access_items: &[AccessXml],
    default_read: bool,
    is_writable_attr: bool,
) -> AccessFlags {
    let mut flags = AccessFlags {
        read: default_read,
        write: is_writable_attr,
        ..Default::default()
    };
    for ax in access_items {
        let priv_val = ax
            .privilege
            .as_deref()
            .or(ax.role.as_deref())
            .and_then(AccessPrivilege::parse);
        match ax.op.as_deref().map(str::trim) {
            Some("read") => {
                flags.read = true;
                if let Some(p) = priv_val {
                    flags.read_privilege = Some(p);
                }
            }
            Some("write") => {
                flags.write = true;
                if let Some(p) = priv_val {
                    flags.write_privilege = Some(p);
                }
            }
            Some("invoke") => {
                flags.invoke = true;
                if let Some(p) = priv_val {
                    flags.invoke_privilege = Some(p);
                }
            }
            _ => {}
        }
        if let Some(modifier) = ax.modifier.as_deref() {
            for part in modifier.split(',').map(str::trim) {
                match part {
                    "fabric-scoped" | "fabricScoped" => flags.fabric_scoped = true,
                    "timed" => flags.timed = true,
                    _ => {}
                }
            }
        }
        if ax.fabric_sensitive.as_deref() == Some("true") {
            flags.fabric_sensitive = true;
        }
    }
    flags
}

/// List/string length constraints.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(crate) struct ListLength {
    pub(crate) min: Option<u32>,
    pub(crate) max: Option<u32>,
    pub(crate) item_constraint: Option<String>,
}

/// Command direction (`ClientToServer` vs `ServerToClient`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) enum CommandDirection {
    #[default]
    ClientToServer,
    ServerToClient,
}

pub(crate) fn parse_command_direction(command_name: &str, source: &str) -> Result<CommandDirection, Error> {
    match source.trim().to_ascii_lowercase().as_str() {
        "client" | "" => Ok(CommandDirection::ClientToServer),
        "server" => Ok(CommandDirection::ServerToClient),
        _ => Err(Error::UnknownCommandSource {
            command: command_name.to_owned(),
            source_name: source.to_owned(),
        }),
    }
}

/// Normalized RPN opcode for compiled CSA conformance expressions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ConformanceOp {
    Mandatory,
    Optional,
    Disallow,
    Provisional,
    Deprecated,
    Otherwise,
    Feature(String),
    Attribute(String),
    /// External condition name (`Zigbee`, `Matter`, etc.).
    Condition(String),
    /// Command presence reference (`[Start]`, `[Stop]`, etc.).
    Command(String),
    Literal(String),
    Not,
    And(usize),
    Or(usize),
    Greater,
    GreaterOrEqual,
}

/// Cross-member choice group (`choice`, `min`, `max`, `more`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(crate) struct ChoiceGroup {
    pub(crate) name: String,
    pub(crate) min: Option<u32>,
    pub(crate) max: Option<u32>,
    pub(crate) more: bool,
    pub(crate) members: Vec<String>,
}

pub(crate) fn compile_conform_xml(conform: &ConformXml, out: &mut Vec<ConformanceOp>) {
    for item in &conform.items {
        match item {
            ConformItem::Feature(r) => out.push(ConformanceOp::Feature(r.name.clone())),
            ConformItem::Attribute(r) => out.push(ConformanceOp::Attribute(r.name.clone())),
            ConformItem::Condition(r) => out.push(ConformanceOp::Condition(r.name.clone())),
            ConformItem::Command(r) => out.push(ConformanceOp::Command(r.name.clone())),
            ConformItem::NotTerm(n) => compile_not_term(&n.items, out),
            ConformItem::OrTerm(o) => compile_or_term(&o.items, out),
            ConformItem::AndTerm(a) => compile_and_term(&a.items, out),
            ConformItem::GreaterTerm(g) => compile_greater_term(&g.items, out),
            ConformItem::GreaterOrEqualTerm(g) => compile_greater_or_equal_term(&g.items, out),
            ConformItem::Text(_) => {}
        }
    }
}

pub(crate) fn compile_greater_or_equal_term(items: &[GreaterOrEqualTermItem], out: &mut Vec<ConformanceOp>) {
    for item in items {
        match item {
            GreaterOrEqualTermItem::Revision(rev) => {
                let val = rev.value.as_deref().or(rev.text.as_deref()).unwrap_or("").trim();
                match val.to_ascii_lowercase().as_str() {
                    "current" => out.push(ConformanceOp::Attribute("ClusterRevision".to_owned())),
                    "" => {}
                    _ => out.push(ConformanceOp::Literal(val.to_owned())),
                }
            }
            GreaterOrEqualTermItem::Text(_) => {}
        }
    }
    out.push(ConformanceOp::GreaterOrEqual);
}

pub(crate) fn compile_otherwise_conform(otherwise: &OtherwiseConformXml, out: &mut Vec<ConformanceOp>) {
    let mut branches = 0_usize;
    for item in &otherwise.items {
        let emitted = match item {
            OtherwiseConformItem::MandatoryConform(c) => {
                compile_conform_xml(c, out);
                out.push(ConformanceOp::Mandatory);
                true
            }
            OtherwiseConformItem::OptionalConform(c) => {
                compile_conform_xml(c, out);
                out.push(ConformanceOp::Optional);
                true
            }
            OtherwiseConformItem::DisallowConform(_) => {
                out.push(ConformanceOp::Disallow);
                true
            }
            OtherwiseConformItem::ProvisionalConform(_) => {
                out.push(ConformanceOp::Provisional);
                true
            }
            OtherwiseConformItem::DeprecateConform(_) => {
                out.push(ConformanceOp::Deprecated);
                true
            }
            OtherwiseConformItem::DescribedConform(_) | OtherwiseConformItem::Text(_) => false,
        };
        if emitted {
            branches = branches.saturating_add(1);
            if branches > 1 {
                out.push(ConformanceOp::Otherwise);
            }
        }
    }
}

/// Resolve a `DeviceTypeIncludeItem` target (`requireAttribute`, `requireCommand`, `requireEvent`)
/// against a list of `(define, name)` pairs using:
/// `@define` -> `name` -> `to_shouty_snake_case(name)` + documented legacy ignore table.
pub(crate) fn resolve_include_target<'a>(
    target: &str,
    candidates: &'a [(Option<&str>, &str)],
    to_shouty_snake: fn(&str) -> String,
) -> Option<&'a str> {
    for (define, name) in candidates {
        if let Some(def) = define
            && *def == target
        {
            return Some(*name);
        }
        if *name == target {
            return Some(*name);
        }
        if to_shouty_snake(name) == target {
            return Some(*name);
        }
    }
    None
}

fn compile_not_term(items: &[NotTermItem], out: &mut Vec<ConformanceOp>) {
    for item in items {
        match item {
            NotTermItem::Feature(r) => out.push(ConformanceOp::Feature(r.name.clone())),
            NotTermItem::Attribute(r) => out.push(ConformanceOp::Attribute(r.name.clone())),
            NotTermItem::OrTerm(o) => compile_or_term(&o.items, out),
            NotTermItem::Text(_) => {}
        }
    }
    out.push(ConformanceOp::Not);
}

fn compile_or_term(items: &[OrTermItem], out: &mut Vec<ConformanceOp>) {
    let mut count = 0_usize;
    for item in items {
        match item {
            OrTermItem::Feature(r) => {
                count = count.saturating_add(1);
                out.push(ConformanceOp::Feature(r.name.clone()));
            }
            OrTermItem::Attribute(r) => {
                count = count.saturating_add(1);
                out.push(ConformanceOp::Attribute(r.name.clone()));
            }
            OrTermItem::Command(r) => {
                count = count.saturating_add(1);
                out.push(ConformanceOp::Command(r.name.clone()));
            }
            OrTermItem::AndTerm(a) => {
                count = count.saturating_add(1);
                compile_and_term(&a.items, out);
            }
            OrTermItem::Text(_) => {}
        }
    }
    if count > 1 {
        out.push(ConformanceOp::Or(count));
    }
}

fn compile_and_term(items: &[AndTermItem], out: &mut Vec<ConformanceOp>) {
    let mut count = 0_usize;
    for item in items {
        match item {
            AndTermItem::Feature(r) => {
                count = count.saturating_add(1);
                out.push(ConformanceOp::Feature(r.name.clone()));
            }
            AndTermItem::Attribute(r) => {
                count = count.saturating_add(1);
                out.push(ConformanceOp::Attribute(r.name.clone()));
            }
            AndTermItem::Condition(r) => {
                count = count.saturating_add(1);
                out.push(ConformanceOp::Condition(r.name.clone()));
            }
            AndTermItem::NotTerm(n) => {
                count = count.saturating_add(1);
                compile_not_term(&n.items, out);
            }
            AndTermItem::OrTerm(o) => {
                count = count.saturating_add(1);
                compile_or_term(&o.items, out);
            }
            AndTermItem::AndTerm(a) => {
                count = count.saturating_add(1);
                compile_and_term(&a.items, out);
            }
            AndTermItem::GreaterOrEqualTerm(g) => {
                count = count.saturating_add(1);
                compile_greater_or_equal_term(&g.items, out);
            }
            AndTermItem::Text(_) => {}
        }
    }
    if count > 1 {
        out.push(ConformanceOp::And(count));
    }
}

fn compile_greater_term(items: &[GreaterTermItem], out: &mut Vec<ConformanceOp>) {
    for item in items {
        match item {
            GreaterTermItem::Attribute(r) => out.push(ConformanceOp::Attribute(r.name.clone())),
            GreaterTermItem::Literal(l) => out.push(ConformanceOp::Literal(l.value.clone())),
            GreaterTermItem::Text(_) => {}
        }
    }
    out.push(ConformanceOp::Greater);
}
