use serde::Deserialize;

use super::{
    AccessXml, ClientServerXml, ConformXml, DomainXml, EmptyConformXml, GreaterOrEqualTermXml, OtherwiseConformXml,
    QualityXml, SimpleTextXml,
};

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
