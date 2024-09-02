// use std::borrow::Borrow;
use std::hash::Hash;

use serde::Deserialize;
use serde::Serialize;

use crate::builtins::unchecked_cell::UncheckedCell;
use crate::traits::Data;
use crate::traits::DeepClone;
use crate::traits::Key;
use crate::HashMap;

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub struct Dict<K: Eq + Hash, V>(pub UncheckedCell<HashMap<K, V>>);

impl<K: Eq + Hash + std::fmt::Display, V: std::fmt::Display> std::fmt::Display for Dict<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut iter = self.0.iter();
        if let Some((k, v)) = iter.next() {
            write!(f, "{}: {}", k, v)?;
            for (k, v) in iter {
                write!(f, ", {}: {}", k, v)?;
            }
        }
        write!(f, "}}")
    }
}

impl<K: Key, V: Data> DeepClone for Dict<K, V> {
    fn deep_clone(&self) -> Self {
        todo!()
        // let map = self
        //     .0
        //     .iter()
        //     .map(|(k, v)| (k.deep_clone(), v.deep_clone()))
        //     .collect();
        // Dict(map)
    }
}

impl<K: Eq + Hash, V> Dict<K, V> {
    pub fn new() -> Dict<K, V> {
        Dict(UncheckedCell::new(HashMap::default()))
    }

    pub fn get(&self, key: impl std::borrow::Borrow<K>) -> Option<V>
    where
        K: Clone,
        V: Clone,
    {
        self.0.get(key.borrow()).cloned()
    }

    pub fn insert(&self, key: K, val: V)
    where
        K: Clone,
        V: Clone,
    {
        // Safety: This is an atomic operation.
        unsafe {
            self.0.as_mut_unchecked().insert(key, val);
        }
    }

    pub fn remove(&self, key: impl std::borrow::Borrow<K>)
    where
        K: Clone,
        V: Clone,
    {
        // Safety: This is an atomic operation.
        unsafe {
            self.0.as_mut_unchecked().remove(key.borrow());
        }
    }

    pub fn contains_key(&self, key: impl std::borrow::Borrow<K>) -> bool {
        self.0.contains_key(key.borrow())
    }
}

impl<K: Eq + Hash, V> From<HashMap<K, V>> for Dict<K, V> {
    fn from(map: HashMap<K, V>) -> Self {
        Dict(UncheckedCell::new(map))
    }
}
