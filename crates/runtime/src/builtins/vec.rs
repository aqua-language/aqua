use macros::Send;
use macros::Sync;
use macros::Unpin;

use serde::Deserialize;
use serde::Serialize;

use crate::builtins::option::Option;
use crate::traits::DeepClone;

use super::cell::Cell;

#[derive(Debug, Send, Sync, Unpin, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub struct Vec<T>(pub Cell<std::vec::Vec<T>>);

impl<T: std::fmt::Display> std::fmt::Display for Vec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        let x = self.0.as_ref();
        let mut iter = x.iter();
        if let Some(v) = iter.next() {
            write!(f, "{}", v)?;
            for v in iter {
                write!(f, ", {}", v)?;
            }
        }
        write!(f, "]")
    }
}

impl<T> Clone for Vec<T> {
    fn clone(&self) -> Self {
        Vec(self.0.clone())
    }
}

impl<T: std::hash::Hash> std::hash::Hash for Vec<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: DeepClone> DeepClone for Vec<T> {
    fn deep_clone(&self) -> Self {
        Self(self.0.deep_clone())
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Vec(Cell::new(std::vec::Vec::<T>::new()))
    }
}

impl<T> Vec<T> {
    pub fn new() -> Vec<T> {
        Self::default()
    }

    pub fn len(self) -> usize {
        self.0.as_ref().len()
    }

    pub fn push(&self, value: T)
    where
        T: Clone,
    {
        self.0.as_mut().push(value);
    }

    pub fn pop(&self) -> Option<T>
    where
        T: Clone,
    {
        Option::from(self.0.as_mut().pop())
    }

    pub fn remove(&self, index: usize) -> T
    where
        T: Clone,
    {
        self.0.as_mut().remove(index)
    }

    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        Option::from(self.0.as_ref().get(index).cloned())
    }

    pub fn insert(&self, index: usize, value: T)
    where
        T: Clone,
    {
        self.0.as_mut().insert(index, value);
    }

    pub fn is_empty(self) -> bool {
        self.0.as_ref().is_empty()
    }

    pub fn sort(&self)
    where
        T: Clone + PartialOrd,
    {
        self.0
            .as_mut()
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn truncate(&self, len: usize)
    where
        T: Clone,
    {
        self.0.as_mut().truncate(len);
    }

    pub fn clear(&self) {
        self.0.as_mut().clear();
    }

    pub fn iter(&self) -> VecIter<T>
    where
        T: Clone,
    {
        VecIter {
            vec: self.clone(),
            index: 0,
        }
    }

    pub fn extend(&self, other: Vec<T>)
    where
        T: Clone,
    {
        self.0.as_mut().extend(other.0.as_ref().clone());
    }
}

impl<T> From<std::vec::Vec<T>> for Vec<T> {
    fn from(vec: std::vec::Vec<T>) -> Self {
        Vec(Cell::new(vec))
    }
}

pub struct VecIter<T> {
    vec: Vec<T>,
    index: usize,
}

impl<T> Iterator for VecIter<T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> std::option::Option<Self::Item> {
        let result = self.vec.get(self.index);
        self.index += 1;
        result.0
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Vec(Cell::new(std::vec::Vec::from_iter(iter)))
    }
}
