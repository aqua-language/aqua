use std::collections::HashSet;
use std::hash::Hash;

use serde::Deserialize;
use serde::Serialize;

use crate::builtins::unchecked_cell::UncheckedCell;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct Set<T: Eq + Hash>(pub(crate) UncheckedCell<HashSet<T>>);

impl<T: Eq + Hash> Default for Set<T> {
    fn default() -> Self {
        Self(UncheckedCell::new(HashSet::new()))
    }
}

impl<T: Eq + Hash> Set<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// # Safety
    ///
    /// Refer to the documentation of `UncheckedCell::as_mut_unchecked`.
    pub unsafe fn insert(&self, value: T)
    where
        T: Clone,
    {
        self.0.as_mut_unchecked().insert(value);
    }

    /// # Safety
    ///
    /// Refer to the documentation of `UncheckedCell::as_mut_unchecked`.
    pub unsafe fn remove(&self, value: impl std::borrow::Borrow<T>)
    where
        T: Clone,
    {
        self.0.as_mut_unchecked().remove(value.borrow());
    }

    pub fn contains(&self, value: impl std::borrow::Borrow<T>) -> bool
    where
        T: Clone,
    {
        self.0.contains(value.borrow())
    }

    pub fn into_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.0.iter().cloned().collect()
    }
}

impl<T: Eq + Hash> From<HashSet<T>> for Set<T> {
    fn from(set: HashSet<T>) -> Self {
        Self(UncheckedCell::new(set))
    }
}
