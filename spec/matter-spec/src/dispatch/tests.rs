use alloc::vec::Vec;
#[cfg(feature = "serde")]
use alloc::{format, string::String};

use super::*;
use crate::{
    error::{AttributeError, ClusterError, CommandError, EndpointError, EventError},
    model::{
        Attribute, AttributeId, AttributeValue, Cluster, ClusterId, Command, CommandId, DeviceTypeEntry, DeviceTypeId,
        Endpoint, EndpointId, Event, EventId, EventPriority, MatterStatus,
    },
    traits::{AttributeSpec, ClusterSpec, CommandSpec, DeviceTypeSpec, EventSpec, PartialCluster},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
struct MockZstCommand;

impl From<MockZstCommand> for Command {
    fn from(_: MockZstCommand) -> Self {
        Self::new(
            MockZstCommand::CLUSTER_ID,
            MockZstCommand::COMMAND_ID,
            MockZstCommand::DIRECTION,
            AttributeValue::Null,
        )
    }
}

impl TryFrom<Command> for MockZstCommand {
    type Error = CommandError;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        if cmd.cluster_id != Self::CLUSTER_ID {
            return Err(CommandError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: cmd.cluster_id,
            });
        }
        if cmd.command_id != Self::COMMAND_ID {
            return Err(CommandError::CommandIdMismatch {
                expected: Self::COMMAND_ID,
                actual: cmd.command_id,
            });
        }
        Ok(Self)
    }
}

impl CommandSpec for MockZstCommand {
    type Response = ();

    const CLUSTER_ID: ClusterId = ClusterId::new(6);
    const CLUSTER_NAME: &'static str = "OnOff";
    const COMMAND_ID: CommandId = CommandId::new(0);
    const COMMAND_NAME: &'static str = "Off";
    const IS_EMPTY: bool = true;
}

impl MatterDescribe for MockZstCommand {
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema::concrete(
        EntityDescriptor::Command {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            command: Self::COMMAND_NAME,
            command_id: Self::COMMAND_ID.raw(),
        },
        Self::IS_EMPTY,
    );

    fn descriptor(&self) -> EntityDescriptor {
        EntityDescriptor::Command {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            command: Self::COMMAND_NAME,
            command_id: Self::COMMAND_ID.raw(),
        }
    }

    fn is_empty_payload(&self) -> bool {
        Self::IS_EMPTY
    }

    #[cfg(feature = "serde")]
    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_concrete_described(deserializer, Self.descriptor(), Self::IS_EMPTY)
    }

    #[cfg(feature = "serde")]
    fn deserialize_invocation<'de, D>(deserializer: D) -> Result<(EndpointId, Self), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_concrete_invocation(deserializer, Self.descriptor(), Self::IS_EMPTY)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
struct MockArgCommand {
    level: u8,
}

impl From<MockArgCommand> for Command {
    fn from(cmd: MockArgCommand) -> Self {
        Self::new(
            MockArgCommand::CLUSTER_ID,
            MockArgCommand::COMMAND_ID,
            MockArgCommand::DIRECTION,
            AttributeValue::U8(cmd.level),
        )
    }
}

impl TryFrom<Command> for MockArgCommand {
    type Error = CommandError;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        if cmd.cluster_id != Self::CLUSTER_ID {
            return Err(CommandError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: cmd.cluster_id,
            });
        }
        if cmd.command_id != Self::COMMAND_ID {
            return Err(CommandError::CommandIdMismatch {
                expected: Self::COMMAND_ID,
                actual: cmd.command_id,
            });
        }
        match cmd.payload {
            AttributeValue::U8(level) => Ok(Self { level }),
            _ => Err(CommandError::InvalidPayload {
                id: cmd.command_id,
                field_id: None,
                status: MatterStatus::InvalidDataType,
            }),
        }
    }
}

impl CommandSpec for MockArgCommand {
    type Response = ();

    const CLUSTER_ID: ClusterId = ClusterId::new(8);
    const CLUSTER_NAME: &'static str = "LevelControl";
    const COMMAND_ID: CommandId = CommandId::new(0);
    const COMMAND_NAME: &'static str = "MoveToLevel";
}

impl MatterDescribe for MockArgCommand {
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema::concrete(
        EntityDescriptor::Command {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            command: Self::COMMAND_NAME,
            command_id: Self::COMMAND_ID.raw(),
        },
        Self::IS_EMPTY,
    );

    fn descriptor(&self) -> EntityDescriptor {
        EntityDescriptor::Command {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            command: Self::COMMAND_NAME,
            command_id: Self::COMMAND_ID.raw(),
        }
    }

    #[cfg(feature = "serde")]
    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_concrete_described(deserializer, Self::default().descriptor(), Self::IS_EMPTY)
    }

    #[cfg(feature = "serde")]
    fn deserialize_invocation<'de, D>(deserializer: D) -> Result<(EndpointId, Self), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_concrete_invocation(deserializer, Self::default().descriptor(), Self::IS_EMPTY)
    }
}

#[cfg(feature = "serde")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
enum MockCommandEnum {
    Off(MockZstCommand),
    MoveToLevel(MockArgCommand),
}

#[cfg(feature = "serde")]
impl CommandMeta for MockCommandEnum {
    fn cluster_id(&self) -> ClusterId {
        match self {
            Self::Off(c) => c.cluster_id(),
            Self::MoveToLevel(c) => c.cluster_id(),
        }
    }

    fn cluster_name(&self) -> &'static str {
        match self {
            Self::Off(c) => c.cluster_name(),
            Self::MoveToLevel(c) => c.cluster_name(),
        }
    }

    fn command_id(&self) -> CommandId {
        match self {
            Self::Off(c) => c.command_id(),
            Self::MoveToLevel(c) => c.command_id(),
        }
    }

    fn command_name(&self) -> &'static str {
        match self {
            Self::Off(c) => c.command_name(),
            Self::MoveToLevel(c) => c.command_name(),
        }
    }

    fn is_empty_payload(&self) -> bool {
        match self {
            Self::Off(c) => CommandMeta::is_empty_payload(c),
            Self::MoveToLevel(c) => CommandMeta::is_empty_payload(c),
        }
    }
}

#[cfg(feature = "serde")]
fn select_mock_command_enum(
    scope_name: Option<&str>,
    scope_id: Option<u32>,
    item_name: Option<&str>,
    item_id: Option<u32>,
    payload: BufferedPayload,
) -> Result<MockCommandEnum, String> {
    enum Variant {
        Off,
        MoveToLevel,
    }

    let by_name = match item_name {
        Some("Off") => Some(Variant::Off),
        Some("MoveToLevel") => Some(Variant::MoveToLevel),
        Some(other) => return Err(format!("unknown command '{other}'")),
        None => None,
    };

    let by_id = match (scope_name, scope_id, item_id) {
        (_, Some(6), Some(0)) | (Some("OnOff") | None, None, Some(0)) => Some(Variant::Off),
        (_, Some(8), Some(0)) | (Some("LevelControl"), None, Some(0)) | (_, _, Some(1)) => Some(Variant::MoveToLevel),
        (_, _, Some(other)) => return Err(format!("unknown command_id {other}")),
        (_, _, None) => None,
    };

    let selected = match (by_name, by_id) {
        (Some(Variant::Off) | None, Some(Variant::Off)) | (Some(Variant::Off), None) => Variant::Off,
        (Some(Variant::MoveToLevel) | None, Some(Variant::MoveToLevel)) | (Some(Variant::MoveToLevel), None) => {
            Variant::MoveToLevel
        }
        (Some(_), Some(_)) => return Err(String::from("conflicting command name and command_id")),
        (None, None) => return Err(String::from("missing command discriminator")),
    };

    match selected {
        Variant::Off => payload.deserialize_into::<MockZstCommand>().map(MockCommandEnum::Off),
        Variant::MoveToLevel => payload
            .deserialize_into::<MockArgCommand>()
            .map(MockCommandEnum::MoveToLevel),
    }
}

#[cfg(feature = "serde")]
impl MatterDescribe for MockCommandEnum {
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema::open_enum(EntityKind::Command);

    fn descriptor(&self) -> EntityDescriptor {
        match self {
            Self::Off(c) => c.descriptor(),
            Self::MoveToLevel(c) => c.descriptor(),
        }
    }

    fn is_empty_payload(&self) -> bool {
        CommandMeta::is_empty_payload(self)
    }

    #[cfg(feature = "serde")]
    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_enum_described(deserializer, EntityKind::Command, None, select_mock_command_enum)
    }

    #[cfg(feature = "serde")]
    fn deserialize_invocation<'de, D>(deserializer: D) -> Result<(EndpointId, Self), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_enum_invocation(deserializer, EntityKind::Command, None, select_mock_command_enum)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
struct MockCluster {
    on_off: bool,
}

impl From<MockCluster> for Cluster {
    fn from(c: MockCluster) -> Self {
        Self::new(
            MockCluster::CLUSTER_ID,
            alloc::vec![Attribute::new(AttributeId::new(0), AttributeValue::Bool(c.on_off))],
        )
    }
}

impl TryFrom<Cluster> for MockCluster {
    type Error = ClusterError;

    fn try_from(c: Cluster) -> Result<Self, Self::Error> {
        if c.id != Self::CLUSTER_ID {
            return Err(ClusterError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: c.id,
            });
        }
        let val = c
            .attribute(AttributeId::new(0))
            .ok_or(ClusterError::MissingMandatoryAttribute(AttributeId::new(0)))?;
        match val {
            AttributeValue::Bool(on_off) => Ok(Self { on_off: *on_off }),
            _ => Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                id: AttributeId::new(0),
                field_id: None,
                status: MatterStatus::InvalidDataType,
            })),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct MockClusterPatch;

impl PartialCluster for MockClusterPatch {
    type Full = MockCluster;

    fn apply_attribute(&mut self, id: AttributeId, _value: AttributeValue) -> Result<(), AttributeError> {
        Err(AttributeError::UnsupportedAttribute(id))
    }

    fn merge(&mut self, _other: Self) {}

    fn try_into_full(self) -> Result<Self::Full, ClusterError> {
        Ok(MockCluster::default())
    }

    fn into_attributes(self) -> Vec<(AttributeId, AttributeValue)> {
        Vec::new()
    }
}

impl ClusterSpec for MockCluster {
    type Attribute = ();
    type Command = ();
    type CommandResponse = ();
    type Event = ();
    type Features = u32;
    type Patch = MockClusterPatch;

    const CLUSTER_ID: ClusterId = ClusterId::new(6);
    const CLUSTER_REVISION: u16 = 1;
    const NAME: &'static str = "OnOff";

    fn apply_patch(&mut self, _patch: Self::Patch) {}
}

impl MatterDescribe for MockCluster {
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema::concrete(
        EntityDescriptor::Cluster {
            cluster: Self::NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
        },
        Self::IS_EMPTY,
    );

    fn descriptor(&self) -> EntityDescriptor {
        EntityDescriptor::Cluster {
            cluster: Self::NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
        }
    }

    #[cfg(feature = "serde")]
    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_concrete_described(deserializer, self_desc_cluster(), Self::IS_EMPTY)
    }
}

#[cfg(feature = "serde")]
const fn self_desc_cluster() -> EntityDescriptor {
    EntityDescriptor::Cluster {
        cluster: MockCluster::NAME,
        cluster_id: MockCluster::CLUSTER_ID.raw(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
struct MockAttribute(bool);

impl From<MockAttribute> for Attribute {
    fn from(a: MockAttribute) -> Self {
        Self::new(MockAttribute::ATTRIBUTE_ID, AttributeValue::Bool(a.0))
    }
}

impl TryFrom<Attribute> for MockAttribute {
    type Error = AttributeError;

    fn try_from(a: Attribute) -> Result<Self, Self::Error> {
        if a.id != Self::ATTRIBUTE_ID {
            return Err(AttributeError::AttributeIdMismatch {
                expected: Self::ATTRIBUTE_ID,
                actual: a.id,
            });
        }
        match a.value {
            AttributeValue::Bool(v) => Ok(Self(v)),
            _ => Err(AttributeError::InvalidValue {
                id: a.id,
                field_id: None,
                status: MatterStatus::InvalidDataType,
            }),
        }
    }
}

impl AttributeSpec for MockAttribute {
    const ATTRIBUTE_ID: AttributeId = AttributeId::new(0);
    const ATTRIBUTE_NAME: &'static str = "OnOff";
    const CLUSTER_ID: ClusterId = ClusterId::new(6);
    const CLUSTER_NAME: &'static str = "OnOff";
}

impl MatterDescribe for MockAttribute {
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema::concrete(
        EntityDescriptor::Attribute {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            attribute: Self::ATTRIBUTE_NAME,
            attribute_id: Self::ATTRIBUTE_ID.raw(),
        },
        false,
    );

    fn descriptor(&self) -> EntityDescriptor {
        EntityDescriptor::Attribute {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            attribute: Self::ATTRIBUTE_NAME,
            attribute_id: Self::ATTRIBUTE_ID.raw(),
        }
    }

    #[cfg(feature = "serde")]
    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_concrete_described(deserializer, Self::descriptor_const(), false)
    }
}

#[cfg(feature = "serde")]
impl MockAttribute {
    const fn descriptor_const() -> EntityDescriptor {
        EntityDescriptor::Attribute {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            attribute: Self::ATTRIBUTE_NAME,
            attribute_id: Self::ATTRIBUTE_ID.raw(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
struct MockEvent {
    new_position: u8,
}

impl From<MockEvent> for Event {
    fn from(e: MockEvent) -> Self {
        Self::new(
            MockEvent::CLUSTER_ID,
            MockEvent::EVENT_ID,
            MockEvent::PRIORITY,
            AttributeValue::U8(e.new_position),
        )
    }
}

impl TryFrom<Event> for MockEvent {
    type Error = EventError;

    fn try_from(e: Event) -> Result<Self, Self::Error> {
        if e.cluster_id != Self::CLUSTER_ID {
            return Err(EventError::ClusterIdMismatch {
                expected: Self::CLUSTER_ID,
                actual: e.cluster_id,
            });
        }
        if e.event_id != Self::EVENT_ID {
            return Err(EventError::EventIdMismatch {
                expected: Self::EVENT_ID,
                actual: e.event_id,
            });
        }
        match e.payload {
            AttributeValue::U8(new_position) => Ok(Self { new_position }),
            _ => Err(EventError::InvalidPayload {
                id: e.event_id,
                field_id: None,
                status: MatterStatus::InvalidDataType,
            }),
        }
    }
}

impl EventSpec for MockEvent {
    const CLUSTER_ID: ClusterId = ClusterId::new(0x003B);
    const CLUSTER_NAME: &'static str = "Switch";
    const EVENT_ID: EventId = EventId::new(0);
    const EVENT_NAME: &'static str = "SwitchLatched";
    const PRIORITY: EventPriority = EventPriority::Info;
}

impl MatterDescribe for MockEvent {
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema::concrete(
        EntityDescriptor::Event {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            event: Self::EVENT_NAME,
            event_id: Self::EVENT_ID.raw(),
        },
        Self::IS_EMPTY,
    );

    fn descriptor(&self) -> EntityDescriptor {
        EntityDescriptor::Event {
            cluster: Self::CLUSTER_NAME,
            cluster_id: Self::CLUSTER_ID.raw(),
            event: Self::EVENT_NAME,
            event_id: Self::EVENT_ID.raw(),
        }
    }

    #[cfg(feature = "serde")]
    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_concrete_described(
            deserializer,
            EntityDescriptor::Event {
                cluster: Self::CLUSTER_NAME,
                cluster_id: Self::CLUSTER_ID.raw(),
                event: Self::EVENT_NAME,
                event_id: Self::EVENT_ID.raw(),
            },
            Self::IS_EMPTY,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
struct MockDevice {
    on_off: MockCluster,
}

impl TryFrom<Endpoint> for MockDevice {
    type Error = EndpointError;

    fn try_from(ep: Endpoint) -> Result<Self, Self::Error> {
        if !ep.has_device_type(Self::DEVICE_TYPE_ID) {
            return Err(EndpointError::DeviceTypeMismatch {
                expected: Self::DEVICE_TYPE_ID,
                actual: ep.primary_device_type().map(|dt| dt.id).unwrap_or(DeviceTypeId::new(0)),
            });
        }
        let cluster = ep
            .server_cluster(MockCluster::CLUSTER_ID)
            .cloned()
            .ok_or(EndpointError::MissingMandatoryServerCluster(MockCluster::CLUSTER_ID))?;
        let on_off = MockCluster::try_from(cluster).map_err(|source| EndpointError::InvalidCluster {
            id: MockCluster::CLUSTER_ID,
            source,
        })?;
        Ok(Self { on_off })
    }
}

impl DeviceTypeSpec for MockDevice {
    type View<'a> = ();

    const DEVICE_TYPE_ID: DeviceTypeId = DeviceTypeId::new(0x0100);
    const NAME: &'static str = "OnOffLight";
    const REVISION: u16 = 4;

    fn into_endpoint(self, id: EndpointId) -> Endpoint {
        Endpoint::new(
            id,
            alloc::vec![DeviceTypeEntry::new(MockDevice::DEVICE_TYPE_ID, MockDevice::REVISION)],
            alloc::vec![Cluster::from(self.on_off)],
            Vec::new(),
        )
    }

    fn view(_endpoint: &Endpoint) -> Result<Self::View<'_>, EndpointError> {
        Ok(())
    }
}

impl MatterDescribe for MockDevice {
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema::concrete(
        EntityDescriptor::Device {
            device_type: Self::NAME,
            device_type_id: Self::DEVICE_TYPE_ID.raw(),
        },
        Self::IS_EMPTY,
    );

    fn descriptor(&self) -> EntityDescriptor {
        EntityDescriptor::Device {
            device_type: Self::NAME,
            device_type_id: Self::DEVICE_TYPE_ID.raw(),
        }
    }

    #[cfg(feature = "serde")]
    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_concrete_described(
            deserializer,
            EntityDescriptor::Device {
                device_type: Self::NAME,
                device_type_id: Self::DEVICE_TYPE_ID.raw(),
            },
            Self::IS_EMPTY,
        )
    }
}

#[test]
fn runtime_meta_blanket_impls() {
    let zst = MockZstCommand;
    assert_eq!(zst.cluster_id(), ClusterId::new(6), "cluster_id must match");
    assert_eq!(zst.cluster_name(), "OnOff", "cluster_name must match");
    assert_eq!(zst.command_id(), CommandId::new(0), "command_id must match");
    assert_eq!(zst.command_name(), "Off", "command_name must match");
    assert!(
        CommandMeta::is_empty_payload(&zst),
        "MockZstCommand must report empty payload"
    );

    let cluster = MockCluster { on_off: true };
    assert_eq!(cluster.cluster_id(), ClusterId::new(6), "cluster_id must match");
    assert_eq!(cluster.cluster_name(), "OnOff", "cluster_name must match");

    let attr = MockAttribute(true);
    assert_eq!(attr.attribute_id(), AttributeId::new(0), "attribute_id must match");
    assert_eq!(attr.attribute_name(), "OnOff", "attribute_name must match");

    let ev = MockEvent { new_position: 2 };
    assert_eq!(ev.event_id(), EventId::new(0), "event_id must match");
    assert_eq!(ev.event_name(), "SwitchLatched", "event_name must match");

    let dev = MockDevice::default();
    assert_eq!(
        dev.device_type_id(),
        DeviceTypeId::new(0x0100),
        "device_type_id must match"
    );
    assert_eq!(dev.device_type_name(), "OnOffLight", "device_type_name must match");
    assert_eq!(dev.revision(), 4, "revision must match");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MockOtherAttribute(u8);

impl From<MockOtherAttribute> for Attribute {
    fn from(a: MockOtherAttribute) -> Self {
        Self::new(MockOtherAttribute::ATTRIBUTE_ID, AttributeValue::U8(a.0))
    }
}

impl TryFrom<Attribute> for MockOtherAttribute {
    type Error = AttributeError;

    fn try_from(a: Attribute) -> Result<Self, Self::Error> {
        if a.id != Self::ATTRIBUTE_ID {
            return Err(AttributeError::AttributeIdMismatch {
                expected: Self::ATTRIBUTE_ID,
                actual: a.id,
            });
        }
        match a.value {
            AttributeValue::U8(v) => Ok(Self(v)),
            _ => Err(AttributeError::InvalidValue {
                id: a.id,
                field_id: None,
                status: MatterStatus::InvalidDataType,
            }),
        }
    }
}

impl AttributeSpec for MockOtherAttribute {
    const ATTRIBUTE_ID: AttributeId = AttributeId::new(0);
    const ATTRIBUTE_NAME: &'static str = "CurrentLevel";
    const CLUSTER_ID: ClusterId = ClusterId::new(8);
    const CLUSTER_NAME: &'static str = "LevelControl";
}

#[cfg(feature = "serde")]
impl MockAttribute {
    const CLUSTER_ID: ClusterId = <Self as AttributeSpec>::CLUSTER_ID;
}

impl MockOtherAttribute {
    const CLUSTER_ID: ClusterId = <Self as AttributeSpec>::CLUSTER_ID;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockGlobalAttribute {
    OnOff(MockAttribute),
    LevelControl(MockOtherAttribute),
}

impl AttributeMeta for MockGlobalAttribute {
    fn cluster_id(&self) -> ClusterId {
        match self {
            Self::OnOff(a) => a.cluster_id(),
            Self::LevelControl(a) => a.cluster_id(),
        }
    }

    fn cluster_name(&self) -> &'static str {
        match self {
            Self::OnOff(a) => a.cluster_name(),
            Self::LevelControl(a) => a.cluster_name(),
        }
    }

    fn attribute_id(&self) -> AttributeId {
        match self {
            Self::OnOff(a) => a.attribute_id(),
            Self::LevelControl(a) => a.attribute_id(),
        }
    }

    fn attribute_name(&self) -> &'static str {
        match self {
            Self::OnOff(a) => a.attribute_name(),
            Self::LevelControl(a) => a.attribute_name(),
        }
    }
}

crate::impl_global_variant!(attribute, MockAttribute, MockGlobalAttribute, OnOff);
crate::impl_global_variant!(attribute, MockOtherAttribute, MockGlobalAttribute, LevelControl);

#[test]
fn impl_global_variant_attribute_returns_cluster_id_mismatch() {
    let global = MockGlobalAttribute::from(MockOtherAttribute(42));
    assert_eq!(MockOtherAttribute::try_from(global), Ok(MockOtherAttribute(42)));
    assert_eq!(
        MockAttribute::try_from(global),
        Err(AttributeError::ClusterIdMismatch {
            expected: ClusterId::new(6),
            actual: ClusterId::new(8),
        })
    );
}

#[cfg(feature = "serde")]
#[test]
fn described_zst_and_arg_commands_serialize_and_deserialize() {
    let zst_json = serde_json::to_value(Described::new(MockZstCommand)).expect("serialize zst");
    assert_eq!(
        zst_json,
        serde_json::json!({
            "cluster": "OnOff",
            "cluster_id": 6,
            "command": "Off",
            "command_id": 0,
        }),
        "ZST command must omit payload"
    );

    // Optional metadata and empty payload tolerance
    assert_eq!(
        serde_json::from_str::<Described<MockZstCommand>>("{}")
            .expect("empty object")
            .into_inner(),
        MockZstCommand,
        "omitted metadata and payload should deserialize"
    );
    assert_eq!(
        serde_json::from_str::<Described<MockZstCommand>>(r#"{"payload": null}"#)
            .expect("null payload")
            .into_inner(),
        MockZstCommand,
        "null payload should deserialize"
    );
    assert_eq!(
        serde_json::from_str::<Described<MockZstCommand>>(r#"{"payload": {}}"#)
            .expect("empty object payload")
            .into_inner(),
        MockZstCommand,
        "empty object payload should deserialize"
    );

    let arg = MockArgCommand { level: 50 };
    let arg_json = serde_json::to_value(Described::new(arg)).expect("serialize arg");
    assert_eq!(
        arg_json,
        serde_json::json!({
            "cluster": "LevelControl",
            "cluster_id": 8,
            "command": "MoveToLevel",
            "command_id": 0,
            "payload": { "level": 50 },
        }),
        "parameterized command must include payload"
    );

    let omitted_meta: Described<MockArgCommand> =
        serde_json::from_str(r#"{"payload": {"level": 50}}"#).expect("omitted metadata");
    assert_eq!(omitted_meta.into_inner(), arg, "omitted metadata must deserialize");
}

#[cfg(feature = "serde")]
#[test]
fn described_concrete_rejects_mismatches_and_invalid_payloads() {
    let bad_cluster_id = serde_json::from_str::<Described<MockZstCommand>>(r#"{"cluster_id": 99}"#);
    assert!(bad_cluster_id.is_err(), "mismatched cluster_id must fail");

    let bad_cluster = serde_json::from_str::<Described<MockZstCommand>>(r#"{"cluster": "Wrong"}"#);
    assert!(bad_cluster.is_err(), "mismatched cluster name must fail");

    let bad_cmd_id = serde_json::from_str::<Described<MockZstCommand>>(r#"{"command_id": 42}"#);
    assert!(bad_cmd_id.is_err(), "mismatched command_id must fail");

    let bad_cmd = serde_json::from_str::<Described<MockZstCommand>>(r#"{"command": "On"}"#);
    assert!(bad_cmd.is_err(), "mismatched command name must fail");

    let non_empty_zst = serde_json::from_str::<Described<MockZstCommand>>(r#"{"payload": {"extra": 1}}"#);
    assert!(non_empty_zst.is_err(), "non-empty payload on ZST command must fail");

    let missing_arg_payload = serde_json::from_str::<Described<MockArgCommand>>(r#"{"command": "MoveToLevel"}"#);
    assert!(
        missing_arg_payload.is_err(),
        "missing payload on parameterized command must fail"
    );
}

#[cfg(feature = "serde")]
#[test]
fn described_polymorphic_enum_dispatch() {
    let by_name: Described<MockCommandEnum> =
        serde_json::from_str(r#"{"command": "Off"}"#).expect("select by command name");
    assert_eq!(by_name.into_inner(), MockCommandEnum::Off(MockZstCommand));

    let by_id: Described<MockCommandEnum> =
        serde_json::from_str(r#"{"cluster_id": 6, "command_id": 0}"#).expect("select by command_id");
    assert_eq!(by_id.into_inner(), MockCommandEnum::Off(MockZstCommand));

    // Out-of-order payload before discriminator
    let out_of_order: Described<MockCommandEnum> =
        serde_json::from_str(r#"{"payload": {"level": 77}, "command": "MoveToLevel"}"#)
            .expect("out-of-order payload before discriminator");
    assert_eq!(
        out_of_order.into_inner(),
        MockCommandEnum::MoveToLevel(MockArgCommand { level: 77 })
    );

    let both_matching: Described<MockCommandEnum> = serde_json::from_str(
        r#"{"cluster": "LevelControl", "cluster_id": 8, "command": "MoveToLevel", "command_id": 0, "payload": {"level": 12}}"#,
    )
    .expect("both matching");
    assert_eq!(
        both_matching.into_inner(),
        MockCommandEnum::MoveToLevel(MockArgCommand { level: 12 })
    );

    let missing_disc = serde_json::from_str::<Described<MockCommandEnum>>(r#"{"cluster": "OnOff"}"#);
    assert!(missing_disc.is_err(), "missing command discriminator must fail");

    let conflicting = serde_json::from_str::<Described<MockCommandEnum>>(r#"{"command": "Off", "command_id": 1}"#);
    assert!(conflicting.is_err(), "conflicting name and id must fail");
}

#[cfg(feature = "serde")]
#[test]
fn invocation_envelope_roundtrip_and_validation() {
    let zst_inv = Invocation::new(EndpointId::new(1), MockZstCommand);
    let zst_json = serde_json::to_value(zst_inv).expect("serialize zst invocation");
    assert_eq!(
        zst_json,
        serde_json::json!({
            "endpoint_id": 1,
            "cluster": "OnOff",
            "cluster_id": 6,
            "command": "Off",
            "command_id": 0,
        }),
        "Invocation of ZST command must omit payload"
    );
    let zst_back: Invocation<MockZstCommand> = serde_json::from_value(zst_json).expect("deserialize zst invocation");
    assert_eq!(zst_back, zst_inv, "ZST invocation must round-trip");

    let arg_inv = Invocation::new(EndpointId::new(2), MockArgCommand { level: 128 });
    let arg_json = serde_json::to_value(arg_inv).expect("serialize arg invocation");
    let arg_back: Invocation<MockArgCommand> = serde_json::from_value(arg_json).expect("deserialize arg invocation");
    assert_eq!(arg_back, arg_inv, "arg invocation must round-trip");

    let enum_inv = Invocation::new(
        EndpointId::new(3),
        MockCommandEnum::MoveToLevel(MockArgCommand { level: 254 }),
    );
    let enum_json = serde_json::to_value(enum_inv).expect("serialize enum invocation");
    let enum_back: Invocation<MockCommandEnum> =
        serde_json::from_value(enum_json).expect("deserialize enum invocation");
    assert_eq!(enum_back, enum_inv, "enum invocation must round-trip");

    let missing_ep = serde_json::from_str::<Invocation<MockZstCommand>>(r#"{"command": "Off"}"#);
    assert!(missing_ep.is_err(), "Invocation without endpoint_id must fail");
}

#[cfg(feature = "serde")]
#[test]
fn described_all_five_entity_kinds() {
    let cluster = Described::new(MockCluster { on_off: true });
    let cluster_json = serde_json::to_value(cluster).expect("serialize cluster");
    assert_eq!(cluster_json["cluster"], "OnOff", "cluster name must match");
    assert_eq!(cluster_json["cluster_id"], 6, "cluster_id must match");
    let cluster_back: Described<MockCluster> = serde_json::from_value(cluster_json).expect("deserialize cluster");
    assert_eq!(cluster_back, cluster, "cluster must round-trip");

    let attr = Described::new(MockAttribute(true));
    let attr_json = serde_json::to_value(attr).expect("serialize attribute");
    assert_eq!(attr_json["attribute"], "OnOff", "attribute name must match");
    assert_eq!(attr_json["attribute_id"], 0, "attribute_id must match");
    let attr_back: Described<MockAttribute> = serde_json::from_value(attr_json).expect("deserialize attribute");
    assert_eq!(attr_back, attr, "attribute must round-trip");

    let ev = Described::new(MockEvent { new_position: 3 });
    let ev_json = serde_json::to_value(ev).expect("serialize event");
    assert_eq!(ev_json["event"], "SwitchLatched", "event name must match");
    assert_eq!(ev_json["event_id"], 0, "event_id must match");
    let ev_back: Described<MockEvent> = serde_json::from_value(ev_json).expect("deserialize event");
    assert_eq!(ev_back, ev, "event must round-trip");

    let dev = Described::new(MockDevice {
        on_off: MockCluster { on_off: false },
    });
    let dev_json = serde_json::to_value(dev).expect("serialize device");
    assert_eq!(dev_json["device_type"], "OnOffLight", "device_type name must match");
    assert_eq!(dev_json["device_type_id"], 0x0100, "device_type_id must match");
    let dev_back: Described<MockDevice> = serde_json::from_value(dev_json).expect("deserialize device");
    assert_eq!(dev_back, dev, "device must round-trip");
}

#[cfg(feature = "schemars")]
#[test]
fn schemars_emits_const_constraints() {
    let zst_schema = schemars::schema_for!(Described<MockZstCommand>);
    let zst_val = serde_json::to_value(&zst_schema).expect("serialize zst schema");
    assert_eq!(zst_val["properties"]["cluster"]["const"], "OnOff");
    assert_eq!(zst_val["properties"]["cluster_id"]["const"], 6);
    assert_eq!(zst_val["properties"]["command"]["const"], "Off");
    assert_eq!(zst_val["properties"]["command_id"]["const"], 0);
    assert!(
        zst_val["properties"].get("payload").is_none(),
        "ZST schema must omit payload property"
    );

    let arg_schema = schemars::schema_for!(Described<MockArgCommand>);
    let arg_val = serde_json::to_value(&arg_schema).expect("serialize arg schema");
    assert_eq!(arg_val["properties"]["cluster"]["const"], "LevelControl");
    assert_eq!(arg_val["properties"]["cluster_id"]["const"], 8);
    assert_eq!(arg_val["properties"]["command"]["const"], "MoveToLevel");
    assert_eq!(arg_val["properties"]["command_id"]["const"], 0);
    assert!(
        arg_val["properties"].get("payload").is_some(),
        "parameterized command schema must include payload"
    );

    let inv_schema = schemars::schema_for!(Invocation<MockArgCommand>);
    let inv_val = serde_json::to_value(&inv_schema).expect("serialize invocation schema");
    assert!(
        inv_val["properties"].get("endpoint_id").is_some(),
        "Invocation schema must include endpoint_id"
    );
    assert_eq!(inv_val["properties"]["cluster"]["const"], "LevelControl");
    assert_eq!(inv_val["properties"]["command"]["const"], "MoveToLevel");
}
