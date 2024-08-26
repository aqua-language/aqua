use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Map<K, V>(Vec<(K, V)>);

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub enum Entry<'a, K, V> {
    Occupied(&'a mut V),
    Vacant(&'a mut Map<K, V>, K),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: PartialEq + Clone,
{
    pub fn or_insert_with<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(v1) => v1,
            Entry::Vacant(map, k) => {
                map.insert(k.clone(), f());
                map.get_mut(&k).unwrap()
            }
        }
    }
}

impl<K, V> Map<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn entry<'a>(&'a mut self, k: K) -> Entry<'a, K, V>
    where
        K: PartialEq,
    {
        let idx = self.0.iter().position(|(k1, _)| k1 == &k);
        match idx {
            Some(idx) => Entry::Occupied(&mut self.0[idx].1),
            None => Entry::Vacant(self, k),
        }
    }

    pub fn singleton(k: K, v: V) -> Self {
        Self(vec![(k, v)])
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.0.push((k, v))
    }

    pub fn get(&self, k: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        self.0.iter().find(|(k1, _)| k1 == k).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V>
    where
        K: PartialEq,
    {
        self.0.iter_mut().find(|(k1, _)| k1 == k).map(|(_, v)| v)
    }

    pub fn contains_key(&self, k: &K) -> bool
    where
        K: PartialEq,
    {
        self.0.iter().any(|(k1, _)| k1 == k)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<(K, V)> {
        self.0.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }

    pub fn map<U, F>(&self, mut f: F) -> Map<K, U>
    where
        F: FnMut(&K, &V) -> (K, U),
    {
        Map(self.iter().map(|(k, v)| f(k, v)).collect())
    }

    pub fn mapv<U, F>(&self, mut f: F) -> Map<K, U>
    where
        F: FnMut(&V) -> U,
        K: Clone,
    {
        Map(self.iter().map(|(k, v)| (k.clone(), f(v))).collect())
    }

    pub fn same_keys(&self, other: &Self) -> bool
    where
        K: PartialEq,
    {
        self.iter().all(|(k, _)| other.contains_key(k))
            && other.iter().all(|(k, _)| self.contains_key(k))
    }

    pub fn intersect(&self, other: &Self) -> Self
    where
        K: PartialEq,
        K: Clone,
        V: Clone,
    {
        Self(
            self.iter()
                .filter(|(k, _)| other.contains_key(k))
                .cloned()
                .collect(),
        )
    }

    pub fn union(&self, other: &Self) -> Self
    where
        K: Clone,
        V: Clone,
    {
        self.iter()
            .chain(other.iter())
            .fold(Map::new(), |mut acc, (k, v)| {
                acc.insert(k.clone(), v.clone());
                acc
            })
    }

    pub fn sort_keys(&self) -> Self
    where
        K: Clone + Ord,
        V: Clone,
    {
        let mut v = self.0.clone();
        v.sort_by_key(|(k, _)| k.clone());
        Self(v)
    }

    pub fn same_keys_sorted(&self, other: &Self) -> bool
    where
        K: PartialEq + Ord,
    {
        for ((k1, _), (k2, _)) in self.iter().zip(other.iter()) {
            if k1 != k2 {
                return false;
            }
        }
        return true;
    }
}

impl<K, V> AsRef<[(K, V)]> for Map<K, V> {
    fn as_ref(&self) -> &[(K, V)] {
        &self.0
    }
}

impl<K, V> Deref for Map<K, V> {
    type Target = [(K, V)];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> std::ops::DerefMut for Map<K, V> {
    fn deref_mut(&mut self) -> &mut [(K, V)] {
        &mut self.0
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, V> From<Vec<(K, V)>> for Map<K, V> {
    fn from(v: Vec<(K, V)>) -> Self {
        Self(v)
    }
}

impl<const N: usize, K, V> From<[(K, V); N]> for Map<K, V> {
    fn from(v: [(K, V); N]) -> Self {
        Self(v.into_iter().collect())
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = &'a (K, V);
    type IntoIter = std::slice::Iter<'a, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
