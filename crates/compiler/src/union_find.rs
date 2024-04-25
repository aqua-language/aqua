#[derive(Clone, Debug)]
struct UnionFind<T> {
    data: Vec<T>,
    parent: Vec<Id>,
}

struct Node<T> {
    id: Id,
    data: T,
    version: u16,
}

impl<T> Default for UnionFind<T> {
    fn default() -> Self {
        UnionFind {
            parent: Vec::new(),
            data: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Id(usize);

impl<T> UnionFind<T> {
    fn new() -> Self {
        UnionFind::default()
    }

    fn add(&mut self, x: T) -> Id {
        let id = Id(self.parent.len());
        self.parent.push(id);
        self.data.push(x);
        id
    }

    fn get(&mut self, x: Id) -> &T {
        let id = self.find(x).0;
        &self.data[id]
    }

    fn union(&mut self, x: Id, y: Id) {
        let x_root = self.find(x);
        let y_root = self.find(y);
        if x_root != y_root {
            if x_root.0 < y_root.0 {
                self.parent[y_root.0] = x_root;
            } else {
                self.parent[x_root.0] = y_root;
            }
        }
    }

    fn find(&mut self, mut x: Id) -> Id {
        let mut root = x;
        while root != self.parent[root.0] {
            root = self.parent[root.0];
        }
        // Path compression
        while x != root {
            let new_x = self.parent[x.0];
            self.parent[x.0] = root;
            x = new_x;
        }
        root
    }
}
