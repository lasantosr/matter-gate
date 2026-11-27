use super::access::{AccessFlags, AccessPrivilege, QualityFlags};
use crate::model::{
    AttributeId, ClusterId, CommandDirection, CommandId, DeviceTypeId, EventId, EventPriority, FieldId,
};

/// Static descriptor for a cluster attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeDescriptor {
    /// Attribute ID.
    pub(crate) id: AttributeId,
    /// Canonical `PascalCase` attribute name.
    pub(crate) name: &'static str,
    /// Rust struct `snake_case` field name.
    pub(crate) field_name: &'static str,
    /// Access flags.
    pub(crate) access: AccessFlags,
    /// Quality and transport flags.
    pub(crate) quality: QualityFlags,
    /// Required ACL privilege for reading this attribute (`None` if write-only).
    pub(crate) read_privilege: Option<AccessPrivilege>,
    /// Required ACL privilege for writing this attribute (`None` if read-only).
    pub(crate) write_privilege: Option<AccessPrivilege>,
    /// Value constraint.
    pub(crate) constraint: ValueConstraint,
    /// Conformance RPN program.
    pub(crate) conformance: &'static [ConformanceOp],
}

impl AttributeDescriptor {
    /// Returns the attribute ID.
    #[must_use]
    pub const fn id(&self) -> AttributeId {
        self.id
    }

    /// Returns the canonical `PascalCase` attribute name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the Rust struct `snake_case` field name.
    #[must_use]
    pub const fn field_name(&self) -> &'static str {
        self.field_name
    }

    /// Returns the attribute access flags.
    #[must_use]
    pub const fn access(&self) -> AccessFlags {
        self.access
    }

    /// Returns the attribute quality and transport flags.
    #[must_use]
    pub const fn quality(&self) -> QualityFlags {
        self.quality
    }

    /// Returns the required ACL privilege to read this attribute, or `None` if write-only.
    #[must_use]
    pub const fn read_privilege(&self) -> Option<AccessPrivilege> {
        self.read_privilege
    }

    /// Returns the required ACL privilege to write this attribute, or `None` if read-only.
    #[must_use]
    pub const fn write_privilege(&self) -> Option<AccessPrivilege> {
        self.write_privilege
    }

    /// Returns `true` if `AttributeValue::Null` is permitted by the specification.
    #[must_use]
    pub const fn is_nullable(&self) -> bool {
        self.constraint.nullable
    }
}

/// Static descriptor for a cluster command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDescriptor {
    /// Command ID.
    pub(crate) id: CommandId,
    /// Command name.
    pub(crate) name: &'static str,
    /// Expected command direction.
    pub(crate) direction: CommandDirection,
    /// Access flags.
    pub(crate) access: AccessFlags,
    /// Quality and transport flags.
    pub(crate) quality: QualityFlags,
    /// Required ACL privilege to invoke this command (`None` for `ServerToClient` responses).
    pub(crate) invoke_privilege: Option<AccessPrivilege>,
    /// Expected `ServerToClient` response [`CommandId`], or `None` for `DefaultResponse` / responses.
    pub(crate) response_id: Option<CommandId>,
    /// Payload struct fields (empty if no payload fields).
    pub(crate) fields: &'static [FieldDescriptor],
    /// Conformance RPN program.
    pub(crate) conformance: &'static [ConformanceOp],
}

impl CommandDescriptor {
    /// Returns the command ID.
    #[must_use]
    pub const fn id(&self) -> CommandId {
        self.id
    }

    /// Returns the command name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the expected command direction.
    #[must_use]
    pub const fn direction(&self) -> CommandDirection {
        self.direction
    }

    /// Returns the command access flags.
    #[must_use]
    pub const fn access(&self) -> AccessFlags {
        self.access
    }

    /// Returns the command quality and transport flags.
    #[must_use]
    pub const fn quality(&self) -> QualityFlags {
        self.quality
    }

    /// Returns the required ACL privilege to invoke this command, or `None` for `ServerToClient` responses.
    #[must_use]
    pub const fn invoke_privilege(&self) -> Option<AccessPrivilege> {
        self.invoke_privilege
    }

    /// Returns the expected `ServerToClient` response [`CommandId`], or `None` when the command
    /// returns a status `DefaultResponse` (or is itself a response).
    #[must_use]
    pub const fn response_id(&self) -> Option<CommandId> {
        self.response_id
    }
}

/// Static descriptor for a cluster event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventDescriptor {
    /// Event ID.
    pub(crate) id: EventId,
    /// Event name.
    pub(crate) name: &'static str,
    /// Expected event priority.
    pub(crate) priority: EventPriority,
    /// Access flags.
    pub(crate) access: AccessFlags,
    /// Required ACL privilege to read/subscribe to this event.
    pub(crate) read_privilege: AccessPrivilege,
    /// Payload struct fields.
    pub(crate) fields: &'static [FieldDescriptor],
    /// Conformance RPN program.
    pub(crate) conformance: &'static [ConformanceOp],
}

impl EventDescriptor {
    /// Returns the event ID.
    #[must_use]
    pub const fn id(&self) -> EventId {
        self.id
    }

    /// Returns the event name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the expected event priority.
    #[must_use]
    pub const fn priority(&self) -> EventPriority {
        self.priority
    }

    /// Returns the event access flags.
    #[must_use]
    pub const fn access(&self) -> AccessFlags {
        self.access
    }

    /// Returns the required ACL privilege to read/subscribe to this event.
    #[must_use]
    pub const fn read_privilege(&self) -> AccessPrivilege {
        self.read_privilege
    }
}

/// Static descriptor for a Matter cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClusterDescriptor {
    /// Cluster ID.
    pub(crate) id: ClusterId,
    /// Specification cluster revision (`ClusterRevision` `0xFFFD`).
    pub(crate) revision: u16,
    /// Cluster name.
    pub(crate) name: &'static str,
    /// Cluster attributes.
    pub(crate) attributes: &'static [AttributeDescriptor],
    /// Cluster commands.
    pub(crate) commands: &'static [CommandDescriptor],
    /// Cluster events.
    pub(crate) events: &'static [EventDescriptor],
    /// Cross-attribute choice groups.
    pub(crate) choice_groups: &'static [ChoiceGroup],
}

impl ClusterDescriptor {
    /// Returns the cluster ID.
    #[must_use]
    pub const fn id(&self) -> ClusterId {
        self.id
    }

    /// Returns the specification cluster revision (`ClusterRevision` `0xFFFD`).
    #[must_use]
    pub const fn revision(&self) -> u16 {
        self.revision
    }

    /// Returns the cluster name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the cluster attribute descriptors.
    #[must_use]
    pub const fn attributes(&self) -> &'static [AttributeDescriptor] {
        self.attributes
    }

    /// Returns the cluster command descriptors.
    #[must_use]
    pub const fn commands(&self) -> &'static [CommandDescriptor] {
        self.commands
    }

    /// Returns the cluster event descriptors.
    #[must_use]
    pub const fn events(&self) -> &'static [EventDescriptor] {
        self.events
    }

    /// Looks up an attribute descriptor by [`AttributeId`].
    #[must_use]
    pub fn find_attribute(&self, id: AttributeId) -> Option<&'static AttributeDescriptor> {
        self.attributes.iter().find(|a| a.id == id)
    }

    /// Looks up a command descriptor by [`CommandId`] and [`CommandDirection`].
    #[must_use]
    pub fn find_command(&self, id: CommandId, direction: CommandDirection) -> Option<&'static CommandDescriptor> {
        self.commands.iter().find(|c| c.id == id && c.direction == direction)
    }

    /// Looks up an event descriptor by [`EventId`].
    #[must_use]
    pub fn find_event(&self, id: EventId) -> Option<&'static EventDescriptor> {
        self.events.iter().find(|e| e.id == id)
    }
}

/// Static descriptor for a Matter device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceTypeDescriptor {
    /// Device Type ID.
    pub(crate) id: DeviceTypeId,
    /// Device type name.
    pub(crate) name: &'static str,
    /// Server cluster requirements.
    pub(crate) server_clusters: &'static [DeviceClusterReq],
    /// Client cluster requirements.
    pub(crate) client_clusters: &'static [DeviceClusterReq],
    /// Cross-cluster choice groups.
    pub(crate) choice_groups: &'static [ChoiceGroup],
    /// Included member requirements.
    pub(crate) includes: &'static [DeviceTypeIncludeItem],
}

impl DeviceTypeDescriptor {
    /// Returns the device type ID.
    #[must_use]
    pub const fn id(&self) -> DeviceTypeId {
        self.id
    }

    /// Returns the device type name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

/// Scalar and length bounds for wire values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueBounds {
    /// No additional bounds beyond the wire type.
    None,
    /// Inclusive unsigned integer range.
    Unsigned {
        /// Minimum allowed value (inclusive).
        min: u64,
        /// Maximum allowed value (inclusive).
        max: u64,
    },
    /// Inclusive signed integer range.
    Signed {
        /// Minimum allowed value (inclusive).
        min: i64,
        /// Maximum allowed value (inclusive).
        max: i64,
    },
    /// UTF-8 string byte-length range.
    StringLen {
        /// Minimum byte length (inclusive).
        min: usize,
        /// Maximum byte length (inclusive).
        max: usize,
    },
    /// Octet string byte-length range.
    BytesLen {
        /// Minimum byte length (inclusive).
        min: usize,
        /// Maximum byte length (inclusive).
        max: usize,
    },
}

/// Length and element constraint for `AttributeValue::List`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ListLength {
    /// Minimum number of elements (inclusive).
    pub min: usize,
    /// Maximum number of elements (inclusive).
    pub max: usize,
    /// Constraint applied to each element in the list.
    pub item_constraint: &'static ValueConstraint,
}

/// Expected wire value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueType {
    /// Any wire value (used in unit tests).
    #[cfg(test)]
    Any,
    /// `AttributeValue::Bool`.
    Bool,
    /// `AttributeValue::U8`.
    U8,
    /// `AttributeValue::U16`.
    U16,
    /// `AttributeValue::U32`.
    U32,
    /// `AttributeValue::U64`.
    U64,
    /// `AttributeValue::I8`.
    I8,
    /// `AttributeValue::I16`.
    I16,
    /// `AttributeValue::I32`.
    I32,
    /// `AttributeValue::I64`.
    I64,
    /// `AttributeValue::F32`.
    F32,
    /// `AttributeValue::F64`.
    F64,
    /// `AttributeValue::String`.
    String,
    /// `AttributeValue::Bytes`.
    OctetString,
    /// `AttributeValue::List`.
    List(&'static ListLength),
    /// `AttributeValue::Struct`.
    Struct(&'static StructDescriptor),
    /// Unsigned integer enum with explicit valid discriminants.
    Enum {
        /// Bit width of the enum (`8` or `16`).
        bits: u8,
        /// Sorted or small slice of valid enum discriminants.
        valid_values: &'static [u64],
    },
    /// Unsigned integer bitmap with valid bit mask.
    Bitmap {
        /// Bit width of the bitmap (`8`, `16`, `32`, or `64`).
        bits: u8,
        /// Bitmask of all defined bits.
        valid_mask: u64,
    },
}

/// Complete constraint for a wire [`AttributeValue`](crate::model::AttributeValue).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueConstraint {
    /// Expected wire type.
    pub value_type: ValueType,
    /// Optional scalar or length bounds.
    pub bounds: ValueBounds,
    /// Whether `AttributeValue::Null` is permitted.
    pub nullable: bool,
}

/// RPN operation for evaluating CSA conformance expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    any(
        feature = "v1_0",
        feature = "v1_1",
        feature = "v1_2",
        feature = "v1_3",
        feature = "v1_4"
    ),
    expect(
        dead_code,
        reason = "Older Matter spec versions do not emit every conformance opcode"
    )
)]
pub(crate) enum ConformanceOp {
    /// Push `true` if `(feature_map & mask) != 0`.
    PushFeature(u32),
    /// Push `true` if attribute `id` is present on the cluster.
    PushAttrPresent(u32),
    /// Push numeric value of attribute `id` (or `None` if absent/non-numeric).
    PushAttrValue(u32),
    /// Push a constant integer for `greaterTerm` / `greaterOrEqualTerm`.
    PushConst(i64),
    /// Push a boolean condition literal.
    PushBool(bool),
    /// Logical AND of top two booleans.
    And,
    /// Logical OR of top two booleans.
    Or,
    /// Logical NOT of top boolean.
    Not,
    /// Numeric `lhs > rhs`.
    Greater,
    /// Numeric `lhs >= rhs`.
    GreaterOrEqual,
    /// If top boolean is `true`, yield `Mandatory`, else `None`.
    Mandatory,
    /// If top boolean is `true`, yield `Optional`, else `None`.
    Optional,
    /// If top boolean is `true`, yield `Disallowed`, else `None`.
    Disallow,
    /// If top boolean is `true`, yield `Provisional`, else `None`.
    Provisional,
    /// If top boolean is `true`, yield `Deprecated`, else `None`.
    Deprecate,
    /// Combine two conformance results: if first is `Some`, keep it; otherwise take second.
    Otherwise,
}

/// Evaluated conformance outcome for a member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConformanceResult {
    /// Member is mandatory.
    Mandatory,
    /// Member is optional.
    Optional,
    /// Member is disallowed.
    Disallowed,
    /// Member is provisional.
    Provisional,
    /// Member is deprecated.
    Deprecated,
}

/// Cross-member choice group (`choice` with `min`/`max`/`more`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChoiceGroup {
    /// Minimum number of present members required.
    pub min: u8,
    /// Maximum number of present members allowed (ignored when `more` is `true`).
    pub max: u8,
    /// When `true`, more than `max` members are permitted.
    pub more: bool,
    /// Member IDs (attribute IDs or cluster IDs) belonging to this choice group.
    pub members: &'static [u32],
}

/// Static descriptor for a struct field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FieldDescriptor {
    /// Field ID.
    pub id: FieldId,
    /// Field name.
    pub name: &'static str,
    /// Value constraint.
    pub constraint: ValueConstraint,
    /// Conformance RPN program.
    pub conformance: &'static [ConformanceOp],
}

/// Static descriptor for a Matter struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StructDescriptor {
    /// Struct name.
    pub name: &'static str,
    /// Fields in this struct.
    pub fields: &'static [FieldDescriptor],
}

/// Cluster requirement on a device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeviceClusterReq {
    /// Cluster ID.
    pub cluster_id: ClusterId,
    /// Conformance RPN program.
    pub conformance: &'static [ConformanceOp],
}

/// `DeviceTypeIncludeItem` rule requiring specific attributes, commands, or events on a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeviceTypeIncludeItem {
    /// Target cluster ID.
    pub cluster_id: ClusterId,
    /// `true` for server cluster, `false` for client cluster.
    pub is_server: bool,
    /// Attributes required by this device type.
    pub required_attributes: &'static [AttributeId],
    /// Commands required by this device type (checked when `AcceptedCommandList`/`GeneratedCommandList` is present).
    pub required_commands: &'static [CommandId],
    /// Events required by this device type (checked when `EventList` is present).
    pub required_events: &'static [EventId],
}

static U32_CONSTRAINT: ValueConstraint = ValueConstraint {
    value_type: ValueType::U32,
    bounds: ValueBounds::None,
    nullable: false,
};

static U32_LIST_LENGTH: ListLength = ListLength {
    min: 0,
    max: usize::MAX,
    item_constraint: &U32_CONSTRAINT,
};

/// Static descriptors for Matter Core Specification §7.13 global attributes (`0xFFF8`–`0xFFFD`),
/// sorted by [`AttributeId`].
pub(super) static GLOBAL_ATTRIBUTES: &[AttributeDescriptor] = &[
    AttributeDescriptor {
        id: AttributeId::GENERATED_COMMAND_LIST,
        name: "GeneratedCommandList",
        field_name: "generated_command_list",
        access: AccessFlags::READ,
        quality: QualityFlags::EMPTY,
        read_privilege: Some(AccessPrivilege::View),
        write_privilege: None,
        constraint: ValueConstraint {
            value_type: ValueType::List(&U32_LIST_LENGTH),
            bounds: ValueBounds::None,
            nullable: false,
        },
        conformance: &[ConformanceOp::Mandatory],
    },
    AttributeDescriptor {
        id: AttributeId::ACCEPTED_COMMAND_LIST,
        name: "AcceptedCommandList",
        field_name: "accepted_command_list",
        access: AccessFlags::READ,
        quality: QualityFlags::EMPTY,
        read_privilege: Some(AccessPrivilege::View),
        write_privilege: None,
        constraint: ValueConstraint {
            value_type: ValueType::List(&U32_LIST_LENGTH),
            bounds: ValueBounds::None,
            nullable: false,
        },
        conformance: &[ConformanceOp::Mandatory],
    },
    AttributeDescriptor {
        id: AttributeId::EVENT_LIST,
        name: "EventList",
        field_name: "event_list",
        access: AccessFlags::READ,
        quality: QualityFlags::EMPTY,
        read_privilege: Some(AccessPrivilege::View),
        write_privilege: None,
        constraint: ValueConstraint {
            value_type: ValueType::List(&U32_LIST_LENGTH),
            bounds: ValueBounds::None,
            nullable: false,
        },
        conformance: &[ConformanceOp::Optional],
    },
    AttributeDescriptor {
        id: AttributeId::ATTRIBUTE_LIST,
        name: "AttributeList",
        field_name: "attribute_list",
        access: AccessFlags::READ,
        quality: QualityFlags::EMPTY,
        read_privilege: Some(AccessPrivilege::View),
        write_privilege: None,
        constraint: ValueConstraint {
            value_type: ValueType::List(&U32_LIST_LENGTH),
            bounds: ValueBounds::None,
            nullable: false,
        },
        conformance: &[ConformanceOp::Mandatory],
    },
    AttributeDescriptor {
        id: AttributeId::FEATURE_MAP,
        name: "FeatureMap",
        field_name: "feature_map",
        access: AccessFlags::READ,
        quality: QualityFlags::EMPTY,
        read_privilege: Some(AccessPrivilege::View),
        write_privilege: None,
        constraint: ValueConstraint {
            value_type: ValueType::U32,
            bounds: ValueBounds::None,
            nullable: false,
        },
        conformance: &[ConformanceOp::Mandatory],
    },
    AttributeDescriptor {
        id: AttributeId::CLUSTER_REVISION,
        name: "ClusterRevision",
        field_name: "cluster_revision",
        access: AccessFlags::READ,
        quality: QualityFlags::EMPTY,
        read_privilege: Some(AccessPrivilege::View),
        write_privilege: None,
        constraint: ValueConstraint {
            value_type: ValueType::U16,
            bounds: ValueBounds::Unsigned { min: 1, max: 0xFFFF },
            nullable: false,
        },
        conformance: &[ConformanceOp::Mandatory],
    },
];
