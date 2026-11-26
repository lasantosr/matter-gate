use serde::Deserialize;

use super::RevisionXml;

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
