use anyhow::{Context as _, Result};
use genco::lang::rust::Tokens;

mod dispatch;
mod lib_rs;
mod manifest;
mod version;

pub(super) use dispatch::{
    generate_version_dispatch_attribute_rs, generate_version_dispatch_cluster_rs, generate_version_dispatch_command_rs,
    generate_version_dispatch_endpoint_rs, generate_version_dispatch_event_rs, generate_version_dispatch_mod_rs,
};
pub(super) use lib_rs::generate_top_lib_rs;
pub(super) use manifest::{generate_spec_build_rs, update_spec_cargo_toml};
pub(super) use version::{generate_endpoints_mod_rs, generate_version_clusters_mod, generate_version_mod};

#[derive(Debug, Clone)]
pub(super) struct DispatchClusterEntry {
    pub(super) cluster_id: u32,
    pub(super) module_name: String,
    pub(super) name: String,
    pub(super) struct_name: String,
    pub(super) cfg: String,
    pub(super) has_attributes: bool,
    pub(super) has_client_commands: bool,
    pub(super) has_server_commands: bool,
    pub(super) has_events: bool,
    pub(super) is_large_state: bool,
    pub(super) is_large_patch: bool,
}

#[derive(Debug, Clone)]
pub(super) struct DispatchEndpointEntry {
    pub(super) device_id: u32,
    pub(super) module_name: String,
    pub(super) name: String,
    pub(super) feature: String,
    pub(super) is_empty: bool,
    pub(super) is_large: bool,
}

fn render(tokens: Tokens) -> Result<String> {
    tokens.to_file_string().context("Failed to render Rust tokens")
}
