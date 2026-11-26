//! Internal declarative macros for `matter-spec` generated models.
//!
//! These macros collapse repetitive trait implementations, bitmap/enum boilerplate,
//! and wire/dispatch conversions across generated version modules (`v1_0`..`v1_6`)
//! to keep the crate compact and fast to parse.

mod dispatch;
mod endpoints;
mod globals;
mod members;
mod primitives;

pub(crate) use dispatch::{impl_cluster_enum_dispatch, impl_matter_describe, impl_select_described};
pub(crate) use endpoints::impl_device_type_spec;
pub(crate) use globals::{impl_matter_global_id_list_attr, impl_matter_global_scalar_attr};
pub(crate) use members::{
    impl_cluster_member_variant, impl_cluster_wire_conversions, impl_matter_attribute, impl_matter_command,
    impl_matter_event, impl_matter_struct_wire,
};
pub(crate) use primitives::{define_matter_bitmap, define_matter_enum, define_matter_semantic_bool};
