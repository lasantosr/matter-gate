use quick_xml::de::from_str;
use serde::Deserialize;

use super::{ClusterXml, DeviceTypeXml, EnumXml, StructXml, cluster::BitmapXml};

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

impl ConfiguratorXml {
    pub(crate) fn from_xml(xml: &str) -> Result<Self, quick_xml::DeError> {
        from_str(xml)
    }
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
    Attribute(super::AttributeXml),
    #[serde(rename = "command")]
    Command(super::CommandXml),
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
    Attribute(Box<super::AttributeXml>),
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
