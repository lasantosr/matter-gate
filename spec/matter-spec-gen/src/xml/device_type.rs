use serde::Deserialize;

use super::{FeaturesXml, SimpleTextXml};

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
