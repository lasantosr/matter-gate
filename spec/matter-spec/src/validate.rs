//! Zero-allocation, non-mutating static schema validation engine.
//!
//! Operates directly over immutable `'static` descriptor tables (`SpecRegistry`)
//! and borrowed [`matter_data_model`] wire types without heap allocation, cloning,
//! or mutating valid payloads.

use alloc::string::String;
use core::{
    fmt,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

use crate::{
    error::{AttributeError, ClusterError, CommandError, EndpointError, EventError, NodeError},
    model::{
        Attribute, AttributeId, AttributeValue, Cluster, ClusterId, Command, CommandDirection, CommandId, DeviceTypeId,
        Endpoint, Event, EventId, EventPriority, FieldId, MatterStatus, Node, SemanticTag, VendorId,
    },
};

/// Maximum number of standard attributes or struct fields tracked via stack bitset.
const MAX_STACK_MEMBERS: usize = 256;
/// Number of `u64` words in the stack bitset (`256 / 64 = 4`).
const BITSET_WORDS: usize = 4;
/// Maximum RPN evaluation stack depth.
const MAX_RPN_STACK: usize = 32;

/// Validation strictness mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationMode {
    /// Validate only the members present in a partial/delta payload.
    Partial,
    /// Validate a complete cluster or endpoint snapshot, including mandatory members
    /// and global lists.
    FullSnapshot {
        /// When `true`, provisional (`P`) members are treated as disallowed.
        enforce_provisional: bool,
    },
}

impl Default for ValidationMode {
    fn default() -> Self {
        Self::FullSnapshot {
            enforce_provisional: false,
        }
    }
}

/// Access control flags for cluster attributes, commands, and events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct AccessFlags(pub(crate) u8);

impl AccessFlags {
    /// Empty set of access flags (`0`).
    pub const EMPTY: Self = Self(0);
    /// Fabric-scoped member.
    pub const FABRIC_SCOPED: Self = Self(1 << 2);
    /// Fabric-sensitive member.
    pub const FABRIC_SENSITIVE: Self = Self(1 << 3);
    /// Readable attribute.
    pub const READ: Self = Self(1 << 0);
    /// Scene-capable attribute.
    pub const SCENE: Self = Self(1 << 5);
    /// Requires timed interaction for writes/invocations.
    pub const TIMED: Self = Self(1 << 4);
    /// Writable attribute.
    pub const WRITE: Self = Self(1 << 1);

    /// Returns an empty set of access flags.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns the raw bitflags representation.
    #[inline]
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Constructs `AccessFlags` from raw bits, truncating any unknown bits.
    #[inline]
    #[must_use]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        let valid = Self::READ.0
            | Self::WRITE.0
            | Self::FABRIC_SCOPED.0
            | Self::FABRIC_SENSITIVE.0
            | Self::TIMED.0
            | Self::SCENE.0;
        Self(bits & valid)
    }

    /// Returns `true` if no flags are set (`0`).
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all bits in `other` are set in `self`.
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Inserts all flags in `other` into `self`.
    #[inline]
    pub const fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Removes all flags in `other` from `self`.
    #[inline]
    pub const fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    /// Returns the bitwise union (`self | other`) of two flag sets.
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns the bitwise intersection (`self & other`) of two flag sets.
    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl fmt::Display for AccessFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X}", self.0)
    }
}

impl BitOr for AccessFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for AccessFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for AccessFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for AccessFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitXor for AccessFlags {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for AccessFlags {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for AccessFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Self::from_bits_truncate(!self.0)
    }
}

impl From<AccessFlags> for u8 {
    #[inline]
    fn from(flags: AccessFlags) -> Self {
        flags.0
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
        /// Sorted or small slice of valid enum discriminants.
        valid_values: &'static [u64],
    },
    /// Unsigned integer bitmap with valid bit mask.
    Bitmap {
        /// Bitmask of all defined bits.
        valid_mask: u64,
    },
}

/// Complete constraint for a wire [`AttributeValue`].
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
}

/// Static descriptor for a Matter cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClusterDescriptor {
    /// Cluster ID.
    pub(crate) id: ClusterId,
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

/// Read-only static schema registry for zero-allocation validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecRegistry {
    /// Whether `EventList` (`0xFFFA`) is defined in this release's global attributes and expected in `FullSnapshot`
    /// mode.
    pub(crate) expects_event_list: bool,
    /// Sorted slice of cluster descriptors (ascending by `ClusterId`).
    pub(crate) clusters: &'static [ClusterDescriptor],
    /// Sorted slice of device type descriptors (ascending by `DeviceTypeId`).
    pub(crate) device_types: &'static [DeviceTypeDescriptor],
}

/// Zero-cost, builder-configured validator bound to a [`SpecRegistry`].
///
/// Defaults to [`ValidationMode::FullSnapshot`] with `enforce_provisional: false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryValidator<'a> {
    registry: &'a SpecRegistry,
    mode: ValidationMode,
}

impl<'a> RegistryValidator<'a> {
    /// Creates a [`RegistryValidator`] in [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`).
    #[must_use]
    pub const fn new(registry: &'a SpecRegistry) -> Self {
        Self::full(registry)
    }

    /// Creates a [`RegistryValidator`] in [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`).
    #[must_use]
    pub const fn full(registry: &'a SpecRegistry) -> Self {
        Self {
            registry,
            mode: ValidationMode::FullSnapshot {
                enforce_provisional: false,
            },
        }
    }

    /// Creates a [`RegistryValidator`] in [`ValidationMode::Partial`].
    #[must_use]
    pub const fn partial(registry: &'a SpecRegistry) -> Self {
        Self {
            registry,
            mode: ValidationMode::Partial,
        }
    }

    /// Creates a [`RegistryValidator`] with an explicit [`ValidationMode`].
    #[must_use]
    pub const fn with_mode(self, mode: ValidationMode) -> Self {
        Self {
            registry: self.registry,
            mode,
        }
    }

    /// Switches the validator to [`ValidationMode::Partial`].
    #[must_use]
    pub const fn partial_mode(self) -> Self {
        self.with_mode(ValidationMode::Partial)
    }

    /// Switches the validator to [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`).
    #[must_use]
    pub const fn full_mode(self) -> Self {
        self.with_mode(ValidationMode::FullSnapshot {
            enforce_provisional: false,
        })
    }

    /// Sets `enforce_provisional` when in [`ValidationMode::FullSnapshot`].
    ///
    /// Has no effect when in [`ValidationMode::Partial`].
    #[must_use]
    pub const fn enforce_provisional(self, enforce: bool) -> Self {
        match self.mode {
            ValidationMode::FullSnapshot { .. } => self.with_mode(ValidationMode::FullSnapshot {
                enforce_provisional: enforce,
            }),
            ValidationMode::Partial => self,
        }
    }

    /// Returns the active [`ValidationMode`].
    #[must_use]
    pub const fn mode(&self) -> ValidationMode {
        self.mode
    }

    /// Validates a borrowed [`Attribute`] on `cluster_id` without allocating or mutating.
    pub fn attribute(&self, cluster_id: ClusterId, attr: &Attribute) -> Result<(), AttributeError> {
        self.registry.validate_attribute_ref(cluster_id, attr)
    }

    /// Validates a borrowed [`Command`] without allocating or mutating.
    pub fn command(&self, cmd: &Command) -> Result<(), CommandError> {
        self.registry.validate_command_ref(cmd)
    }

    /// Validates a borrowed [`Event`] without allocating or mutating.
    pub fn event(&self, event: &Event) -> Result<(), EventError> {
        self.registry.validate_event_ref(event)
    }

    /// Validates a borrowed [`Cluster`] using the configured [`ValidationMode`].
    pub fn cluster(&self, cluster: &Cluster) -> Result<(), ClusterError> {
        self.registry.validate_cluster_ref_with_mode(cluster, self.mode)
    }

    /// Validates a borrowed [`Endpoint`] using the configured [`ValidationMode`].
    pub fn endpoint(&self, endpoint: &Endpoint) -> Result<(), EndpointError> {
        self.registry.validate_endpoint_ref_with_mode(endpoint, self.mode)
    }

    /// Validates a borrowed [`Node`] using the configured [`ValidationMode`].
    pub fn node(&self, node: &Node) -> Result<(), NodeError> {
        self.registry.validate_node_ref_with_mode(node, self.mode)
    }
}

impl SpecRegistry {
    /// Returns all cluster descriptors in this registry, sorted by [`ClusterId`].
    #[must_use]
    pub const fn clusters(&self) -> &'static [ClusterDescriptor] {
        self.clusters
    }

    /// Returns all device type descriptors in this registry, sorted by [`DeviceTypeId`].
    #[must_use]
    pub const fn device_types(&self) -> &'static [DeviceTypeDescriptor] {
        self.device_types
    }

    /// Returns a [`RegistryValidator`] configured for [`ValidationMode::FullSnapshot`] (`enforce_provisional: false`).
    #[must_use]
    pub const fn validator(&self) -> RegistryValidator<'_> {
        RegistryValidator::full(self)
    }

    /// Returns a [`RegistryValidator`] configured for [`ValidationMode::Partial`].
    #[must_use]
    pub const fn partial_validator(&self) -> RegistryValidator<'_> {
        RegistryValidator::partial(self)
    }

    /// Looks up a cluster descriptor by [`ClusterId`] in `O(log N)` time.
    #[must_use]
    pub fn find_cluster(&self, id: ClusterId) -> Option<&'static ClusterDescriptor> {
        let idx = self.clusters.binary_search_by_key(&id.raw(), |c| c.id.raw()).ok()?;
        self.clusters.get(idx)
    }

    /// Looks up a device type descriptor by [`DeviceTypeId`] in `O(log N)` time.
    #[must_use]
    pub fn find_device_type(&self, id: DeviceTypeId) -> Option<&'static DeviceTypeDescriptor> {
        let idx = self.device_types.binary_search_by_key(&id.raw(), |d| d.id.raw()).ok()?;
        self.device_types.get(idx)
    }

    fn validate_attribute_ref(&self, cluster_id: ClusterId, attr: &Attribute) -> Result<(), AttributeError> {
        let cluster = self
            .find_cluster(cluster_id)
            .ok_or(AttributeError::UnsupportedCluster(cluster_id))?;
        let Some(desc) = cluster.attributes.iter().find(|a| a.id == attr.id) else {
            return Err(AttributeError::UnsupportedAttribute(attr.id));
        };
        validate_value(&attr.value, &desc.constraint).map_err(|(field_id, status)| AttributeError::InvalidValue {
            id: attr.id,
            field_id,
            status,
        })
    }

    fn validate_command_ref(&self, cmd: &Command) -> Result<(), CommandError> {
        let cluster = self
            .find_cluster(cmd.cluster_id)
            .ok_or(CommandError::UnsupportedCluster(cmd.cluster_id))?;
        let Some(desc) = cluster.commands.iter().find(|c| c.id == cmd.command_id) else {
            return Err(CommandError::UnsupportedCommand(cmd.command_id));
        };
        if cmd.direction != desc.direction {
            return Err(CommandError::DirectionMismatch {
                expected: desc.direction,
                actual: cmd.direction,
            });
        }
        validate_struct_payload(&cmd.payload, desc.fields, true, 0, None).map_err(|(field_id, status)| {
            CommandError::InvalidPayload {
                id: cmd.command_id,
                field_id,
                status,
            }
        })
    }

    fn validate_event_ref(&self, event: &Event) -> Result<(), EventError> {
        let cluster = self
            .find_cluster(event.cluster_id)
            .ok_or(EventError::UnsupportedCluster(event.cluster_id))?;
        let Some(desc) = cluster.events.iter().find(|e| e.id == event.event_id) else {
            return Err(EventError::UnsupportedEvent(event.event_id));
        };
        if event.priority != desc.priority {
            return Err(EventError::PriorityMismatch {
                expected: desc.priority,
                actual: event.priority,
            });
        }
        validate_struct_payload(&event.payload, desc.fields, false, 0, None).map_err(|(field_id, status)| {
            EventError::InvalidPayload {
                id: event.event_id,
                field_id,
                status,
            }
        })
    }

    fn validate_cluster_ref_with_mode(&self, cluster: &Cluster, mode: ValidationMode) -> Result<(), ClusterError> {
        let desc = self
            .find_cluster(cluster.id)
            .ok_or(ClusterError::UnsupportedCluster(cluster.id))?;

        let feature_map = extract_feature_map(cluster);
        let mut seen = StackBitset::new();
        let attrs = &cluster.attributes;

        // 1. Check duplicates and validate each present attribute.
        for (idx, attr) in attrs.iter().enumerate() {
            if let Some((desc_idx, attr_desc)) = desc.attributes.iter().enumerate().find(|(_, a)| a.id == attr.id) {
                check_attr_duplicate(&mut seen, attrs, idx, desc_idx, attr.id)?;

                let conf = eval_conformance(attr_desc.conformance, feature_map, Some(cluster));
                if is_disallowed(conf, mode) {
                    return Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
                        attr.id,
                    )));
                }
                validate_value_with_ctx(&attr.value, &attr_desc.constraint, feature_map, Some(cluster)).map_err(
                    |(field_id, status)| {
                        ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                            id: attr.id,
                            field_id,
                            status,
                        })
                    },
                )?;
            } else if is_global_attribute(attr.id) {
                if attrs
                    .get(..idx)
                    .is_some_and(|prevs| prevs.iter().any(|prev| prev.id == attr.id))
                {
                    return Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                        id: attr.id,
                        field_id: None,
                        status: MatterStatus::ConstraintError,
                    }));
                }
            } else {
                return Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
                    attr.id,
                )));
            }
        }

        // 2. FullSnapshot checks: mandatory attributes, EventList (v1.1..=v1.3), and ChoiceGroups.
        if matches!(mode, ValidationMode::FullSnapshot { .. }) {
            for attr_desc in desc.attributes {
                let conf = eval_conformance(attr_desc.conformance, feature_map, Some(cluster));
                if conf == ConformanceResult::Mandatory && !attrs.iter().any(|a| a.id == attr_desc.id) {
                    return Err(ClusterError::MissingMandatoryAttribute(attr_desc.id));
                }
            }

            if self.expects_event_list
                && !desc.events.is_empty()
                && !attrs.iter().any(|a| a.id == AttributeId::EVENT_LIST)
            {
                return Err(ClusterError::MissingMandatoryAttribute(AttributeId::EVENT_LIST));
            }

            for group in desc.choice_groups {
                if !check_choice_group(group, |id| attrs.iter().any(|a| a.id.raw() == id)) {
                    let first_id = group.members.first().copied().unwrap_or(0);
                    return Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                        id: AttributeId::new(first_id),
                        field_id: None,
                        status: MatterStatus::ConstraintError,
                    }));
                }
            }
        }

        Ok(())
    }

    fn validate_endpoint_ref_with_mode(&self, endpoint: &Endpoint, mode: ValidationMode) -> Result<(), EndpointError> {
        if endpoint.device_types.is_empty() {
            return Err(EndpointError::EmptyDeviceTypes);
        }

        // Duplicate DeviceTypeId check (O(N^2) bounded, zero-alloc).
        for (i, dt) in endpoint.device_types.iter().enumerate() {
            if endpoint
                .device_types
                .get(..i)
                .is_some_and(|prevs| prevs.iter().any(|prev| prev.id == dt.id))
            {
                return Err(EndpointError::DuplicateDeviceType(dt.id));
            }
        }

        // Duplicate server cluster IDs.
        for (i, c) in endpoint.server_clusters.iter().enumerate() {
            if endpoint
                .server_clusters
                .get(..i)
                .is_some_and(|prevs| prevs.iter().any(|prev| prev.id == c.id))
            {
                return Err(EndpointError::DuplicateCluster(c.id));
            }
        }

        // Duplicate client cluster IDs.
        for (i, cid) in endpoint.client_clusters.iter().enumerate() {
            if endpoint
                .client_clusters
                .get(..i)
                .is_some_and(|prevs| prevs.contains(cid))
            {
                return Err(EndpointError::DuplicateCluster(*cid));
            }
        }

        // Validate each device type's cluster requirements, choice groups, and includes.
        for dt_entry in &endpoint.device_types {
            let dt_desc = self
                .find_device_type(dt_entry.id)
                .ok_or(EndpointError::UnsupportedDeviceType(dt_entry.id))?;
            validate_device_type_reqs(endpoint, dt_desc, mode)?;
            for inc in dt_desc.includes {
                validate_include_item(endpoint, inc)?;
            }
        }

        // Cross-check Descriptor (0x001D) if present.
        if let Some(descriptor) = endpoint.server_clusters.iter().find(|c| c.id == ClusterId::DESCRIPTOR) {
            validate_descriptor_consistency(endpoint, descriptor)?;
        }

        // Validate each registered server cluster; extra/MEI clusters not in registry are permitted.
        for cluster in &endpoint.server_clusters {
            if self.find_cluster(cluster.id).is_some() {
                self.validate_cluster_ref_with_mode(cluster, mode)
                    .map_err(|source| EndpointError::InvalidCluster { id: cluster.id, source })?;
            }
        }

        Ok(())
    }

    fn validate_node_ref_with_mode(&self, node: &Node, mode: ValidationMode) -> Result<(), NodeError> {
        if node.root_endpoint().is_none() {
            return Err(NodeError::MissingRootEndpoint);
        }
        for (i, ep_a) in node.endpoints.iter().enumerate() {
            for ep_b in node.endpoints.iter().skip(i.saturating_add(1)) {
                if ep_a.id == ep_b.id {
                    return Err(NodeError::DuplicateEndpoint(ep_a.id));
                }
            }
        }
        for ep in &node.endpoints {
            self.validate_endpoint_ref_with_mode(ep, mode)
                .map_err(|source| NodeError::InvalidEndpoint { id: ep.id, source })?;
        }
        Ok(())
    }
}

/// Validates a wire [`AttributeValue`] against a [`ValueConstraint`], returning the innermost
/// failing [`FieldId`] and normative [`MatterStatus`].
pub(crate) fn validate_value(
    value: &AttributeValue,
    constraint: &ValueConstraint,
) -> Result<(), (Option<FieldId>, MatterStatus)> {
    validate_value_with_ctx(value, constraint, 0, None)
}

fn validate_value_with_ctx(
    value: &AttributeValue,
    constraint: &ValueConstraint,
    feature_map: u32,
    cluster: Option<&Cluster>,
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
        ValueType::U8 => match value {
            AttributeValue::U8(v) => check_unsigned(u64::from(*v), constraint.bounds),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::U16 => match value {
            AttributeValue::U16(v) => check_unsigned(u64::from(*v), constraint.bounds),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::U32 => match value {
            AttributeValue::U32(v) => check_unsigned(u64::from(*v), constraint.bounds),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::U64 => match value {
            AttributeValue::U64(v) => check_unsigned(*v, constraint.bounds),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::I8 => match value {
            AttributeValue::I8(v) => check_signed(i64::from(*v), constraint.bounds),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::I16 => match value {
            AttributeValue::I16(v) => check_signed(i64::from(*v), constraint.bounds),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::I32 => match value {
            AttributeValue::I32(v) => check_signed(i64::from(*v), constraint.bounds),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::I64 => match value {
            AttributeValue::I64(v) => check_signed(*v, constraint.bounds),
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
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
                    validate_value_with_ctx(item, list_len.item_constraint, feature_map, cluster)?;
                }
                Ok(())
            }
            _ => Err((None, MatterStatus::InvalidDataType)),
        },
        ValueType::Struct(struct_desc) => {
            validate_struct_payload(value, struct_desc.fields, false, feature_map, cluster)
        }
        ValueType::Enum { valid_values } => {
            let raw = extract_unsigned(value).ok_or((None, MatterStatus::InvalidDataType))?;
            if valid_values.contains(&raw) {
                Ok(())
            } else {
                Err((None, MatterStatus::ConstraintError))
            }
        }
        ValueType::Bitmap { valid_mask } => {
            let raw = extract_unsigned(value).ok_or((None, MatterStatus::InvalidDataType))?;
            if (raw & !valid_mask) == 0 {
                Ok(())
            } else {
                Err((None, MatterStatus::ConstraintError))
            }
        }
    }
}

fn validate_struct_payload(
    value: &AttributeValue,
    fields: &[FieldDescriptor],
    is_command: bool,
    feature_map: u32,
    cluster: Option<&Cluster>,
) -> Result<(), (Option<FieldId>, MatterStatus)> {
    let AttributeValue::Struct(entries) = value else {
        return Err((None, MatterStatus::InvalidDataType));
    };

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

        if let Err((inner_fid, status)) = validate_value_with_ctx(fval, &field_desc.constraint, feature_map, cluster) {
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

fn extract_unsigned(value: &AttributeValue) -> Option<u64> {
    match value {
        AttributeValue::U8(v) => Some(u64::from(*v)),
        AttributeValue::U16(v) => Some(u64::from(*v)),
        AttributeValue::U32(v) => Some(u64::from(*v)),
        AttributeValue::U64(v) => Some(*v),
        _ => None,
    }
}

fn extract_signed(value: &AttributeValue) -> Option<i64> {
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

#[derive(Debug, Clone, Copy)]
enum StackVal {
    Bool(bool),
    Num(Option<i64>),
    Conf(Option<ConformanceResult>),
}

/// Evaluates a [`ConformanceOp`] RPN program using a fixed-size stack.
#[must_use]
pub(crate) fn eval_conformance(
    ops: &[ConformanceOp],
    feature_map: u32,
    cluster: Option<&Cluster>,
) -> ConformanceResult {
    if ops.is_empty() {
        return ConformanceResult::Optional;
    }

    let mut stack = [StackVal::Bool(false); MAX_RPN_STACK];
    let mut sp: usize = 0;

    for &op in ops {
        match op {
            ConformanceOp::PushFeature(mask) => {
                push_stack(&mut stack, &mut sp, StackVal::Bool((feature_map & mask) != 0));
            }
            ConformanceOp::PushAttrPresent(id) => {
                let present = cluster.is_some_and(|c| c.attribute(AttributeId::new(id)).is_some());
                push_stack(&mut stack, &mut sp, StackVal::Bool(present));
            }
            ConformanceOp::PushAttrValue(id) => {
                let num = cluster
                    .and_then(|c| c.attribute(AttributeId::new(id)))
                    .and_then(extract_signed);
                push_stack(&mut stack, &mut sp, StackVal::Num(num));
            }
            ConformanceOp::PushConst(val) => {
                push_stack(&mut stack, &mut sp, StackVal::Num(Some(val)));
            }
            ConformanceOp::PushBool(b) => {
                push_stack(&mut stack, &mut sp, StackVal::Bool(b));
            }
            ConformanceOp::And => {
                if sp >= 2 {
                    let b = pop_bool(&stack, &mut sp);
                    let a = pop_bool(&stack, &mut sp);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(a && b));
                }
            }
            ConformanceOp::Or => {
                if sp >= 2 {
                    let b = pop_bool(&stack, &mut sp);
                    let a = pop_bool(&stack, &mut sp);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(a || b));
                }
            }
            ConformanceOp::Not => {
                if sp >= 1 {
                    let a = pop_bool(&stack, &mut sp);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(!a));
                }
            }
            ConformanceOp::Greater => {
                if sp >= 2 {
                    let rhs = pop_num(&stack, &mut sp);
                    let lhs = pop_num(&stack, &mut sp);
                    let res = matches!((lhs, rhs), (Some(l), Some(r)) if l > r);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(res));
                }
            }
            ConformanceOp::GreaterOrEqual => {
                if sp >= 2 {
                    let rhs = pop_num(&stack, &mut sp);
                    let lhs = pop_num(&stack, &mut sp);
                    let res = matches!((lhs, rhs), (Some(l), Some(r)) if l >= r);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(res));
                }
            }
            ConformanceOp::Mandatory => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Mandatory)),
                );
            }
            ConformanceOp::Optional => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Optional)),
                );
            }
            ConformanceOp::Disallow => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Disallowed)),
                );
            }
            ConformanceOp::Provisional => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Provisional)),
                );
            }
            ConformanceOp::Deprecate => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Deprecated)),
                );
            }
            ConformanceOp::Otherwise => {
                if sp >= 2 {
                    let second = pop_conf(&stack, &mut sp);
                    let first = pop_conf(&stack, &mut sp);
                    push_stack(&mut stack, &mut sp, StackVal::Conf(first.or(second)));
                }
            }
        }
    }

    if let Some(top) = sp.checked_sub(1).and_then(|idx| stack.get(idx)) {
        match top {
            StackVal::Conf(Some(c)) => *c,
            _ => ConformanceResult::Optional,
        }
    } else {
        ConformanceResult::Optional
    }
}

fn push_stack(stack: &mut [StackVal; MAX_RPN_STACK], sp: &mut usize, val: StackVal) {
    if let Some(slot) = stack.get_mut(*sp) {
        *slot = val;
        *sp = sp.saturating_add(1);
    }
}

fn pop_bool(stack: &[StackVal; MAX_RPN_STACK], sp: &mut usize) -> bool {
    *sp = sp.saturating_sub(1);
    match stack.get(*sp) {
        Some(StackVal::Bool(b)) => *b,
        _ => false,
    }
}

fn pop_num(stack: &[StackVal; MAX_RPN_STACK], sp: &mut usize) -> Option<i64> {
    *sp = sp.saturating_sub(1);
    match stack.get(*sp) {
        Some(StackVal::Num(n)) => *n,
        _ => None,
    }
}

fn pop_conf(stack: &[StackVal; MAX_RPN_STACK], sp: &mut usize) -> Option<ConformanceResult> {
    *sp = sp.saturating_sub(1);
    match stack.get(*sp) {
        Some(StackVal::Conf(c)) => *c,
        _ => None,
    }
}

fn pop_bool_or_true(stack: &[StackVal; MAX_RPN_STACK], sp: &mut usize) -> bool {
    if let Some(idx) = sp.checked_sub(1) {
        if let Some(StackVal::Bool(b)) = stack.get(idx).copied() {
            *sp = idx;
            return b;
        }
    }
    true
}

const fn is_disallowed(conf: ConformanceResult, mode: ValidationMode) -> bool {
    match conf {
        ConformanceResult::Disallowed => true,
        ConformanceResult::Provisional => matches!(
            mode,
            ValidationMode::FullSnapshot {
                enforce_provisional: true
            }
        ),
        _ => false,
    }
}

fn check_choice_group(group: &ChoiceGroup, is_present: impl Fn(u32) -> bool) -> bool {
    let mut count: u8 = 0;
    for &member_id in group.members {
        if is_present(member_id) {
            count = count.saturating_add(1);
        }
    }
    count >= group.min && (group.more || count <= group.max)
}

fn extract_feature_map(cluster: &Cluster) -> u32 {
    match cluster.attribute(AttributeId::FEATURE_MAP) {
        Some(AttributeValue::U32(v)) => *v,
        _ => 0,
    }
}

const fn is_global_attribute(id: AttributeId) -> bool {
    matches!(id.raw(), 0xFFF8..=0xFFFD)
}

fn list_contains_u32(val: &AttributeValue, target: u32) -> bool {
    let AttributeValue::List(items) = val else {
        return false;
    };
    items
        .iter()
        .any(|item| extract_unsigned(item) == Some(u64::from(target)))
}

fn descriptor_attr_error(attr_id: u32) -> EndpointError {
    EndpointError::InvalidCluster {
        id: ClusterId::DESCRIPTOR,
        source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
            id: AttributeId::new(attr_id),
            field_id: None,
            status: MatterStatus::ConstraintError,
        }),
    }
}

fn decode_device_type_entry_ref(val: &AttributeValue) -> Option<(u32, u16)> {
    let AttributeValue::Struct(fields) = val else {
        return None;
    };
    let mut id = None;
    let mut rev = None;
    for (fid, fval) in fields {
        match fid.raw() {
            0 => id = extract_unsigned(fval).and_then(|v| u32::try_from(v).ok()),
            1 => rev = extract_unsigned(fval).and_then(|v| u16::try_from(v).ok()),
            _ => {}
        }
    }
    Some((id?, rev?))
}

fn semantic_tag_matches_ref(val: &AttributeValue, tag: &SemanticTag) -> bool {
    let AttributeValue::Struct(fields) = val else {
        return false;
    };
    let mut mfg_code: Option<Option<u16>> = None;
    let mut namespace_id: Option<u8> = None;
    let mut tag_val: Option<u8> = None;
    let mut label: Option<Option<&str>> = None;
    for (fid, fval) in fields {
        match fid.raw() {
            0 => match fval {
                AttributeValue::Null => mfg_code = Some(None),
                other => {
                    let Some(v) = extract_unsigned(other).and_then(|v| u16::try_from(v).ok()) else {
                        return false;
                    };
                    mfg_code = Some(Some(v));
                }
            },
            1 => namespace_id = extract_unsigned(fval).and_then(|v| u8::try_from(v).ok()),
            2 => tag_val = extract_unsigned(fval).and_then(|v| u8::try_from(v).ok()),
            3 => match fval {
                AttributeValue::Null => label = Some(None),
                AttributeValue::String(s) => label = Some(Some(s.as_str())),
                _ => return false,
            },
            _ => {}
        }
    }
    mfg_code.unwrap_or(None) == tag.mfg_code.into_option().map(VendorId::raw)
        && namespace_id == Some(tag.namespace_id)
        && tag_val == Some(tag.tag)
        && label.unwrap_or(None)
            == tag
                .label
                .as_ref()
                .into_option()
                .and_then(|n| n.as_ref().into_option())
                .map(String::as_str)
}

fn validate_descriptor_consistency(endpoint: &Endpoint, descriptor: &Cluster) -> Result<(), EndpointError> {
    if let Some(val) = descriptor.attribute(AttributeId::new(0x0000)) {
        let AttributeValue::List(dt_list) = val else {
            return Err(descriptor_attr_error(0x0000));
        };
        if dt_list.len() != endpoint.device_types.len() {
            return Err(descriptor_attr_error(0x0000));
        }
        for dt in &endpoint.device_types {
            if !dt_list
                .iter()
                .any(|item| decode_device_type_entry_ref(item) == Some((dt.id.raw(), dt.revision)))
            {
                return Err(descriptor_attr_error(0x0000));
            }
        }
    }

    if let Some(val) = descriptor.attribute(AttributeId::new(0x0001)) {
        let AttributeValue::List(srv_list) = val else {
            return Err(descriptor_attr_error(0x0001));
        };
        if srv_list.len() != endpoint.server_clusters.len() {
            return Err(descriptor_attr_error(0x0001));
        }
        for srv in &endpoint.server_clusters {
            if !srv_list
                .iter()
                .any(|item| extract_unsigned(item) == Some(u64::from(srv.id.raw())))
            {
                return Err(descriptor_attr_error(0x0001));
            }
        }
    }

    if let Some(val) = descriptor.attribute(AttributeId::new(0x0002)) {
        let AttributeValue::List(cli_list) = val else {
            return Err(descriptor_attr_error(0x0002));
        };
        if cli_list.len() != endpoint.client_clusters.len() {
            return Err(descriptor_attr_error(0x0002));
        }
        for cli in &endpoint.client_clusters {
            if !cli_list
                .iter()
                .any(|item| extract_unsigned(item) == Some(u64::from(cli.raw())))
            {
                return Err(descriptor_attr_error(0x0002));
            }
        }
    }

    if let Some(val) = descriptor.attribute(AttributeId::new(0x0003)) {
        let AttributeValue::List(parts_list) = val else {
            return Err(descriptor_attr_error(0x0003));
        };
        if parts_list.len() != endpoint.parts.len() {
            return Err(descriptor_attr_error(0x0003));
        }
        for part in &endpoint.parts {
            if !parts_list
                .iter()
                .any(|item| extract_unsigned(item) == Some(u64::from(part.raw())))
            {
                return Err(descriptor_attr_error(0x0003));
            }
        }
    } else if !endpoint.parts.is_empty() {
        return Err(descriptor_attr_error(0x0003));
    } else {
        // Empty parts list may be omitted.
    }

    if let Some(val) = descriptor.attribute(AttributeId::new(0x0004)) {
        let AttributeValue::List(tag_list) = val else {
            return Err(descriptor_attr_error(0x0004));
        };
        if tag_list.len() != endpoint.tags.len() {
            return Err(descriptor_attr_error(0x0004));
        }
        for tag in &endpoint.tags {
            if !tag_list.iter().any(|item| semantic_tag_matches_ref(item, tag)) {
                return Err(descriptor_attr_error(0x0004));
            }
        }
    } else if !endpoint.tags.is_empty() {
        return Err(descriptor_attr_error(0x0004));
    } else {
        // Empty tag list may be omitted.
    }

    Ok(())
}

struct StackBitset {
    words: [u64; BITSET_WORDS],
}

impl StackBitset {
    const fn new() -> Self {
        Self {
            words: [0; BITSET_WORDS],
        }
    }

    fn insert(&mut self, idx: usize) -> bool {
        let word = idx >> 6;
        let bit = 1_u64 << (idx & 63);
        let Some(slot) = self.words.get_mut(word) else {
            return false;
        };
        let was_set = (*slot & bit) != 0;
        *slot |= bit;
        !was_set
    }
}

fn check_attr_duplicate(
    seen: &mut StackBitset,
    attrs: &[Attribute],
    idx: usize,
    desc_idx: usize,
    id: AttributeId,
) -> Result<(), ClusterError> {
    let is_dup = if desc_idx < MAX_STACK_MEMBERS {
        !seen.insert(desc_idx)
    } else {
        attrs
            .get(..idx)
            .is_some_and(|prevs| prevs.iter().any(|prev| prev.id == id))
    };
    if is_dup {
        return Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
            id,
            field_id: None,
            status: MatterStatus::ConstraintError,
        }));
    }
    Ok(())
}

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

fn validate_device_type_reqs(
    endpoint: &Endpoint,
    dt_desc: &DeviceTypeDescriptor,
    mode: ValidationMode,
) -> Result<(), EndpointError> {
    if !matches!(mode, ValidationMode::FullSnapshot { .. }) {
        return Ok(());
    }

    for req in dt_desc.server_clusters {
        let conf = eval_conformance(req.conformance, 0, None);
        let present = endpoint.server_clusters.iter().any(|c| c.id == req.cluster_id);
        if conf == ConformanceResult::Mandatory && !present {
            return Err(EndpointError::MissingMandatoryServerCluster(req.cluster_id));
        }
        if is_disallowed(conf, mode) && present {
            return Err(EndpointError::DisallowedCluster(req.cluster_id));
        }
    }

    for req in dt_desc.client_clusters {
        let conf = eval_conformance(req.conformance, 0, None);
        let present = endpoint.client_clusters.contains(&req.cluster_id);
        if conf == ConformanceResult::Mandatory && !present {
            return Err(EndpointError::MissingMandatoryClientCluster(req.cluster_id));
        }
        if is_disallowed(conf, mode) && present {
            return Err(EndpointError::DisallowedCluster(req.cluster_id));
        }
    }

    for group in dt_desc.choice_groups {
        if !check_choice_group(group, |id| {
            endpoint.server_clusters.iter().any(|c| c.id.raw() == id)
                || endpoint.client_clusters.iter().any(|c| c.raw() == id)
        }) {
            let first_id = group.members.first().copied().unwrap_or(0);
            return Err(EndpointError::MissingMandatoryServerCluster(ClusterId::new(first_id)));
        }
    }

    Ok(())
}

fn validate_include_item(endpoint: &Endpoint, inc: &DeviceTypeIncludeItem) -> Result<(), EndpointError> {
    if !inc.is_server {
        return Ok(());
    }
    let Some(cluster) = endpoint.server_clusters.iter().find(|c| c.id == inc.cluster_id) else {
        return Ok(());
    };

    for &req_attr in inc.required_attributes {
        if !cluster.attributes.iter().any(|a| a.id == req_attr) {
            return Err(EndpointError::InvalidCluster {
                id: cluster.id,
                source: ClusterError::MissingMandatoryAttribute(req_attr),
            });
        }
    }

    if !inc.required_commands.is_empty() {
        if let Some(accepted) = cluster.attribute(AttributeId::ACCEPTED_COMMAND_LIST) {
            for &req_cmd in inc.required_commands {
                if !list_contains_u32(accepted, req_cmd.raw()) {
                    return Err(EndpointError::InvalidCluster {
                        id: cluster.id,
                        source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                            id: AttributeId::ACCEPTED_COMMAND_LIST,
                            field_id: None,
                            status: MatterStatus::ConstraintError,
                        }),
                    });
                }
            }
        }
    }

    if !inc.required_events.is_empty() {
        if let Some(events) = cluster.attribute(AttributeId::EVENT_LIST) {
            for &req_ev in inc.required_events {
                if !list_contains_u32(events, req_ev.raw()) {
                    return Err(EndpointError::InvalidCluster {
                        id: cluster.id,
                        source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                            id: AttributeId::EVENT_LIST,
                            field_id: None,
                            status: MatterStatus::ConstraintError,
                        }),
                    });
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec};

    use super::*;
    use crate::model::{DataVersion, DeviceTypeEntry, EndpointId, EventPriority, NodeId, Optional};

    static NESTED_STRUCT: StructDescriptor = StructDescriptor {
        name: "NestedStruct",
        fields: &[FieldDescriptor {
            id: FieldId::new(1),
            name: "Count",
            constraint: ValueConstraint {
                value_type: ValueType::U8,
                bounds: ValueBounds::Unsigned { min: 1, max: 10 },
                nullable: false,
            },
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
        }],
    };

    static OUTER_STRUCT: StructDescriptor = StructDescriptor {
        name: "OuterStruct",
        fields: &[
            FieldDescriptor {
                id: FieldId::new(0),
                name: "Label",
                constraint: ValueConstraint {
                    value_type: ValueType::String,
                    bounds: ValueBounds::StringLen { min: 2, max: 8 },
                    nullable: false,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
            },
            FieldDescriptor {
                id: FieldId::new(1),
                name: "Nested",
                constraint: ValueConstraint {
                    value_type: ValueType::Struct(&NESTED_STRUCT),
                    bounds: ValueBounds::None,
                    nullable: true,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Optional],
            },
        ],
    };

    static ITEM_U8_CONSTRAINT: ValueConstraint = ValueConstraint {
        value_type: ValueType::U8,
        bounds: ValueBounds::Unsigned { min: 1, max: 5 },
        nullable: false,
    };

    static LIST_CONSTRAINT: ListLength = ListLength {
        min: 1,
        max: 3,
        item_constraint: &ITEM_U8_CONSTRAINT,
    };

    static MOCK_CLUSTERS: &[ClusterDescriptor] = &[ClusterDescriptor {
        id: ClusterId::new(0x0006),
        name: "OnOff",
        attributes: &[
            AttributeDescriptor {
                id: AttributeId::new(0x0000),
                name: "OnOff",
                field_name: "on_off",
                access: AccessFlags::READ,
                constraint: ValueConstraint {
                    value_type: ValueType::Any,
                    bounds: ValueBounds::None,
                    nullable: false,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
            },
            AttributeDescriptor {
                id: AttributeId::new(0x0001),
                name: "FeatureAttr",
                field_name: "feature_attr",
                access: AccessFlags::READ,
                constraint: ValueConstraint {
                    value_type: ValueType::U8,
                    bounds: ValueBounds::Unsigned { min: 0, max: 100 },
                    nullable: false,
                },
                conformance: &[
                    ConformanceOp::PushFeature(0x01),
                    ConformanceOp::Mandatory,
                    ConformanceOp::PushBool(true),
                    ConformanceOp::Disallow,
                    ConformanceOp::Otherwise,
                ],
            },
            AttributeDescriptor {
                id: AttributeId::new(0x0002),
                name: "ProvisionalAttr",
                field_name: "provisional_attr",
                access: AccessFlags::READ,
                constraint: ValueConstraint {
                    value_type: ValueType::Any,
                    bounds: ValueBounds::None,
                    nullable: false,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Provisional],
            },
            AttributeDescriptor {
                id: AttributeId::new(0x0003),
                name: "GreaterAttr",
                field_name: "greater_attr",
                access: AccessFlags::READ,
                constraint: ValueConstraint {
                    value_type: ValueType::Any,
                    bounds: ValueBounds::None,
                    nullable: false,
                },
                conformance: &[
                    ConformanceOp::PushAttrValue(0x0001),
                    ConformanceOp::PushConst(10),
                    ConformanceOp::Greater,
                    ConformanceOp::Mandatory,
                ],
            },
            AttributeDescriptor {
                id: AttributeId::new(0x0004),
                name: "ChoiceA",
                field_name: "choice_a",
                access: AccessFlags::READ,
                constraint: ValueConstraint {
                    value_type: ValueType::Any,
                    bounds: ValueBounds::None,
                    nullable: false,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Optional],
            },
            AttributeDescriptor {
                id: AttributeId::new(0x0005),
                name: "ChoiceB",
                field_name: "choice_b",
                access: AccessFlags::READ,
                constraint: ValueConstraint {
                    value_type: ValueType::Any,
                    bounds: ValueBounds::None,
                    nullable: false,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Optional],
            },
        ],
        commands: &[CommandDescriptor {
            id: CommandId::new(0x00),
            name: "Off",
            direction: CommandDirection::ClientToServer,
            access: AccessFlags::READ,
            fields: &[FieldDescriptor {
                id: FieldId::new(0),
                name: "Arg",
                constraint: ValueConstraint {
                    value_type: ValueType::U8,
                    bounds: ValueBounds::Unsigned { min: 0, max: 10 },
                    nullable: false,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
            }],
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
        }],
        events: &[EventDescriptor {
            id: EventId::new(0x00),
            name: "StateChanged",
            priority: EventPriority::Info,
            access: AccessFlags::READ,
            fields: &[],
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Optional],
        }],
        choice_groups: &[ChoiceGroup {
            min: 1,
            max: 1,
            more: false,
            members: &[0x0004, 0x0005],
        }],
    }];

    static MOCK_DEVICE_TYPES: &[DeviceTypeDescriptor] = &[DeviceTypeDescriptor {
        id: DeviceTypeId::new(0x0100),
        name: "OnOffLight",
        server_clusters: &[DeviceClusterReq {
            cluster_id: ClusterId::new(0x0006),
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
        }],
        client_clusters: &[],
        choice_groups: &[],
        includes: &[DeviceTypeIncludeItem {
            cluster_id: ClusterId::new(0x0006),
            is_server: true,
            required_attributes: &[AttributeId::new(0x0000)],
            required_commands: &[CommandId::new(0x00)],
            required_events: &[],
        }],
    }];

    fn mock_registry(expects_event_list: bool) -> SpecRegistry {
        SpecRegistry {
            expects_event_list,
            clusters: MOCK_CLUSTERS,
            device_types: MOCK_DEVICE_TYPES,
        }
    }

    #[test]
    fn rpn_conformance_feature_disallow_provisional_and_greater() {
        let reg = mock_registry(false);
        let mut cluster = Cluster::empty(ClusterId::new(0x0006));
        cluster.data_version = Optional::Present(DataVersion::new(42));
        cluster.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
        cluster.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
        cluster.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(0x00));

        // FeatureAttr (0x0001) is disallowed when feature bit 0x01 is 0.
        cluster.set_attribute(AttributeId::new(0x0001), AttributeValue::U8(5));
        assert_eq!(
            reg.validator().cluster(&cluster),
            Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
                AttributeId::new(0x0001)
            )))
        );

        // Enable feature bit 0x01 -> FeatureAttr becomes mandatory and valid.
        cluster.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(0x01));
        reg.validator().cluster(&cluster).unwrap();

        // Set FeatureAttr = 15 (> 10) -> GreaterAttr (0x0003) becomes mandatory.
        cluster.set_attribute(AttributeId::new(0x0001), AttributeValue::U8(15));
        assert_eq!(
            reg.validator().cluster(&cluster),
            Err(ClusterError::MissingMandatoryAttribute(AttributeId::new(0x0003)))
        );
        cluster.set_attribute(AttributeId::new(0x0003), AttributeValue::U8(1));
        reg.validator().cluster(&cluster).unwrap();

        // ProvisionalAttr: accepted when enforce_provisional = false, rejected when true.
        cluster.set_attribute(AttributeId::new(0x0002), AttributeValue::U8(9));
        reg.validator().cluster(&cluster).unwrap();
        assert_eq!(
            reg.validate_cluster_ref_with_mode(
                &cluster,
                ValidationMode::FullSnapshot {
                    enforce_provisional: true
                }
            ),
            Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
                AttributeId::new(0x0002)
            )))
        );
    }

    #[test]
    fn choice_group_underflow_and_overflow() {
        let reg = mock_registry(false);
        let mut cluster = Cluster::empty(ClusterId::new(0x0006));
        cluster.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));

        // Neither ChoiceA nor ChoiceB -> underflow.
        assert!(reg.validator().cluster(&cluster).is_err());

        // Both ChoiceA and ChoiceB -> overflow (max = 1).
        cluster.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
        cluster.set_attribute(AttributeId::new(0x0005), AttributeValue::U8(1));
        assert!(reg.validator().cluster(&cluster).is_err());
    }

    #[test]
    fn validate_value_bounds_innermost_field_id_and_status_codes() {
        let constraint = ValueConstraint {
            value_type: ValueType::Struct(&OUTER_STRUCT),
            bounds: ValueBounds::None,
            nullable: false,
        };

        // Wrong wire tag -> InvalidDataType (0x8D)
        assert_eq!(
            validate_value(&AttributeValue::U8(1), &constraint),
            Err((None, MatterStatus::InvalidDataType))
        );

        // Below minLength string -> ConstraintError (0x87) with FieldId(0)
        let bad_str = AttributeValue::Struct(vec![(FieldId::new(0), AttributeValue::String(String::from("a")))]);
        assert_eq!(
            validate_value(&bad_str, &constraint),
            Err((Some(FieldId::new(0)), MatterStatus::ConstraintError))
        );

        // Nested struct out-of-range integer -> innermost FieldId(1)
        let bad_nested = AttributeValue::Struct(vec![
            (FieldId::new(0), AttributeValue::String(String::from("ok"))),
            (
                FieldId::new(1),
                AttributeValue::Struct(vec![(FieldId::new(1), AttributeValue::U8(99))]),
            ),
        ]);
        assert_eq!(
            validate_value(&bad_nested, &constraint),
            Err((Some(FieldId::new(1)), MatterStatus::ConstraintError))
        );

        // Unknown struct FieldId -> ConstraintError
        let unknown_field = AttributeValue::Struct(vec![
            (FieldId::new(0), AttributeValue::String(String::from("ok"))),
            (FieldId::new(99), AttributeValue::U8(1)),
        ]);
        assert_eq!(
            validate_value(&unknown_field, &constraint),
            Err((Some(FieldId::new(99)), MatterStatus::ConstraintError))
        );

        // ListLength outer & item constraints
        let list_c = ValueConstraint {
            value_type: ValueType::List(&LIST_CONSTRAINT),
            bounds: ValueBounds::None,
            nullable: false,
        };
        assert_eq!(
            validate_value(&AttributeValue::List(vec![]), &list_c),
            Err((None, MatterStatus::ConstraintError))
        );
        assert_eq!(
            validate_value(&AttributeValue::List(vec![AttributeValue::U8(9)]), &list_c),
            Err((None, MatterStatus::ConstraintError))
        );
        validate_value(&AttributeValue::List(vec![AttributeValue::U8(3)]), &list_c).unwrap();
    }

    #[test]
    fn command_missing_mandatory_field_returns_invalid_command() {
        let reg = mock_registry(false);
        let cmd = Command::new(
            ClusterId::new(0x0006),
            CommandId::new(0x00),
            CommandDirection::ClientToServer,
            AttributeValue::Struct(vec![]),
        );
        assert_eq!(
            reg.validate_command_ref(&cmd),
            Err(CommandError::InvalidPayload {
                id: CommandId::new(0x00),
                field_id: Some(FieldId::new(0)),
                status: MatterStatus::InvalidCommand,
            })
        );
    }

    #[test]
    fn event_list_version_gating_and_non_mutating_endpoint() {
        let mut cluster = Cluster::empty(ClusterId::new(0x0006));
        cluster.data_version = Optional::Present(DataVersion::new(7));
        cluster.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
        cluster.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
        cluster.set_attribute(
            AttributeId::ACCEPTED_COMMAND_LIST,
            AttributeValue::List(vec![AttributeValue::U32(0x00)]),
        );

        // Registry expecting EventList (0xFFFA) requires it when cluster has events.
        let reg_with_ev_list = mock_registry(true);
        assert_eq!(
            reg_with_ev_list.validator().cluster(&cluster),
            Err(ClusterError::MissingMandatoryAttribute(AttributeId::EVENT_LIST))
        );

        // Registry not expecting EventList does not require it.
        let reg_no_ev_list = mock_registry(false);
        let mut ep = Endpoint::new(EndpointId::new(1), vec![], vec![], vec![]);
        ep.device_types.push(DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1));
        ep.server_clusters.push(cluster);
        // Extra/MEI cluster not in registry is permitted.
        ep.server_clusters.push(Cluster::empty(ClusterId::new(0xFFF1_0001)));

        reg_no_ev_list.validator().endpoint(&ep).unwrap();

        // Event validation preserves metadata.
        let ev = Event::new(
            ClusterId::new(0x0006),
            EventId::new(0x00),
            EventPriority::Info,
            AttributeValue::Struct(vec![]),
        );
        reg_no_ev_list.validator().event(&ev).unwrap();
    }

    #[test]
    fn validate_struct_payload_rejects_disallowed_fields() {
        static DISALLOW_STRUCT: StructDescriptor = StructDescriptor {
            name: "DisallowStruct",
            fields: &[
                FieldDescriptor {
                    id: FieldId::new(0),
                    name: "AlwaysAllowed",
                    constraint: ValueConstraint {
                        value_type: ValueType::U8,
                        bounds: ValueBounds::None,
                        nullable: false,
                    },
                    conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
                },
                FieldDescriptor {
                    id: FieldId::new(1),
                    name: "NeverAllowed",
                    constraint: ValueConstraint {
                        value_type: ValueType::Any,
                        bounds: ValueBounds::None,
                        nullable: false,
                    },
                    conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Disallow],
                },
                FieldDescriptor {
                    id: FieldId::new(2),
                    name: "FeatureOnly",
                    constraint: ValueConstraint {
                        value_type: ValueType::Any,
                        bounds: ValueBounds::None,
                        nullable: false,
                    },
                    conformance: &[
                        ConformanceOp::PushFeature(0x01),
                        ConformanceOp::Optional,
                        ConformanceOp::PushBool(true),
                        ConformanceOp::Disallow,
                        ConformanceOp::Otherwise,
                    ],
                },
            ],
        };

        let constraint = ValueConstraint {
            value_type: ValueType::Struct(&DISALLOW_STRUCT),
            bounds: ValueBounds::None,
            nullable: false,
        };

        // Static Disallow field must be rejected with ConstraintError and its FieldId.
        let with_disallowed = AttributeValue::Struct(vec![
            (FieldId::new(0), AttributeValue::U8(1)),
            (FieldId::new(1), AttributeValue::U8(2)),
        ]);
        assert_eq!(
            validate_value(&with_disallowed, &constraint),
            Err((Some(FieldId::new(1)), MatterStatus::ConstraintError))
        );

        // Feature-conditional Disallow field must be rejected when feature bit is unset,
        // and accepted when feature bit is set.
        let with_feature_field = AttributeValue::Struct(vec![
            (FieldId::new(0), AttributeValue::U8(1)),
            (FieldId::new(2), AttributeValue::U8(3)),
        ]);
        assert_eq!(
            validate_value_with_ctx(&with_feature_field, &constraint, 0x00, None),
            Err((Some(FieldId::new(2)), MatterStatus::ConstraintError))
        );
        validate_value_with_ctx(&with_feature_field, &constraint, 0x01, None).unwrap();
    }

    #[test]
    fn validate_node_ref_enforces_mandatory_under_full_snapshot() {
        let reg = mock_registry(false);

        let mut root_ep = Endpoint::new(EndpointId::ROOT, vec![], vec![], vec![]);
        root_ep
            .device_types
            .push(DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1));
        // OnOff cluster is missing mandatory OnOff attribute (0x0000) and ChoiceA/ChoiceB.
        root_ep.server_clusters.push(Cluster::empty(ClusterId::new(0x0006)));

        let node = Node::new(NodeId::new(1), vec![root_ep]);
        assert_eq!(
            reg.validator().node(&node),
            Err(NodeError::InvalidEndpoint {
                id: EndpointId::ROOT,
                source: EndpointError::InvalidCluster {
                    id: ClusterId::new(0x0006),
                    source: ClusterError::MissingMandatoryAttribute(AttributeId::new(0x0000)),
                },
            })
        );
    }
    #[test]
    fn validator_builder_and_descriptor_accessors() {
        static ATTRS: [AttributeDescriptor; 1] = [AttributeDescriptor {
            id: AttributeId::new(0x0000),
            name: "OnOff",
            field_name: "on_off",
            access: AccessFlags::READ,
            constraint: ValueConstraint {
                value_type: ValueType::Any,
                bounds: ValueBounds::None,
                nullable: false,
            },
            conformance: &[ConformanceOp::Mandatory],
        }];
        static CMDS: [CommandDescriptor; 1] = [CommandDescriptor {
            id: CommandId::new(0x0000),
            name: "Off",
            direction: CommandDirection::ClientToServer,
            access: AccessFlags::empty(),
            fields: &[],
            conformance: &[ConformanceOp::Mandatory],
        }];
        static EVENTS: [EventDescriptor; 1] = [EventDescriptor {
            id: EventId::new(0x0000),
            name: "StateChanged",
            priority: EventPriority::Info,
            access: AccessFlags::READ,
            fields: &[],
            conformance: &[ConformanceOp::Optional],
        }];
        static CLUSTERS: [ClusterDescriptor; 1] = [ClusterDescriptor {
            id: ClusterId::new(0x0006),
            name: "OnOff",
            attributes: &ATTRS,
            commands: &CMDS,
            events: &EVENTS,
            choice_groups: &[],
        }];
        static DEVICE_TYPES: [DeviceTypeDescriptor; 1] = [DeviceTypeDescriptor {
            id: DeviceTypeId::new(0x0100),
            name: "OnOffLight",
            server_clusters: &[],
            client_clusters: &[],
            choice_groups: &[],
            includes: &[],
        }];
        static REG: SpecRegistry = SpecRegistry {
            expects_event_list: false,
            clusters: &CLUSTERS,
            device_types: &DEVICE_TYPES,
        };

        assert_eq!(
            ValidationMode::default(),
            ValidationMode::FullSnapshot {
                enforce_provisional: false
            }
        );

        let v = REG.validator();
        let v_copy = v;
        assert_eq!(
            v_copy.mode(),
            ValidationMode::FullSnapshot {
                enforce_provisional: false
            }
        );
        assert_eq!(
            v.enforce_provisional(true).mode(),
            ValidationMode::FullSnapshot {
                enforce_provisional: true
            }
        );
        assert_eq!(v.partial_mode().mode(), ValidationMode::Partial);
        assert_eq!(
            REG.partial_validator().enforce_provisional(true).mode(),
            ValidationMode::Partial
        );
        assert_eq!(
            REG.partial_validator().full_mode().mode(),
            ValidationMode::FullSnapshot {
                enforce_provisional: false
            }
        );
        assert_eq!(
            RegistryValidator::partial(&REG)
                .with_mode(ValidationMode::FullSnapshot {
                    enforce_provisional: true
                })
                .mode(),
            ValidationMode::FullSnapshot {
                enforce_provisional: true
            }
        );

        assert_eq!(REG.clusters().len(), 1);
        assert_eq!(REG.device_types().len(), 1);
        let cluster = REG.find_cluster(ClusterId::new(0x0006)).unwrap();
        assert_eq!(cluster.id(), ClusterId::new(0x0006));
        assert_eq!(cluster.name(), "OnOff");
        assert_eq!(cluster.attributes().len(), 1);
        assert_eq!(cluster.commands().len(), 1);
        assert_eq!(cluster.events().len(), 1);

        let attr = cluster.find_attribute(AttributeId::new(0x0000)).unwrap();
        assert_eq!(attr.id(), AttributeId::new(0x0000));
        assert_eq!(attr.name(), "OnOff");
        assert_eq!(attr.field_name(), "on_off");
        assert_eq!(attr.access(), AccessFlags::READ);

        let cmd = cluster
            .find_command(CommandId::new(0x0000), CommandDirection::ClientToServer)
            .unwrap();
        assert_eq!(cmd.id(), CommandId::new(0x0000));
        assert_eq!(cmd.name(), "Off");
        assert_eq!(cmd.direction(), CommandDirection::ClientToServer);
        assert_eq!(cmd.access(), AccessFlags::empty());

        let ev = cluster.find_event(EventId::new(0x0000)).unwrap();
        assert_eq!(ev.id(), EventId::new(0x0000));
        assert_eq!(ev.name(), "StateChanged");
        assert_eq!(ev.priority(), EventPriority::Info);
        assert_eq!(ev.access(), AccessFlags::READ);

        assert_eq!(
            REG.validator().attribute(
                ClusterId::new(0x9999),
                &Attribute::new(AttributeId::new(0x0000), AttributeValue::Bool(true)),
            ),
            Err(AttributeError::UnsupportedCluster(ClusterId::new(0x9999)))
        );
        assert_eq!(
            REG.validator().event(&Event::new(
                ClusterId::new(0x0006),
                EventId::new(0x0000),
                EventPriority::Critical,
                AttributeValue::Struct(vec![]),
            )),
            Err(EventError::PriorityMismatch {
                expected: EventPriority::Info,
                actual: EventPriority::Critical,
            })
        );

        let dt = REG.find_device_type(DeviceTypeId::new(0x0100)).unwrap();
        assert_eq!(dt.id(), DeviceTypeId::new(0x0100));
        assert_eq!(dt.name(), "OnOffLight");

        let flags = AccessFlags::from_bits_truncate(0xFF);
        assert!(flags.contains(AccessFlags::READ));
        assert_eq!(AccessFlags::READ.bits(), 1 << 0);
    }

    #[test]
    fn access_flags_bitmap_ops() {
        assert!(AccessFlags::EMPTY.is_empty());
        assert_eq!(AccessFlags::EMPTY, AccessFlags::empty());

        let mut flags = AccessFlags::READ | AccessFlags::WRITE;
        assert!(!flags.is_empty());
        assert!(flags.contains(AccessFlags::READ));
        assert!(flags.contains(AccessFlags::WRITE));
        assert_eq!(
            flags.union(AccessFlags::TIMED),
            AccessFlags::READ | AccessFlags::WRITE | AccessFlags::TIMED
        );
        assert_eq!(flags.intersection(AccessFlags::READ), AccessFlags::READ);
        assert_eq!(u8::from(flags), 0x03);
        assert_eq!(alloc::format!("{flags}"), "0x03");

        flags.insert(AccessFlags::SCENE);
        assert!(flags.contains(AccessFlags::SCENE));
        flags.remove(AccessFlags::WRITE);
        assert!(!flags.contains(AccessFlags::WRITE));
        assert_eq!(flags ^ AccessFlags::READ, AccessFlags::SCENE);
        assert!((!AccessFlags::READ).contains(AccessFlags::WRITE));
        assert!(!(!AccessFlags::READ).contains(AccessFlags::READ));
    }
}
