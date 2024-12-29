//! A vector that stores items which contain a key.
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KeyVec<T>(Vec<T>);

pub trait Key {
    type K: PartialEq;
    fn key(&self) -> &Self::K;
}

impl<T: Key> Default for KeyVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Entry<'a, T: Key> {
    Occupied(&'a mut T),
    Vacant(&'a mut KeyVec<T>, T::K),
}

impl<'a, T: Key> Entry<'a, T>
where
    T::K: Clone,
{
    pub fn or_insert_with<F>(self, f: F) -> &'a mut T
    where
        F: FnOnce() -> T,
    {
        match self {
            Entry::Occupied(v1) => v1,
            Entry::Vacant(map, k) => {
                map.insert(f());
                map.get_mut(&k).unwrap()
            }
        }
    }
}

impl<T: Key> KeyVec<T> {
    pub fn new() -> Self {
        KeyVec(Vec::new())
    }

    pub fn entry<'a>(&'a mut self, k: T::K) -> Entry<'a, T> {
        let idx = self.0.iter().position(|v| v.key() == &k);
        match idx {
            Some(idx) => Entry::Occupied(&mut self.0[idx]),
            None => Entry::Vacant(self, k),
        }
    }

    pub fn singleton(v: T) -> Self {
        Self(vec![v])
    }

    pub fn insert(&mut self, value: T) {
        if let Some(idx) = self.0.iter().position(|v| v.key() == value.key()) {
            self.0[idx] = value;
        } else {
            self.0.push(value);
        }
    }

    pub fn get(&self, key: &T::K) -> Option<&T> {
        self.0.iter().find(|v| v.key() == key)
    }

    pub fn get_mut(&mut self, k: &T::K) -> Option<&mut T> {
        self.0.iter_mut().find(|v| v.key() == k)
    }

    pub fn contains_key(&self, k: &T::K) -> bool {
        self.0.iter().any(|v| v.key() == k)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.0.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &T::K> {
        self.0.iter().map(|v| v.key())
    }

    pub fn remove(&mut self, k: &T::K) -> Option<T> {
        match self.0.iter().position(|v| v.key() == k) {
            Some(idx) => Some(self.0.remove(idx)),
            None => None,
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    /// Map that assumes that the keys are not changed.
    pub fn map<U: Key>(&self, f: impl FnMut(&T) -> U) -> KeyVec<U> {
        KeyVec(self.iter().map(f).collect())
    }

    pub fn same_keys(&self, other: &Self) -> bool {
        self.iter().all(|v| other.contains_key(v.key()))
            && other.iter().all(|v| self.contains_key(v.key()))
    }

    pub fn intersect(&self, other: &Self) -> Self
    where
        T::K: Clone,
        T: Clone,
    {
        Self(
            self.iter()
                .filter(|v| other.contains_key(v.key()))
                .cloned()
                .collect(),
        )
    }

    pub fn union(&self, other: &Self) -> Self
    where
        T::K: Clone,
        T: Clone,
    {
        self.iter()
            .chain(other.iter())
            .fold(KeyVec::new(), |mut acc, v| {
                acc.insert(v.clone());
                acc
            })
    }

    pub fn sort_keys(&self) -> Self
    where
        T::K: Ord + Clone,
        T: Clone,
    {
        let mut vec = self.0.clone();
        vec.sort_by_key(|v| v.key().clone());
        Self(vec)
    }

    pub fn same_keys_sorted(&self, other: &Self) -> bool
    where
        T::K: Ord,
    {
        for (v1, v2) in self.iter().zip(other.iter()) {
            if v1.key() != v2.key() {
                return false;
            }
        }
        return true;
    }
}

impl<T: Key> AsRef<[T]> for KeyVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Key> Deref for KeyVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Key> std::ops::DerefMut for KeyVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T: Key> IntoIterator for KeyVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Key> From<Vec<T>> for KeyVec<T> {
    fn from(vec: Vec<T>) -> Self {
        vec.into_iter().collect()
    }
}

impl<const N: usize, T: Key> From<[T; N]> for KeyVec<T> {
    fn from(vec: [T; N]) -> Self {
        Self(vec.into_iter().collect())
    }
}

impl<T: Key> FromIterator<T> for KeyVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter().fold(KeyVec::new(), |mut acc, v| {
            acc.insert(v);
            acc
        })
    }
}

impl<'a, T: Key> IntoIterator for &'a KeyVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
