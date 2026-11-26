//! Declarative impl macros for generated Matter global attribute types (`0xFFF8`–`0xFFFD`).

/// Implements wrapper helpers, wire conversions, trait forwards, and `Display` for a global ID-list attribute struct.
macro_rules! impl_matter_global_id_list_attr {
    ($name:ident, $item_id:ty) => {
        impl $name {
            #[inline]
            #[must_use]
            pub fn new(val: alloc::vec::Vec<$item_id>) -> Self {
                Self(val)
            }

            #[inline]
            #[must_use]
            pub fn raw(&self) -> &[$item_id] {
                &self.0
            }

            #[inline]
            #[must_use]
            pub const fn as_inner(&self) -> &alloc::vec::Vec<$item_id> {
                &self.0
            }

            #[inline]
            #[must_use]
            pub fn into_inner(self) -> alloc::vec::Vec<$item_id> {
                self.0
            }

            #[must_use]
            pub fn into_value(self) -> $crate::model::AttributeValue {
                let items = self
                    .0
                    .into_iter()
                    .map($crate::model::AttributeValue::from)
                    .collect();
                $crate::model::AttributeValue::List(items)
            }

            pub fn try_from_value(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                let $crate::model::AttributeValue::List(items) = val else {
                    return Err($crate::model::MatterStatus::InvalidDataType);
                };
                let res = items
                    .into_iter()
                    .map(<$item_id>::try_from)
                    .collect::<Result<alloc::vec::Vec<_>, _>>()?;
                Ok(Self(res))
            }

            pub fn validate(
                attr: $crate::model::Attribute,
            ) -> Result<$crate::model::Attribute, $crate::error::AttributeError> {
                Self::try_from(attr).map(Into::into)
            }
        }

        impl From<$name> for $crate::model::Attribute {
            #[inline]
            fn from(attr: $name) -> Self {
                Self::new($name::ATTRIBUTE_ID, attr.into_value())
            }
        }

        impl TryFrom<$crate::model::Attribute> for $name {
            type Error = $crate::error::AttributeError;

            fn try_from(attr: $crate::model::Attribute) -> Result<Self, Self::Error> {
                if attr.id != Self::ATTRIBUTE_ID {
                    return Err($crate::error::AttributeError::AttributeIdMismatch {
                        expected: Self::ATTRIBUTE_ID,
                        actual: attr.id,
                    });
                }
                Self::try_from_value(attr.value).map_err(|status| $crate::error::AttributeError::InvalidValue {
                    id: Self::ATTRIBUTE_ID,
                    field_id: None,
                    status,
                })
            }
        }

        impl From<alloc::vec::Vec<$item_id>> for $name {
            #[inline]
            fn from(val: alloc::vec::Vec<$item_id>) -> Self {
                Self(val)
            }
        }

        impl From<$name> for alloc::vec::Vec<$item_id> {
            #[inline]
            fn from(attr: $name) -> Self {
                attr.0
            }
        }

        impl core::ops::Deref for $name {
            type Target = alloc::vec::Vec<$item_id>;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl AsRef<alloc::vec::Vec<$item_id>> for $name {
            #[inline]
            fn as_ref(&self) -> &alloc::vec::Vec<$item_id> {
                &self.0
            }
        }

        impl AsRef<[$item_id]> for $name {
            #[inline]
            fn as_ref(&self) -> &[$item_id] {
                &self.0
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "<list of {} items>", self.0.len())
            }
        }
    };
}

/// Implements wrapper helpers, wire conversions, trait forwards, and `Display` for a global scalar attribute struct.
macro_rules! impl_matter_global_scalar_attr {
    ($name:ident, $inner:ty, $variant:ident, $display_fmt:literal) => {
        impl $name {
            #[inline]
            #[must_use]
            pub const fn new(val: $inner) -> Self {
                Self(val)
            }

            #[inline]
            #[must_use]
            pub const fn raw(self) -> $inner {
                self.0
            }

            #[inline]
            #[must_use]
            pub const fn as_inner(&self) -> &$inner {
                &self.0
            }

            #[inline]
            #[must_use]
            pub const fn into_inner(self) -> $inner {
                self.0
            }

            #[must_use]
            pub fn into_value(self) -> $crate::model::AttributeValue {
                $crate::model::AttributeValue::$variant(self.0)
            }

            pub fn try_from_value(val: $crate::model::AttributeValue) -> Result<Self, $crate::model::MatterStatus> {
                $crate::traits::decode_unsigned::<$inner>(&val).map(Self)
            }

            pub fn validate(
                attr: $crate::model::Attribute,
            ) -> Result<$crate::model::Attribute, $crate::error::AttributeError> {
                Self::try_from(attr).map(Into::into)
            }
        }

        impl From<$name> for $crate::model::Attribute {
            #[inline]
            fn from(attr: $name) -> Self {
                Self::new($name::ATTRIBUTE_ID, attr.into_value())
            }
        }

        impl TryFrom<$crate::model::Attribute> for $name {
            type Error = $crate::error::AttributeError;

            fn try_from(attr: $crate::model::Attribute) -> Result<Self, Self::Error> {
                if attr.id != Self::ATTRIBUTE_ID {
                    return Err($crate::error::AttributeError::AttributeIdMismatch {
                        expected: Self::ATTRIBUTE_ID,
                        actual: attr.id,
                    });
                }
                Self::try_from_value(attr.value).map_err(|status| $crate::error::AttributeError::InvalidValue {
                    id: Self::ATTRIBUTE_ID,
                    field_id: None,
                    status,
                })
            }
        }

        impl From<$inner> for $name {
            #[inline]
            fn from(val: $inner) -> Self {
                Self(val)
            }
        }

        impl From<$name> for $inner {
            #[inline]
            fn from(attr: $name) -> Self {
                attr.0
            }
        }

        impl core::ops::Deref for $name {
            type Target = $inner;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl AsRef<$inner> for $name {
            #[inline]
            fn as_ref(&self) -> &$inner {
                &self.0
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, $display_fmt, self.0)
            }
        }
    };
}

pub(crate) use impl_matter_global_id_list_attr;
pub(crate) use impl_matter_global_scalar_attr;
