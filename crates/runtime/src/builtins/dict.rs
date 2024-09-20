use std::hash::Hash;

use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;
use crate::HashMap;

use super::cell::Cell;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub struct Dict<K: Eq + Hash, V>(pub Cell<HashMap<K, V>>);

impl<K: Eq + Hash + std::fmt::Display, V: std::fmt::Display> std::fmt::Display for Dict<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{")?;
        let x = self.0.as_ref();
        let mut iter = x.iter();
        if let Some((k, v)) = iter.next() {
            write!(f, "{}: {}", k, v)?;
            for (k, v) in iter {
                write!(f, ", {}: {}", k, v)?;
            }
        }
        write!(f, "}}")
    }
}

impl<K: Eq + Hash + DeepClone, V: DeepClone> DeepClone for Dict<K, V> {
    fn deep_clone(&self) -> Self {
        let map = self
            .0
            .as_ref()
            .iter()
            .map(|(k, v)| (k.deep_clone(), v.deep_clone()))
            .collect();
        Dict(Cell::new(map))
    }
}

impl<K: Eq + Hash, V> Dict<K, V> {
    pub fn new() -> Dict<K, V> {
        Dict(Cell::new(HashMap::default()))
    }

    pub fn get(&self, key: impl std::borrow::Borrow<K>) -> Option<V>
    where
        K: Clone,
        V: Clone,
    {
        self.0.as_ref().get(key.borrow()).cloned()
    }

    pub fn insert(&self, key: K, val: V)
    where
        K: Clone,
        V: Clone,
    {
        self.0.as_mut().insert(key, val);
    }

    pub fn remove(&self, key: impl std::borrow::Borrow<K>) -> Option<V>
    where
        K: Clone,
        V: Clone,
    {
        self.0.as_mut().remove(key.borrow())
    }

    pub fn contains_key(&self, key: impl std::borrow::Borrow<K>) -> bool {
        self.0.as_ref().contains_key(key.borrow())
    }
}

impl<K: Eq + Hash, V> From<HashMap<K, V>> for Dict<K, V> {
    fn from(map: HashMap<K, V>) -> Self {
        Dict(Cell::new(map))
    }
}
