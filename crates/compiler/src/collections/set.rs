#[derive(Debug, Clone)]
pub struct Set<K>(Vec<K>);

impl<K> Default for Set<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> Set<K> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, k: K)
    where
        K: PartialEq,
    {
        if !self.contains(&k) {
            self.0.push(k)
        }
    }

    pub fn contains(&self, k: &K) -> bool
    where
        K: PartialEq,
    {
        self.0.contains(k)
    }

    pub fn iter(&self) -> std::slice::Iter<K> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn take(&mut self) -> Set<K> {
        std::mem::take(self)
    }
}

impl<K> From<Vec<K>> for Set<K>
where
    K: PartialEq,
{
    fn from(v: Vec<K>) -> Self {
        let mut set = Set::new();
        for k in v {
            set.insert(k);
        }
        set
    }
}

impl<K> FromIterator<K> for Set<K>
where
    K: PartialEq,
{
    fn from_iter<I: IntoIterator<Item = K>>(iter: I) -> Self {
        let mut set = Set::new();
        for k in iter {
            set.insert(k);
        }
        set
    }
}

impl<K> IntoIterator for Set<K> {
    type Item = K;
    type IntoIter = std::vec::IntoIter<K>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K> Into<Vec<K>> for Set<K> {
    fn into(self) -> Vec<K> {
        self.0
    }
}

impl<K> PartialEq for Set<K>
where
    K: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len() && self.0.iter().all(|k| other.contains(k))
    }
}

impl<K> Eq for Set<K> where K: PartialEq {}

impl<K> std::hash::Hash for Set<K>
where
    K: std::hash::Hash + PartialEq + Ord + Clone,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut v = self.0.clone();
        v.sort();
        v.hash(state);
    }
}
