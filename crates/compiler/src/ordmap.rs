use std::collections::HashMap;
use std::hash::Hash;

struct OrdMap<K, V> {
    keys: HashMap<K, usize>,
    values: Vec<V>,
}

impl<K, V> OrdMap<K, V> {
    fn new() -> Self {
        OrdMap {
            keys: HashMap::new(),
            values: Vec::new(),
        }
    }

    fn insert(&mut self, key: K, value: V)
    where
        K: Eq + Hash,
    {
        let index = self.values.len();
        self.keys.insert(key, index);
        self.values.push(value);
    }

    fn get(&self, key: &K) -> Option<&V>
    where
        K: Eq + Hash,
    {
        self.keys.get(key).map(|&index| &self.values[index])
    }

    fn values(&self) -> impl Iterator<Item = &V> {
        self.values.iter()
    }
}
