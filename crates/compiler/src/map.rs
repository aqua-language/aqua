use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Map<K, V>(Vec<(K, V)>);

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<K, V> Map<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
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
        Map(self.0.iter().map(|(k, v)| f(k, v)).collect())
    }

    pub fn mapv<U, F>(&self, mut f: F) -> Map<K, U>
    where
        F: FnMut(&V) -> U,
        K: Clone,
    {
        Map(self.0.iter().map(|(k, v)| (k.clone(), f(v))).collect())
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
