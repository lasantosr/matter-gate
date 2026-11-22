use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

use crate::error::Error;

/// Zero-config domain product family derived directly from XML `<domain>`.
#[derive(Debug, Clone)]
pub struct Category(String);

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Category {}

impl PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Category {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for Category {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Category {
    pub const GENERAL: Self = Self(String::new());

    /// Returns the canonical general category (`"general"`).
    #[must_use]
    pub fn general() -> Self {
        Self("general".to_owned())
    }

    #[cfg(test)]
    fn new_general_slug() -> Self {
        Self::try_from("general").expect("general")
    }

    /// Returns the opt-in testing category (`"testing"`).
    #[must_use]
    pub fn testing() -> Self {
        Self("testing".to_owned())
    }

    /// Normalizes a raw XML `<domain>` string into a known category slug.
    ///
    /// - `"CHIP"`, `"General"`, or empty maps to `"general"`.
    /// - Fails fast with [`Error::UnknownCategory`] on unrecognized domains.
    pub fn from_xml_domain(raw: &str) -> Result<Self, Error> {
        let trimmed = raw.trim();
        let slug = match trimmed.to_ascii_lowercase().as_str() {
            "" | "chip" | "general" => "general",
            "appliances" => "appliances",
            "cameras" => "cameras",
            "closures" => "closures",
            "energy management" => "energy-management",
            "ha" => "ha",
            "hvac" => "hvac",
            "lighting" => "lighting",
            "measurement & sensing" | "measurement &amp; sensing" => "measurement-and-sensing",
            "media" => "media",
            "network infrastructure" => "network-infrastructure",
            "robots" => "robots",
            "xchip" => "xchip",
            _ => {
                return Err(Error::UnknownCategory {
                    category: trimmed.to_owned(),
                });
            }
        };
        Ok(Self(slug.to_owned()))
    }

    /// Returns the kebab-case identifier matching spec feature flags (e.g. `energy-management`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        if self.0.is_empty() { "general" } else { &self.0 }
    }

    /// Returns the snake-case module name for Rust source files (e.g. `energy_management`).
    #[must_use]
    pub fn to_module_name(&self) -> String {
        self.as_str().replace('-', "_")
    }

    /// Returns the feature name matching spec feature flags (identical to `as_str`).
    #[must_use]
    pub fn to_feature_name(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for Category {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_xml_domain(value)
    }
}

impl TryFrom<String> for Category {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_xml_domain(&value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use super::*;

    #[test]
    fn general_constant_equals_general_constructors() {
        assert_eq!(Category::GENERAL, Category::general());
        assert_eq!(Category::GENERAL, Category::new_general_slug());
        let btree: BTreeSet<_> = [Category::GENERAL, Category::general()].into_iter().collect();
        assert_eq!(btree.len(), 1);
        let hash: HashSet<_> = [Category::GENERAL, Category::general()].into_iter().collect();
        assert_eq!(hash.len(), 1);
    }

    #[test]
    fn zero_config_xml_domain_slugification() {
        assert_eq!(Category::from_xml_domain("Appliances").unwrap().as_str(), "appliances");
        assert_eq!(
            Category::from_xml_domain("Appliances").unwrap().to_module_name(),
            "appliances"
        );

        assert_eq!(
            Category::from_xml_domain("Measurement & Sensing").unwrap().as_str(),
            "measurement-and-sensing"
        );
        assert_eq!(
            Category::from_xml_domain("Measurement & Sensing")
                .unwrap()
                .to_module_name(),
            "measurement_and_sensing"
        );

        assert_eq!(
            Category::from_xml_domain("Energy Management").unwrap().as_str(),
            "energy-management"
        );
        assert_eq!(
            Category::from_xml_domain("Energy Management").unwrap().to_module_name(),
            "energy_management"
        );

        assert_eq!(
            Category::from_xml_domain("Network Infrastructure").unwrap().as_str(),
            "network-infrastructure"
        );
        assert_eq!(
            Category::from_xml_domain("Network Infrastructure")
                .unwrap()
                .to_module_name(),
            "network_infrastructure"
        );

        assert_eq!(Category::from_xml_domain("CHIP").unwrap().as_str(), "general");
        assert_eq!(Category::from_xml_domain("chip").unwrap().to_module_name(), "general");
        assert_eq!(Category::from_xml_domain("General").unwrap().as_str(), "general");
        assert_eq!(Category::from_xml_domain("").unwrap().as_str(), "general");
        assert_eq!(Category::from_xml_domain("  ").unwrap().to_module_name(), "general");

        assert_eq!(Category::from_xml_domain("HVAC").unwrap().as_str(), "hvac");
        assert_eq!(Category::from_xml_domain("Lighting").unwrap().as_str(), "lighting");
        assert_eq!(Category::from_xml_domain("Media").unwrap().as_str(), "media");
        assert_eq!(Category::from_xml_domain("Closures").unwrap().as_str(), "closures");
        assert_eq!(Category::from_xml_domain("Robots").unwrap().as_str(), "robots");
        assert_eq!(Category::from_xml_domain("Cameras").unwrap().as_str(), "cameras");
    }

    #[test]
    fn unknown_xml_domain_fails_fast() {
        let err = Category::from_xml_domain("Unknown Fictional Domain").unwrap_err();
        assert!(matches!(err, Error::UnknownCategory { .. }));
    }
}
