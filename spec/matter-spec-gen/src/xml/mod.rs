//! ZAP XML schema models corresponding to the official upstream `project-chip/alchemy` definitions.
//!
//! Upstream Repository: <https://github.com/project-chip/alchemy>
//! Commit: `fe4593a2f200fd861ebb3fc6e00dbbfc8e3c66ea`.
//!
//! All structs in this module enforce strict, exhaustive deserialization via
//! `#[serde(deny_unknown_fields)]` and exhaustive `$value` enums to ensure no unknown XML tags or
//! attributes are ignored.

mod cluster;
mod configurator;
mod conformance;
mod device_type;

#[cfg(test)]
mod tests;

pub(crate) use cluster::*;
pub(crate) use configurator::*;
pub(crate) use conformance::*;
pub(crate) use device_type::*;
