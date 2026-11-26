//! Zero-allocation, non-mutating static schema validation engine.
//!
//! Operates directly over immutable `'static` descriptor tables (`SpecRegistry`)
//! and borrowed [`matter_data_model`] wire types without heap allocation, cloning,
//! or mutating valid payloads.

mod access;
mod conformance;
mod descriptor_checks;
mod descriptors;
mod registry;
mod value;

#[cfg(test)]
mod tests;

pub use access::AccessFlags;
#[cfg(any(test, all(feature = "dispatch", feature = "schemars")))]
pub(crate) use conformance::eval_conformance;
#[cfg(any(test, all(feature = "dispatch", feature = "schemars")))]
pub(crate) use descriptors::ConformanceResult;
pub use descriptors::{
    AttributeDescriptor, ClusterDescriptor, CommandDescriptor, DeviceTypeDescriptor, EventDescriptor,
};
pub(crate) use descriptors::{
    ChoiceGroup, ConformanceOp, DeviceClusterReq, DeviceTypeIncludeItem, FieldDescriptor, ListLength, StructDescriptor,
    ValueBounds, ValueConstraint, ValueType,
};
pub use registry::{RegistryValidator, SpecRegistry};
#[cfg(test)]
pub(crate) use value::validate_value;

/// Maximum number of standard attributes or struct fields tracked via stack bitset.
const MAX_STACK_MEMBERS: usize = 256;
/// Number of `u64` words in the stack bitset (`256 / 64 = 4`).
const BITSET_WORDS: usize = 4;

/// Validation strictness mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationMode {
    /// Validate only the members present in a partial/delta payload.
    Partial,
    /// Validate a complete cluster or endpoint snapshot, including mandatory members
    /// and global lists.
    FullSnapshot {
        /// When `true`, provisional (`P`) members are treated as disallowed.
        enforce_provisional: bool,
    },
}

impl Default for ValidationMode {
    fn default() -> Self {
        Self::FullSnapshot {
            enforce_provisional: false,
        }
    }
}

/// Integer width-matching policy for [`RegistryValidator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IntegerMode {
    /// Require the exact [`AttributeValue`](crate::model::AttributeValue) integer variant matching the schema width
    /// (e.g., `U32` only).
    Strict,
    /// Allow unsigned/signed widening (`wire_bits <= target_bits`, per Matter TLV minimal-width encoding),
    /// rejecting narrowing with [`MatterStatus::InvalidDataType`](crate::model::MatterStatus::InvalidDataType).
    #[default]
    Widening,
    /// Allow any same-signedness integer whose numeric value fits in the target width (`ConstraintError` on
    /// overflow, `InvalidDataType` on wrong signedness).
    Fit,
}

/// Fixed 256-bit stack-allocated bitset for zero-alloc O(1) duplicate attribute and field tracking.
///
/// Covers descriptor indices `0..256` without heap allocation; callers fall back to a linear slice scan
/// for any member beyond `MAX_STACK_MEMBERS`.
pub(super) struct StackBitset {
    words: [u64; BITSET_WORDS],
}

impl StackBitset {
    /// Creates an empty bitset with all 256 bits cleared.
    pub(super) const fn new() -> Self {
        Self {
            words: [0; BITSET_WORDS],
        }
    }

    /// Marks `idx` as seen and returns `true` if it was not previously set (`false` on duplicate or out-of-range).
    pub(super) fn insert(&mut self, idx: usize) -> bool {
        let word = idx >> 6;
        let bit = 1_u64 << (idx & 63);
        let Some(slot) = self.words.get_mut(word) else {
            return false;
        };
        let was_set = (*slot & bit) != 0;
        *slot |= bit;
        !was_set
    }
}
