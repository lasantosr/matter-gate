//! Zero-allocation, single-pass typed iterator over [`AttributeValue`] slices.

extern crate alloc;

use alloc::vec::Vec;
use core::{
    borrow::Borrow,
    fmt::{Debug, Formatter, Result as FmtResult},
    iter::FusedIterator,
    marker::PhantomData,
    slice::Iter as SliceIter,
};

use crate::{optional::Optional, status::MatterStatus, value::AttributeValue};

/// Zero-allocation, single-pass view and iterator over a slice of [`AttributeValue`] elements
/// decoded on demand into `T`.
///
/// Each element is converted via [`TryFrom<&AttributeValue>`] as it is visited without upfront
/// scanning or heap allocation, yielding [`Result<T, MatterStatus>`].
pub struct ResultListIter<'a, T> {
    inner: SliceIter<'a, AttributeValue>,
    _marker: PhantomData<fn() -> T>,
}

impl<'a, T> ResultListIter<'a, T> {
    /// Construct a typed iterator over a slice of [`AttributeValue`] items.
    #[must_use]
    pub fn new(items: &'a [AttributeValue]) -> Self {
        Self {
            inner: items.iter(),
            _marker: PhantomData,
        }
    }

    /// Return the number of elements remaining in the underlying wire list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Return `true` if no elements remain in the underlying wire list.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.as_slice().is_empty()
    }
}

impl<'a, T> ResultListIter<'a, T>
where
    T: TryFrom<&'a AttributeValue, Error = MatterStatus> + PartialEq,
{
    /// Return `true` if the remaining elements contain `target`, short-circuiting on match or decode error.
    pub fn contains(&self, target: impl Borrow<T>) -> Result<bool, MatterStatus> {
        let target = target.borrow();
        for item in self.inner.as_slice() {
            if &T::try_from(item)? == target {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl<'a, T> ResultListIter<'a, T>
where
    T: TryFrom<&'a AttributeValue, Error = MatterStatus>,
{
    /// Collect the remaining decoded elements (not yet yielded by `next`) into a [`Vec<T>`] in a single pass.
    pub fn to_vec(&self) -> Result<Vec<T>, MatterStatus> {
        let mut out = Vec::with_capacity(self.inner.len());
        for item in self.inner.as_slice() {
            out.push(T::try_from(item)?);
        }
        Ok(out)
    }
}

impl<T> Clone for ResultListIter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for ResultListIter<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ResultListIter").field("inner", &self.inner).finish()
    }
}

impl<'a, T> Iterator for ResultListIter<'a, T>
where
    T: TryFrom<&'a AttributeValue, Error = MatterStatus>,
{
    type Item = Result<T, MatterStatus>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(T::try_from)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for ResultListIter<'a, T> where T: TryFrom<&'a AttributeValue, Error = MatterStatus> {}

impl<'a, T> DoubleEndedIterator for ResultListIter<'a, T>
where
    T: TryFrom<&'a AttributeValue, Error = MatterStatus>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(T::try_from)
    }
}

impl<'a, T> FusedIterator for ResultListIter<'a, T> where T: TryFrom<&'a AttributeValue, Error = MatterStatus> {}

impl<'a, T> Optional<ResultListIter<'a, T>>
where
    T: TryFrom<&'a AttributeValue, Error = MatterStatus> + PartialEq,
{
    /// Return `Ok(true)` if `Present` and the wire list contains `target`, or `Ok(false)` if `Omitted` or absent.
    pub fn contains(&self, target: impl Borrow<T>) -> Result<bool, MatterStatus> {
        match self {
            Self::Omitted => Ok(false),
            Self::Present(iter) => iter.contains(target),
        }
    }
}

impl<'a, T> Optional<ResultListIter<'a, T>>
where
    T: TryFrom<&'a AttributeValue, Error = MatterStatus>,
{
    /// Collect a `Present` list into [`Optional::Present(Vec<T>)`](Optional::Present), preserving [`Optional::Omitted`].
    pub fn to_vec(&self) -> Result<Optional<Vec<T>>, MatterStatus> {
        match self {
            Self::Omitted => Ok(Optional::Omitted),
            Self::Present(iter) => iter.to_vec().map(Optional::Present),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeviceTypeEntry, DeviceTypeId, command::CommandId};

    #[test]
    fn result_list_iter_works_for_ids_and_structs() {
        let cmd_items = alloc::vec![AttributeValue::U8(1), AttributeValue::U64(2)];
        let cmd_iter = ResultListIter::<CommandId>::new(&cmd_items);
        assert_eq!(cmd_iter.len(), 2);
        assert!(!cmd_iter.is_empty());
        assert_eq!(cmd_iter.contains(CommandId::new(2)), Ok(true));
        assert_eq!(cmd_iter.contains(CommandId::new(3)), Ok(false));
        assert_eq!(cmd_iter.to_vec(), Ok(alloc::vec![CommandId::new(1), CommandId::new(2)]));

        let dt = DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 2);
        let dt_items = alloc::vec![AttributeValue::from(dt)];
        let dt_iter = ResultListIter::<DeviceTypeEntry>::new(&dt_items);
        assert_eq!(dt_iter.contains(dt), Ok(true));
        assert_eq!(dt_iter.to_vec(), Ok(alloc::vec![dt]));

        let omitted: Optional<ResultListIter<'_, CommandId>> = Optional::Omitted;
        assert_eq!(omitted.contains(CommandId::new(1)), Ok(false));
        assert_eq!(omitted.to_vec(), Ok(Optional::Omitted));

        let present = Optional::Present(cmd_iter);
        assert_eq!(present.contains(CommandId::new(2)), Ok(true));
        assert_eq!(
            present.to_vec(),
            Ok(Optional::Present(alloc::vec![CommandId::new(1), CommandId::new(2)]))
        );
    }
}
