#[derive(Debug)]
pub struct UnionFind<K: Union> {
    parent: Vec<K>,
    rank: Vec<u32>,
    values: Vec<K::Value>,
    snapshots: Vec<Vec<(usize, K, u32, K::Value)>>,
}

impl<K: Union> Default for UnionFind<K> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Union: Copy + PartialEq {
    type Value: Clone;
    fn into_u32(self) -> u32;
    fn from_u32(i: u32) -> Self;
    fn union(a: &Self::Value, b: &Self::Value) -> Self::Value;
}

fn idx(k: impl Union) -> usize {
    k.into_u32() as usize
}

impl<K: Union> UnionFind<K> {
    pub fn new() -> Self {
        Self {
            parent: Vec::new(),
            rank: Vec::new(),
            values: Vec::new(),
            snapshots: Vec::new(),
        }
    }

    pub fn probe(&self, x: K) -> K::Value {
        let rep = self.find(x);
        self.values[idx(rep)].clone()
    }

    pub fn make(&mut self, value: K::Value) -> K {
        let uid = K::from_u32(self.parent.len() as u32);
        self.parent.push(uid);
        self.rank.push(0);
        self.values.push(value);
        uid
    }

    pub fn find(&self, x: K) -> K {
        let mut cur = x;
        while self.parent[idx(cur)] != cur {
            cur = self.parent[idx(cur)];
        }
        cur
    }

    pub fn find_mut(&mut self, x: K) -> K {
        let mut cur = x;
        while self.parent[idx(cur)] != cur {
            cur = self.parent[idx(cur)];
        }
        let root = cur;
        cur = x;
        while self.parent[idx(cur)] != root {
            self.record(idx(cur)); // record the parent update
            self.parent[idx(cur)] = root;
            cur = self.parent[idx(cur)];
        }
        root
    }

    fn record(&mut self, index: usize) {
        if let Some(snap) = self.snapshots.last_mut() {
            snap.push((
                index,
                self.parent[index],
                self.rank[index],
                self.values[index].clone(),
            ));
        }
    }

    pub fn union(&mut self, x: K, y: K) {
        let x_root = self.find(x);
        let y_root = self.find(y);
        if x_root == y_root {
            return;
        }
        let (xi, yi) = (idx(x_root), idx(y_root));
        if self.rank[xi] > self.rank[yi] {
            self.record(yi);
            self.parent[yi] = x_root;
            self.record(xi);
            self.values[xi] = K::union(&self.values[xi], &self.values[yi]);
        } else if self.rank[xi] < self.rank[yi] {
            self.record(xi);
            self.parent[xi] = y_root;
            self.record(yi);
            self.values[yi] = K::union(&self.values[yi], &self.values[xi]);
        } else {
            self.record(yi);
            self.parent[yi] = x_root;
            self.record(xi);
            self.values[xi] = K::union(&self.values[xi], &self.values[yi]);
            self.record(xi);
            self.rank[xi] += 1;
        }
    }

    pub fn union_value(&mut self, x: K, v: K::Value) -> K::Value {
        let x_root = self.find(x);
        let xi = idx(x_root);
        self.record(xi);
        self.values[xi] = K::union(&self.values[xi], &v);
        self.values[xi].clone()
    }

    pub fn snapshot(&mut self) {
        self.snapshots.push(Vec::new());
    }

    pub fn rollback(&mut self) {
        if let Some(snap) = self.snapshots.pop() {
            for (i, p, r, v) in snap.into_iter().rev() {
                self.parent[i] = p;
                self.rank[i] = r;
                self.values[i] = v;
            }
        }
    }

    pub fn commit(&mut self) {
        self.snapshots.pop();
    }
}
