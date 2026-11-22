//! Ergonomic extensions for Matter Mode Base clusters.

use alloc::{format, string::String, vec, vec::Vec};

use crate::{
    model::MatterStatus,
    traits::{ModeBaseCluster, ModeOptionBuilder},
};

/// Ergonomic extension trait for clusters implementing [`ModeBaseCluster`].
pub trait ModeClusterExt: ModeBaseCluster {
    /// Construct a mode cluster instance directly from an iterator of mode tags and the current active tag.
    ///
    /// # Errors
    ///
    /// Returns [`MatterStatus::ConstraintError`] if more than
    /// 256 mode options would be created (since Matter mode IDs are `u8`).
    fn from_tags(tags: impl IntoIterator<Item = Self::ModeTag>, current: Self::ModeTag) -> Result<Self, MatterStatus> {
        let mut options = Vec::new();
        let mut current_mode = None;
        for (idx, tag) in tags.into_iter().enumerate() {
            let mode = u8::try_from(idx).map_err(|_| MatterStatus::ConstraintError)?;
            if tag == current && current_mode.is_none() {
                current_mode = Some(mode);
            }
            let label = format!("{tag:?}");
            options.push(Self::ModeOptionStruct::build(label, mode, vec![tag.into()]));
        }
        let current_mode = match current_mode {
            Some(mode) => mode,
            None if !options.is_empty() => {
                let mode = u8::try_from(options.len()).map_err(|_| MatterStatus::ConstraintError)?;
                let label = format!("{current:?}");
                options.push(Self::ModeOptionStruct::build(label, mode, vec![current.into()]));
                mode
            }
            None => 0,
        };
        Ok(Self::new(options, current_mode))
    }
}

impl<C: ModeBaseCluster> ModeClusterExt for C {}

/// Ergonomic extension trait for mode option builders.
pub trait ModeOptionExt<TTag>: ModeOptionBuilder<TTag> + Sized {
    /// Create a new mode option with a single tag.
    #[must_use]
    fn with_tag(label: impl Into<String>, mode: u8, tag: TTag) -> Self {
        Self::build(label.into(), mode, vec![tag])
    }
}

impl<TTag, B: ModeOptionBuilder<TTag>> ModeOptionExt<TTag> for B {}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned as _;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MockTag {
        Auto,
        Eco,
    }

    impl From<MockTag> for u16 {
        fn from(tag: MockTag) -> Self {
            match tag {
                MockTag::Auto => 1,
                MockTag::Eco => 2,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct MockOption {
        label: String,
        mode: u8,
        tags: Vec<u16>,
    }

    impl ModeOptionBuilder<u16> for MockOption {
        fn build(label: String, mode: u8, mode_tags: Vec<u16>) -> Self {
            Self {
                label,
                mode,
                tags: mode_tags,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct MockCluster {
        options: Vec<MockOption>,
        current_mode: u8,
    }

    impl ModeBaseCluster for MockCluster {
        type ModeOptionStruct = MockOption;
        type ModeTag = MockTag;
        type ModeTagStruct = u16;

        fn new(supported_modes: Vec<Self::ModeOptionStruct>, current_mode: u8) -> Self {
            Self {
                options: supported_modes,
                current_mode,
            }
        }
    }

    #[test]
    fn mode_cluster_and_option_ext_builders() {
        let cluster = MockCluster::from_tags([MockTag::Auto, MockTag::Eco], MockTag::Eco).unwrap();
        assert_eq!(cluster.current_mode, 1);
        assert_eq!(cluster.options.len(), 2);
        assert_eq!(
            cluster.options.first().map(|o| o.tags.as_slice()),
            Some([1_u16].as_slice())
        );

        let opt = MockOption::with_tag("Boost", 2, 3);
        assert_eq!(opt.tags, vec![3]);

        let opt_owned = MockOption::with_tag("Boost".to_owned(), 2, 3);
        assert_eq!(opt_owned.tags, vec![3]);

        // Missing current tag is appended when tags is non-empty
        let appended = MockCluster::from_tags([MockTag::Auto], MockTag::Eco).unwrap();
        assert_eq!(appended.options.len(), 2);
        assert_eq!(appended.current_mode, 1);
        assert_eq!(
            appended.options.get(1).map(|o| o.tags.as_slice()),
            Some([2_u16].as_slice())
        );

        // Empty tags stays empty with current_mode 0
        let empty = MockCluster::from_tags([], MockTag::Eco).unwrap();
        assert_eq!(empty.options.len(), 0);
        assert_eq!(empty.current_mode, 0);
    }
}
