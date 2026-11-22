//! **From official CSA Matter XML to strongly typed Rust, purely in memory.**
//!
//! `matter-spec-gen` is the internal AST compiler and code generator behind `matter-spec`. It transforms raw CSA
//! cluster and device-type XML schemas into compile-time-checked Rust models.
//!
//! # Overview
//!
//! `matter-spec-gen` parses Connectivity Standards Alliance (CSA) cluster and device-type XML definitions and
//! compiles them into strongly typed Rust token streams and source strings targeting `matter-data-model`. Because it
//! performs zero I/O and makes no assumptions about directory layout, orchestrators such as `xtask` stay in full
//! control of where schemas come from and how generated code is written.
//!
//! # Features
//!
//! - **Pure in-memory.** Zero filesystem, network, git, or CLI dependencies. Pass XML documents as in-memory strings
//!   ([`SpecInput`], [`XmlDocument`]) to [`build`] and receive structured [`Spec`] and [`VersionSpec`] items back.
//! - **Fail-fast validation.** Unmapped scalar types, invalid boolean semantics, unknown command sources, or
//!   unrecognized event priorities surface immediately as typed [`Error`]s instead of silent fallbacks.
//! - **Deterministic rendering.** Every generated item ([`ClusterItem`], [`DeviceTypeItem`], [`GlobalsItem`],
//!   [`GeneratedItem`]) implements [`Render`], emitting `genco` Rust token trees or formatted source text with
//!   byte-identical output for identical inputs.
//! - **Layout agnostic.** Decides nothing about files, directories, `mod.rs` trees, or `Cargo.toml` feature tables.
//!   Your tooling owns the output structure.
//!
//! # Cargo Features
//!
//! This crate has no optional feature flags.
//!
//! # Examples
//!
//! ```rust,no_run
//! use matter_spec_gen::{Render, SpecInput, XmlDocument, build};
//!
//! # let (version_label, release_tag) = ("1.4", "v1.4.0.0");
//! # let (chip_types_xml, global_attrs_xml, on_off_cluster_xml) = ("", "", "");
//! let docs = vec![
//!     XmlDocument::new("chip-types.xml", chip_types_xml),
//!     XmlDocument::new("global-attributes.xml", global_attrs_xml),
//!     XmlDocument::new("on-off.xml", on_off_cluster_xml),
//! ];
//!
//! // Pass a version label (for naming) and the upstream release tag the XML comes from.
//! let input = SpecInput::new(version_label, release_tag, docs);
//! let spec = build([input]).expect("failed to build spec models");
//!
//! for version in &spec.versions {
//!     for cluster in &version.clusters {
//!         println!("Cluster: {} ({:?})", cluster.name(), cluster.category());
//!         let rust_code = cluster.render().expect("failed to render cluster");
//!     }
//! }
//! ```

pub mod category;
pub mod error;
pub mod items;
pub mod render;

pub(crate) mod build;
pub(crate) mod model;
pub(crate) mod reconcile;
pub(crate) mod shared;
pub(crate) mod xml;

pub use build::{Spec, SpecInput, VersionSpec, XmlDocument, build};
pub use category::Category;
pub use error::Error;
pub use items::{ClusterItem, DeviceTypeItem, GeneratedItem, GlobalsItem};
pub use reconcile::reconcile_categories_across_versions;
pub use render::Render;
pub use shared::{SharedBucket, SharedKind, SharedPlan};
