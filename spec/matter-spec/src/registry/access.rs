use core::{
    fmt,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

/// Matter Access Control privilege level required to perform an interaction.
///
/// Discriminants match the normative Matter Core Specification `AccessControlEntryPrivilegeEnum`
/// (`View = 1 < Operate = 3 < Manage = 4 < Administer = 5`; `2` is `ProxyView`), so callers can
/// compare privileges directly with `>=`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum AccessPrivilege {
    /// Read-only visibility (`1`).
    View = 1,
    /// Standard operational control (`3`).
    Operate = 3,
    /// Configuration and management (`4`).
    Manage = 4,
    /// Administrative / commissioning control (`5`).
    Administer = 5,
}

impl AccessPrivilege {
    /// Returns the normative Matter `AccessControlEntryPrivilegeEnum` discriminant (`1..=5`).
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::View => 1,
            Self::Operate => 3,
            Self::Manage => 4,
            Self::Administer => 5,
        }
    }
}

impl From<AccessPrivilege> for u8 {
    #[inline]
    fn from(privilege: AccessPrivilege) -> Self {
        privilege.raw()
    }
}

/// Access control flags for cluster attributes, commands, and events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct AccessFlags(pub(crate) u8);

impl AccessFlags {
    /// Empty set of access flags (`0`).
    pub const EMPTY: Self = Self(0);
    /// Fabric-scoped member (includes list/struct attributes whose element type is fabric-scoped).
    pub const FABRIC_SCOPED: Self = Self(1 << 2);
    /// Fabric-sensitive member.
    pub const FABRIC_SENSITIVE: Self = Self(1 << 3);
    /// Invokable `ClientToServer` command.
    pub const INVOKE: Self = Self(1 << 6);
    /// Readable attribute or event.
    pub const READ: Self = Self(1 << 0);
    /// Scene-capable attribute (reserved; upstream ZAP XML does not currently emit scene quality).
    pub const SCENE: Self = Self(1 << 5);
    /// Requires timed interaction for writes/invocations.
    pub const TIMED: Self = Self(1 << 4);
    /// Writable attribute.
    pub const WRITE: Self = Self(1 << 1);

    /// Returns an empty set of access flags.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns the raw bitflags representation.
    #[inline]
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Constructs `AccessFlags` from raw bits, truncating any unknown bits.
    #[inline]
    #[must_use]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        let valid = Self::READ.0
            | Self::WRITE.0
            | Self::FABRIC_SCOPED.0
            | Self::FABRIC_SENSITIVE.0
            | Self::TIMED.0
            | Self::SCENE.0
            | Self::INVOKE.0;
        Self(bits & valid)
    }

    /// Returns `true` if no flags are set (`0`).
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all bits in `other` are set in `self`.
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Inserts all flags in `other` into `self`.
    #[inline]
    pub const fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Removes all flags in `other` from `self`.
    #[inline]
    pub const fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    /// Returns the bitwise union (`self | other`) of two flag sets.
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns the bitwise intersection (`self & other`) of two flag sets.
    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl fmt::Display for AccessFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X}", self.0)
    }
}

impl BitOr for AccessFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for AccessFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for AccessFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for AccessFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitXor for AccessFlags {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for AccessFlags {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for AccessFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Self::from_bits_truncate(!self.0)
    }
}

impl From<AccessFlags> for u8 {
    #[inline]
    fn from(flags: AccessFlags) -> Self {
        flags.0
    }
}

/// Matter specification quality and transport hints for cluster attributes and commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct QualityFlags(pub(crate) u8);

impl QualityFlags {
    /// Attribute requires an atomic write transaction (`mustUseAtomicWrite="true"`, CSA quality `A`).
    pub const ATOMIC_WRITE: Self = Self(1 << 2);
    /// Attribute changes are omitted from subscription reports (`changeOmitted="true"`, CSA quality `C`).
    pub const CHANGE_OMITTED: Self = Self(1 << 0);
    /// Empty set of quality flags (`0`).
    pub const EMPTY: Self = Self(0);
    /// Member requires Large Message / TCP transport (`largeMessage="true"`, CSA quality `L`).
    pub const LARGE_MESSAGE: Self = Self(1 << 1);
    /// Upstream ZAP reporting hint (`reportable="true"`).
    ///
    /// Note that in the Matter specification all attributes are reportable unless [`Self::CHANGE_OMITTED`] is set.
    pub const REPORTABLE: Self = Self(1 << 3);

    /// Returns an empty set of quality flags.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns the raw bitflags representation.
    #[inline]
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Constructs `QualityFlags` from raw bits, truncating any unknown bits.
    #[inline]
    #[must_use]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        let valid = Self::CHANGE_OMITTED.0 | Self::LARGE_MESSAGE.0 | Self::ATOMIC_WRITE.0 | Self::REPORTABLE.0;
        Self(bits & valid)
    }

    /// Returns `true` if no flags are set (`0`).
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all bits in `other` are set in `self`.
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Inserts all flags in `other` into `self`.
    #[inline]
    pub const fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Removes all flags in `other` from `self`.
    #[inline]
    pub const fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    /// Returns the bitwise union (`self | other`) of two flag sets.
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns the bitwise intersection (`self & other`) of two flag sets.
    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl fmt::Display for QualityFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X}", self.0)
    }
}

impl BitOr for QualityFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for QualityFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for QualityFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for QualityFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitXor for QualityFlags {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for QualityFlags {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for QualityFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Self::from_bits_truncate(!self.0)
    }
}

impl From<QualityFlags> for u8 {
    #[inline]
    fn from(flags: QualityFlags) -> Self {
        flags.0
    }
}
