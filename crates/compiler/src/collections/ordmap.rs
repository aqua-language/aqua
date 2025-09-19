use std::collections::hash_map::OccupiedEntry as HashOccupiedEntry;
use std::collections::hash_map::VacantEntry as HashVacantEntry;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct OrdMap<K, V> {
    map: HashMap<K, V>,
    keys: Vec<K>,
}

impl<K, V> Eq for OrdMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Eq,
{
}

impl<K, V> PartialEq for OrdMap<K, V>
where
    K: Eq + Hash + Clone,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.keys.len() != other.keys.len() {
            return false;
        }
        for (k1, k2) in self.keys.iter().zip(other.keys.iter()) {
            if k1 != k2 || self.map[k1] != other.map[k2] {
                return false;
            }
        }
        true
    }
}

impl<K, V> Default for OrdMap<K, V> {
    fn default() -> Self {
        OrdMap {
            map: HashMap::new(),
            keys: Vec::new(),
        }
    }
}

pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    entry: HashOccupiedEntry<'a, K, V>,
}

pub struct VacantEntry<'a, K: 'a, V: 'a> {
    entry: HashVacantEntry<'a, K, V>,
    keys: &'a mut Vec<K>,
    key: K,
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Clone,
{
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    pub fn get(&self) -> &V {
        self.entry.get()
    }

    pub fn get_mut(&mut self) -> &mut V {
        self.entry.get_mut()
    }

    pub fn into_mut(self) -> &'a mut V {
        self.entry.into_mut()
    }

    pub fn insert(&mut self, value: V) -> V {
        self.entry.insert(value)
    }
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Clone,
{
    pub fn insert(self, value: V) -> &'a mut V {
        self.keys.push(self.key.clone());
        self.entry.insert(value)
    }
}

impl<K, V> OrdMap<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Eq + Hash + Clone,
    {
        let old_value = self.map.insert(key.clone(), value);
        if old_value.is_none() {
            self.keys.push(key);
        }
        old_value
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        match self.map.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                Entry::Occupied(OccupiedEntry { entry })
            }
            std::collections::hash_map::Entry::Vacant(entry) => Entry::Vacant(VacantEntry {
                entry,
                keys: &mut self.keys,
                key,
            }),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Eq + Hash,
    {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Eq + Hash,
    {
        self.map.get_mut(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys
            .iter()
            .filter_map(|k| self.map.get(k).map(|v| (k, v)))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.keys.iter().filter_map(|k| self.map.get(k))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.iter()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
