use std::collections::HashSet;
use std::hash::Hash;

use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;

use super::cell::Cell;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct Bag<T: Eq + Hash>(pub Cell<HashSet<T>>);

impl<T: Eq + Hash + DeepClone> DeepClone for Bag<T> {
    fn deep_clone(&self) -> Self {
        let bag = self.0.as_ref().iter().map(|x| x.deep_clone()).collect();
        Bag(Cell::new(bag))
    }
}

impl<T: Eq + Hash + std::fmt::Display> std::fmt::Display for Bag<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{")?;
        let x = self.0.as_ref();
        let mut iter = x.iter();
        if let Some(x) = iter.next() {
            write!(f, "{}", x)?;
            for x in iter {
                write!(f, ", {}", x)?;
            }
        }
        write!(f, "}}")
    }
}

impl<T: Eq + Hash> Default for Bag<T> {
    fn default() -> Self {
        Self(Cell::new(HashSet::new()))
    }
}

impl<T: Eq + Hash> Bag<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// # Safety
    ///
    /// Refer to the documentation of `UncheckedCell::as_mut_unchecked`.
    pub fn insert(&self, value: T)
    where
        T: Clone,
    {
        self.0.as_mut().insert(value);
    }

    /// # Safety
    ///
    /// Refer to the documentation of `UncheckedCell::as_mut_unchecked`.
    pub fn remove(&self, value: impl std::borrow::Borrow<T>)
    where
        T: Clone,
    {
        self.0.as_mut().remove(value.borrow());
    }

    pub fn contains(&self, value: impl std::borrow::Borrow<T>) -> bool
    where
        T: Clone,
    {
        self.0.as_mut().contains(value.borrow())
    }

    pub fn into_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.0.as_ref().iter().cloned().collect()
    }
}

impl<T: Eq + Hash> From<HashSet<T>> for Bag<T> {
    fn from(bag: HashSet<T>) -> Self {
        Self(Cell::new(bag))
    }
}
