//! ZAP XML schema models corresponding to the official upstream `project-chip/alchemy` definitions.
//!
//! Upstream Repository: <https://github.com/project-chip/alchemy>
//! Commit: `fe4593a2f200fd861ebb3fc6e00dbbfc8e3c66ea`.
//!
//! All structs in this module enforce strict, exhaustive deserialization via
//! `#[serde(deny_unknown_fields)]` and exhaustive `$value` enums to ensure no unknown XML tags or
//! attributes are ignored.

use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfiguratorXml {
    #[serde(rename = "@xmlns:xsi", default)]
    pub(crate) xmlns_xsi: Option<String>,
    #[serde(rename = "@xsi:noNamespaceSchemaLocation", default)]
    pub(crate) schema_location: Option<String>,
    #[serde(rename = "@noNamespaceSchemaLocation", default)]
    pub(crate) no_namespace_schema_location: Option<String>,
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<ConfiguratorItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum ConfiguratorItem {
    #[serde(rename = "domain")]
    Domain(DomainXml),
    #[serde(rename = "cluster")]
    Cluster(ClusterXml),
    #[serde(rename = "enum")]
    Enum(EnumXml),
    #[serde(rename = "struct")]
    Struct(StructXml),
    #[serde(rename = "bitmap")]
    Bitmap(BitmapXml),
    #[serde(rename = "accessControl")]
    AccessControl(AccessControlXml),
    #[serde(rename = "atomic")]
    Atomic(AtomicXml),
    #[serde(rename = "clusterExtension")]
    ClusterExtension(ClusterExtensionXml),
    #[serde(rename = "global")]
    Global(GlobalXml),
    #[serde(rename = "deviceType")]
    DeviceType(DeviceTypeXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DomainXml {
    #[serde(rename = "@name", default)]
    pub(crate) name: Option<String>,
    #[serde(rename = "@spec", default)]
    pub(crate) spec: Option<String>,
    #[serde(rename = "@dependsOn", default)]
    pub(crate) depends_on: Option<String>,
    #[serde(rename = "@certifiable", default)]
    pub(crate) certifiable: Option<String>,
    #[serde(rename = "$text", default)]
    pub(crate) text: Option<String>,
}

impl DomainXml {
    pub(crate) fn as_str(&self) -> Option<&str> {
        self.name
            .as_deref()
            .map(str::trim)
            .filter(|trimmed| !trimmed.is_empty())
            .or_else(|| {
                self.text
                    .as_deref()
                    .map(str::trim)
                    .filter(|trimmed| !trimmed.is_empty())
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClusterXml {
    #[serde(rename = "@code", default)]
    pub(crate) code_attr: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,
    #[serde(rename = "@singleton", default)]
    pub(crate) singleton: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<ClusterItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum ClusterItem {
    #[serde(rename = "name")]
    Name(SimpleTextXml),
    #[serde(rename = "domain")]
    Domain(DomainXml),
    #[serde(rename = "description")]
    Description(SimpleTextXml),
    #[serde(rename = "code")]
    Code(SimpleTextXml),
    #[serde(rename = "define")]
    Define(SimpleTextXml),
    #[serde(rename = "client")]
    Client(ClientServerXml),
    #[serde(rename = "server")]
    Server(ClientServerXml),
    #[serde(rename = "globalAttribute")]
    GlobalAttribute(GlobalAttributeXml),
    #[serde(rename = "features")]
    Features(FeaturesXml),
    #[serde(rename = "attribute")]
    Attribute(AttributeXml),
    #[serde(rename = "command")]
    Command(CommandXml),
    #[serde(rename = "event")]
    Event(EventXml),
    #[serde(rename = "tag")]
    Tag(TagXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SimpleTextXml {
    #[serde(rename = "$text", default)]
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClientServerXml {
    #[serde(rename = "@init", default)]
    pub(crate) init: Option<String>,
    #[serde(rename = "@tick", default)]
    pub(crate) tick: Option<String>,
    #[serde(rename = "@tickFrequency", default)]
    pub(crate) tick_frequency: Option<String>,
    #[serde(rename = "$text", default)]
    pub(crate) text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GlobalAttributeXml {
    #[serde(rename = "@code")]
    pub(crate) code: String,
    #[serde(rename = "@side", default)]
    pub(crate) side: Option<String>,
    #[serde(rename = "@type", default)]
    pub(crate) type_attr: Option<String>,
    #[serde(rename = "@value", default)]
    pub(crate) value: Option<String>,
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<GlobalAttributeItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum GlobalAttributeItem {
    #[serde(rename = "featureBit")]
    FeatureBit(FeatureBitXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FeatureBitXml {
    #[serde(rename = "@bit")]
    pub(crate) bit: String,
    #[serde(rename = "@tag", default)]
    pub(crate) tag: Option<String>,
    #[serde(rename = "@code", default)]
    pub(crate) code: Option<String>,
    #[serde(rename = "@name", default)]
    pub(crate) name: Option<String>,
    #[serde(rename = "@summary", default)]
    pub(crate) summary: Option<String>,
    #[serde(rename = "$text", default)]
    pub(crate) text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TagXml {
    #[serde(rename = "@name", default)]
    pub(crate) name: Option<String>,
    #[serde(rename = "@description", default)]
    pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FeaturesXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<FeaturesItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum FeaturesItem {
    #[serde(rename = "feature")]
    Feature(FeatureXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FeatureXml {
    #[serde(rename = "@bit", default)]
    pub(crate) bit: Option<String>,
    #[serde(rename = "@code", default)]
    pub(crate) code: Option<String>,
    #[serde(rename = "@name", default)]
    pub(crate) name: Option<String>,
    #[serde(rename = "@summary", default)]
    pub(crate) summary: Option<String>,
    #[serde(rename = "@default", default)]
    pub(crate) default: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<FeatureItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum FeatureItem {
    #[serde(rename = "mandatoryConform")]
    MandatoryConform(ConformXml),
    #[serde(rename = "optionalConform")]
    OptionalConform(ConformXml),
    #[serde(rename = "otherwiseConform")]
    OtherwiseConform(OtherwiseConformXml),
    #[serde(rename = "deprecateConform")]
    DeprecateConform(EmptyConformXml),
    #[serde(rename = "disallowConform")]
    DisallowConform(EmptyConformXml),
    #[serde(rename = "describedConform")]
    DescribedConform(EmptyConformXml),
    #[serde(rename = "provisionalConform")]
    ProvisionalConform(EmptyConformXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttributeXml {
    #[serde(rename = "@code")]
    pub(crate) code: String,
    #[serde(rename = "@name", default)]
    pub(crate) name_attr: Option<String>,
    #[serde(rename = "@define", default)]
    pub(crate) define: Option<String>,
    #[serde(rename = "@type", default)]
    pub(crate) type_attr: Option<String>,
    #[serde(rename = "@side", default)]
    pub(crate) side: Option<String>,
    #[serde(rename = "@optional", default)]
    pub(crate) optional: Option<String>,
    #[serde(rename = "@writable", default)]
    pub(crate) writable: Option<String>,
    #[serde(rename = "@readable", default)]
    pub(crate) readable: Option<String>,
    #[serde(rename = "@reportable", default)]
    pub(crate) reportable: Option<String>,
    #[serde(rename = "@default", default)]
    pub(crate) default: Option<String>,
    #[serde(rename = "@min", default)]
    pub(crate) min: Option<String>,
    #[serde(rename = "@max", default)]
    pub(crate) max: Option<String>,
    #[serde(rename = "@minLength", default)]
    pub(crate) min_length: Option<String>,
    #[serde(rename = "@length", default)]
    pub(crate) length: Option<String>,
    #[serde(rename = "@entryType", default)]
    pub(crate) entry_type: Option<String>,
    #[serde(rename = "@isNullable", default)]
    pub(crate) is_nullable: Option<String>,
    #[serde(rename = "@isFabricSensitive", default)]
    pub(crate) is_fabric_sensitive: Option<String>,
    #[serde(rename = "@mustUseTimedWrite", default)]
    pub(crate) must_use_timed_write: Option<String>,
    #[serde(rename = "@mustUseAtomicWrite", default)]
    pub(crate) must_use_atomic_write: Option<String>,
    #[serde(rename = "@introducedIn", default)]
    pub(crate) introduced_in: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<AttributeItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum AttributeItem {
    #[serde(rename = "description")]
    Description(SimpleTextXml),
    #[serde(rename = "access")]
    Access(AccessXml),
    #[serde(rename = "quality")]
    Quality(QualityXml),
    #[serde(rename = "mandatoryConform")]
    MandatoryConform(ConformXml),
    #[serde(rename = "optionalConform")]
    OptionalConform(ConformXml),
    #[serde(rename = "otherwiseConform")]
    OtherwiseConform(OtherwiseConformXml),
    #[serde(rename = "deprecateConform")]
    DeprecateConform(EmptyConformXml),
    #[serde(rename = "disallowConform")]
    DisallowConform(EmptyConformXml),
    #[serde(rename = "describedConform")]
    DescribedConform(EmptyConformXml),
    #[serde(rename = "greaterOrEqualTerm")]
    GreaterOrEqualTerm(GreaterOrEqualTermXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CommandXml {
    #[serde(rename = "@code")]
    pub(crate) code: String,
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@source", default)]
    pub(crate) source: Option<String>,
    #[serde(rename = "@optional", default)]
    pub(crate) optional: Option<String>,
    #[serde(rename = "@response", default)]
    pub(crate) response: Option<String>,
    #[serde(rename = "@mustUseTimedInvoke", default)]
    pub(crate) must_use_timed_invoke: Option<String>,
    #[serde(rename = "@isFabricScoped", default)]
    pub(crate) is_fabric_scoped: Option<String>,
    #[serde(rename = "@disableDefaultResponse", default)]
    pub(crate) disable_default_response: Option<String>,
    #[serde(rename = "@cli", default)]
    pub(crate) cli: Option<String>,
    #[serde(rename = "@cliFunctionName", default)]
    pub(crate) cli_function_name: Option<String>,
    #[serde(rename = "@noDefaultImplementation", default)]
    pub(crate) no_default_implementation: Option<String>,
    #[serde(rename = "@manufacturerCode", default)]
    pub(crate) manufacturer_code: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<CommandItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum CommandItem {
    #[serde(rename = "description")]
    Description(SimpleTextXml),
    #[serde(rename = "access")]
    Access(AccessXml),
    #[serde(rename = "quality")]
    Quality(QualityXml),
    #[serde(rename = "arg")]
    Arg(Box<ArgXml>),
    #[serde(rename = "mandatoryConform")]
    MandatoryConform(ConformXml),
    #[serde(rename = "optionalConform")]
    OptionalConform(ConformXml),
    #[serde(rename = "otherwiseConform")]
    OtherwiseConform(OtherwiseConformXml),
    #[serde(rename = "disallowConform")]
    DisallowConform(EmptyConformXml),
    #[serde(rename = "describedConform")]
    DescribedConform(EmptyConformXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArgXml {
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@type")]
    pub(crate) type_attr: String,
    #[serde(rename = "@id", default)]
    pub(crate) id: Option<String>,
    #[serde(rename = "@fieldId", default)]
    pub(crate) field_id: Option<String>,
    #[serde(rename = "@optional", default)]
    pub(crate) optional: Option<String>,
    #[serde(rename = "@array", default)]
    pub(crate) array: Option<String>,
    #[serde(rename = "@isNullable", default)]
    pub(crate) is_nullable: Option<String>,
    #[serde(rename = "@entryType", default)]
    pub(crate) entry_type: Option<String>,
    #[serde(rename = "@length", default)]
    pub(crate) length: Option<String>,
    #[serde(rename = "@lenght", default)]
    pub(crate) lenght_typo: Option<String>,
    #[serde(rename = "@minLength", default)]
    pub(crate) min_length: Option<String>,
    #[serde(rename = "@min", default)]
    pub(crate) min: Option<String>,
    #[serde(rename = "@max", default)]
    pub(crate) max: Option<String>,
    #[serde(rename = "@default", default)]
    pub(crate) default: Option<String>,
    #[serde(rename = "@reportable", default)]
    pub(crate) reportable: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EventXml {
    #[serde(rename = "@code")]
    pub(crate) code: String,
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@priority", default)]
    pub(crate) priority: Option<String>,
    #[serde(rename = "@side", default)]
    pub(crate) side: Option<String>,
    #[serde(rename = "@optional", default)]
    pub(crate) optional: Option<String>,
    #[serde(rename = "@isFabricSensitive", default)]
    pub(crate) is_fabric_sensitive: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<EventItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum EventItem {
    #[serde(rename = "description")]
    Description(SimpleTextXml),
    #[serde(rename = "access")]
    Access(AccessXml),
    #[serde(rename = "field")]
    Field(Box<FieldXml>),
    #[serde(rename = "mandatoryConform")]
    MandatoryConform(ConformXml),
    #[serde(rename = "optionalConform")]
    OptionalConform(ConformXml),
    #[serde(rename = "otherwiseConform")]
    OtherwiseConform(OtherwiseConformXml),
    #[serde(rename = "disallowConform")]
    DisallowConform(EmptyConformXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FieldXml {
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@type", default)]
    pub(crate) type_attr: Option<String>,
    #[serde(rename = "@mask", default)]
    pub(crate) mask: Option<String>,
    #[serde(rename = "@id", default)]
    pub(crate) id: Option<String>,
    #[serde(rename = "@fieldId", default)]
    pub(crate) field_id: Option<String>,
    #[serde(rename = "@optional", default)]
    pub(crate) optional: Option<String>,
    #[serde(rename = "@array", default)]
    pub(crate) array: Option<String>,
    #[serde(rename = "@isNullable", default)]
    pub(crate) is_nullable: Option<String>,
    #[serde(rename = "@entryType", default)]
    pub(crate) entry_type: Option<String>,
    #[serde(rename = "@length", default)]
    pub(crate) length: Option<String>,
    #[serde(rename = "@minLength", default)]
    pub(crate) min_length: Option<String>,
    #[serde(rename = "@min", default)]
    pub(crate) min: Option<String>,
    #[serde(rename = "@max", default)]
    pub(crate) max: Option<String>,
    #[serde(rename = "@default", default)]
    pub(crate) default: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EnumXml {
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@type")]
    pub(crate) type_attr: String,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<EnumItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum EnumItem {
    #[serde(rename = "description")]
    Description(SimpleTextXml),
    #[serde(rename = "cluster")]
    Cluster(ClusterRefXml),
    #[serde(rename = "item")]
    Item(Box<ItemXml>),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ItemXml {
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@value", default)]
    pub(crate) value: Option<String>,
    #[serde(rename = "@code", default)]
    pub(crate) code: Option<String>,
    #[serde(rename = "@fieldId", default)]
    pub(crate) field_id: Option<String>,
    #[serde(rename = "@type", default)]
    pub(crate) type_attr: Option<String>,
    #[serde(rename = "@isNullable", default)]
    pub(crate) is_nullable: Option<String>,
    #[serde(rename = "@isFabricSensitive", default)]
    pub(crate) is_fabric_sensitive: Option<String>,
    #[serde(rename = "@optional", default)]
    pub(crate) optional: Option<String>,
    #[serde(rename = "@writable", default)]
    pub(crate) writable: Option<String>,
    #[serde(rename = "@array", default)]
    pub(crate) array: Option<String>,
    #[serde(rename = "@entryType", default)]
    pub(crate) entry_type: Option<String>,
    #[serde(rename = "@length", default)]
    pub(crate) length: Option<String>,
    #[serde(rename = "@minLength", default)]
    pub(crate) min_length: Option<String>,
    #[serde(rename = "@min", default)]
    pub(crate) min: Option<String>,
    #[serde(rename = "@max", default)]
    pub(crate) max: Option<String>,
    #[serde(rename = "@default", default)]
    pub(crate) default: Option<String>,
    #[serde(rename = "@defaut", default)]
    pub(crate) defaut_typo: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<ItemChildItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum ItemChildItem {
    #[serde(rename = "mandatoryConform")]
    MandatoryConform(ConformXml),
    #[serde(rename = "optionalConform")]
    OptionalConform(ConformXml),
    #[serde(rename = "otherwiseConform")]
    OtherwiseConform(OtherwiseConformXml),
    #[serde(rename = "disallowConform")]
    DisallowConform(EmptyConformXml),
    #[serde(rename = "deprecateConform")]
    DeprecateConform(EmptyConformXml),
    #[serde(rename = "provisionalConform")]
    ProvisionalConform(EmptyConformXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClusterRefXml {
    #[serde(rename = "@code")]
    pub(crate) code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BitmapXml {
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@type")]
    pub(crate) type_attr: String,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<BitmapItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum BitmapItem {
    #[serde(rename = "description")]
    Description(SimpleTextXml),
    #[serde(rename = "cluster")]
    Cluster(ClusterRefXml),
    #[serde(rename = "field")]
    Field(Box<FieldXml>),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructXml {
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@isFabricScoped", default)]
    pub(crate) is_fabric_scoped: Option<String>,
    #[serde(rename = "@apiMaturity", default)]
    pub(crate) api_maturity: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<StructItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum StructItem {
    #[serde(rename = "description")]
    Description(SimpleTextXml),
    #[serde(rename = "cluster")]
    Cluster(ClusterRefXml),
    #[serde(rename = "item")]
    Item(Box<ItemXml>),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AccessControlXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<AccessControlItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum AccessControlItem {
    #[serde(rename = "modifier")]
    Modifier(NamedTypeXml),
    #[serde(rename = "operation")]
    Operation(NamedTypeXml),
    #[serde(rename = "role")]
    Role(NamedTypeXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NamedTypeXml {
    #[serde(rename = "@type")]
    pub(crate) type_attr: String,
    #[serde(rename = "@description", default)]
    pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AtomicXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<AtomicItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum AtomicItem {
    #[serde(rename = "type")]
    Type(AtomicTypeXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AtomicTypeXml {
    #[serde(rename = "@id")]
    pub(crate) id: String,
    #[serde(rename = "@name")]
    pub(crate) name: String,
    #[serde(rename = "@description", default)]
    pub(crate) description: Option<String>,
    #[serde(rename = "@size", default)]
    pub(crate) size: Option<String>,
    #[serde(rename = "@discrete", default)]
    pub(crate) discrete: Option<String>,
    #[serde(rename = "@analog", default)]
    pub(crate) analog: Option<String>,
    #[serde(rename = "@signed", default)]
    pub(crate) signed: Option<String>,
    #[serde(rename = "@composite", default)]
    pub(crate) composite: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClusterExtensionXml {
    #[serde(rename = "@code")]
    pub(crate) code: String,
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<ClusterExtensionItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum ClusterExtensionItem {
    #[serde(rename = "attribute")]
    Attribute(AttributeXml),
    #[serde(rename = "command")]
    Command(CommandXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GlobalXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<GlobalItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum GlobalItem {
    #[serde(rename = "attribute")]
    Attribute(Box<AttributeXml>),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeviceTypeXml {
    #[serde(rename = "@id", default)]
    pub(crate) id_attr: Option<String>,
    #[serde(rename = "@name", default)]
    pub(crate) name_attr: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<DeviceTypeItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum DeviceTypeItem {
    #[serde(rename = "name")]
    Name(SimpleTextXml),
    #[serde(rename = "domain")]
    Domain(SimpleTextXml),
    #[serde(rename = "typeName")]
    TypeName(SimpleTextXml),
    #[serde(rename = "profileId")]
    ProfileId(EditableTextXml),
    #[serde(rename = "deviceId")]
    DeviceId(EditableTextXml),
    #[serde(rename = "class")]
    Class(SimpleTextXml),
    #[serde(rename = "scope")]
    Scope(SimpleTextXml),
    #[serde(rename = "superset")]
    Superset(SimpleTextXml),
    #[serde(rename = "revision")]
    Revision(RevisionXml),
    #[serde(rename = "clusters")]
    Clusters(DeviceTypeClustersXml),
    #[serde(rename = "endpointComposition")]
    EndpointComposition(EndpointCompositionXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EditableTextXml {
    #[serde(rename = "@editable", default)]
    pub(crate) editable: Option<String>,
    #[serde(rename = "$text", default)]
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RevisionXml {
    #[serde(rename = "@editable", default)]
    pub(crate) editable: Option<String>,
    #[serde(rename = "@value", default)]
    pub(crate) value: Option<String>,
    #[serde(rename = "$text", default)]
    pub(crate) text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeviceTypeClustersXml {
    #[serde(rename = "@lockOthers", default)]
    pub(crate) lock_others: Option<String>,
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<DeviceTypeClustersItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum DeviceTypeClustersItem {
    #[serde(rename = "include")]
    Include(DeviceTypeIncludeXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeviceTypeIncludeXml {
    #[serde(rename = "@cluster")]
    pub(crate) cluster: String,
    #[serde(rename = "@client", default)]
    pub(crate) client: Option<String>,
    #[serde(rename = "@server", default)]
    pub(crate) server: Option<String>,
    #[serde(rename = "@clientLocked", default)]
    pub(crate) client_locked: Option<String>,
    #[serde(rename = "@serverLocked", default)]
    pub(crate) server_locked: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<DeviceTypeIncludeItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum DeviceTypeIncludeItem {
    #[serde(rename = "features")]
    Features(FeaturesXml),
    #[serde(rename = "requireAttribute")]
    RequireAttribute(SimpleTextXml),
    #[serde(rename = "requireCommand")]
    RequireCommand(SimpleTextXml),
    #[serde(rename = "requireEvent")]
    RequireEvent(SimpleTextXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EndpointCompositionXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<EndpointCompositionItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum EndpointCompositionItem {
    #[serde(rename = "compositionType")]
    CompositionType(SimpleTextXml),
    #[serde(rename = "endpoint")]
    Endpoint(EndpointXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EndpointXml {
    #[serde(rename = "@conformance", default)]
    pub(crate) conformance: Option<String>,
    #[serde(rename = "@constraint", default)]
    pub(crate) constraint: Option<String>,
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<EndpointItem>,
}

/// Items that can appear inside an `<endpoint><deviceType>` element.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum ComposedDeviceTypeContentXml {
    #[serde(rename = "clusters")]
    Clusters(DeviceTypeClustersXml),
    #[serde(rename = "$text")]
    Text(String),
}

/// A `<deviceType>` reference inside `<endpointComposition><endpoint>`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ComposedDeviceTypeXml {
    #[serde(rename = "@id", default)]
    pub(crate) id: Option<String>,
    #[serde(rename = "@name", default)]
    pub(crate) name: Option<String>,
    #[serde(rename = "$value", default)]
    pub(crate) content: Vec<ComposedDeviceTypeContentXml>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum EndpointItem {
    #[serde(rename = "deviceType")]
    DeviceType(ComposedDeviceTypeXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AccessXml {
    #[serde(rename = "@op", default)]
    pub(crate) op: Option<String>,
    #[serde(rename = "@role", default)]
    pub(crate) role: Option<String>,
    #[serde(rename = "@privilege", default)]
    pub(crate) privilege: Option<String>,
    #[serde(rename = "@modifier", default)]
    pub(crate) modifier: Option<String>,
    #[serde(rename = "@fabricSensitive", default)]
    pub(crate) fabric_sensitive: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct QualityXml {
    #[serde(rename = "@changeOmitted", default)]
    pub(crate) change_omitted: Option<String>,
    #[serde(rename = "@largeMessage", default)]
    pub(crate) large_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EmptyConformXml;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OtherwiseConformXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<OtherwiseConformItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum OtherwiseConformItem {
    #[serde(rename = "mandatoryConform")]
    MandatoryConform(ConformXml),
    #[serde(rename = "optionalConform")]
    OptionalConform(ConformXml),
    #[serde(rename = "deprecateConform")]
    DeprecateConform(EmptyConformXml),
    #[serde(rename = "disallowConform")]
    DisallowConform(EmptyConformXml),
    #[serde(rename = "describedConform")]
    DescribedConform(EmptyConformXml),
    #[serde(rename = "provisionalConform")]
    ProvisionalConform(EmptyConformXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConformXml {
    #[serde(rename = "@choice", default)]
    pub(crate) choice: Option<String>,
    #[serde(rename = "@min", default)]
    pub(crate) min: Option<String>,
    #[serde(rename = "@max", default)]
    pub(crate) max: Option<String>,
    #[serde(rename = "@more", default)]
    pub(crate) more: Option<String>,

    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<ConformItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum ConformItem {
    #[serde(rename = "feature")]
    Feature(NamedRefXml),
    #[serde(rename = "attribute")]
    Attribute(NamedRefXml),
    #[serde(rename = "command")]
    Command(NamedRefXml),
    #[serde(rename = "condition")]
    Condition(NamedRefXml),
    #[serde(rename = "notTerm")]
    NotTerm(NotTermXml),
    #[serde(rename = "orTerm")]
    OrTerm(OrTermXml),
    #[serde(rename = "andTerm")]
    AndTerm(AndTermXml),
    #[serde(rename = "greaterTerm")]
    GreaterTerm(GreaterTermXml),
    #[serde(rename = "greaterOrEqualTerm")]
    GreaterOrEqualTerm(GreaterOrEqualTermXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NamedRefXml {
    #[serde(rename = "@name")]
    pub(crate) name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NotTermXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<NotTermItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum NotTermItem {
    #[serde(rename = "feature")]
    Feature(NamedRefXml),
    #[serde(rename = "attribute")]
    Attribute(NamedRefXml),
    #[serde(rename = "orTerm")]
    OrTerm(OrTermXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OrTermXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<OrTermItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum OrTermItem {
    #[serde(rename = "feature")]
    Feature(NamedRefXml),
    #[serde(rename = "attribute")]
    Attribute(NamedRefXml),
    #[serde(rename = "command")]
    Command(NamedRefXml),
    #[serde(rename = "andTerm")]
    AndTerm(AndTermXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AndTermXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<AndTermItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum AndTermItem {
    #[serde(rename = "feature")]
    Feature(NamedRefXml),
    #[serde(rename = "attribute")]
    Attribute(NamedRefXml),
    #[serde(rename = "condition")]
    Condition(NamedRefXml),
    #[serde(rename = "notTerm")]
    NotTerm(NotTermXml),
    #[serde(rename = "orTerm")]
    OrTerm(OrTermXml),
    #[serde(rename = "andTerm")]
    AndTerm(AndTermXml),
    #[serde(rename = "greaterOrEqualTerm")]
    GreaterOrEqualTerm(GreaterOrEqualTermXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GreaterTermXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<GreaterTermItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum GreaterTermItem {
    #[serde(rename = "attribute")]
    Attribute(NamedRefXml),
    #[serde(rename = "literal")]
    Literal(LiteralXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GreaterOrEqualTermXml {
    #[serde(rename = "$value", default)]
    pub(crate) items: Vec<GreaterOrEqualTermItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) enum GreaterOrEqualTermItem {
    #[serde(rename = "revision")]
    Revision(RevisionXml),
    #[serde(rename = "$text")]
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LiteralXml {
    #[serde(rename = "@value")]
    pub(crate) value: String,
}

impl ConfiguratorXml {
    pub(crate) fn from_xml(xml: &str) -> Result<Self, quick_xml::DeError> {
        from_str(xml)
    }
}
