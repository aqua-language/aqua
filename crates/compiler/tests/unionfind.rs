use compiler::collections::unionfind::Union;
use compiler::collections::unionfind::UnionFind;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Key(u32);

impl Union for Key {
    type Value = ();

    fn into_u32(self) -> u32 {
        self.0
    }
    fn from_u32(i: u32) -> Self {
        Key(i)
    }
    fn union(_: &Self::Value, _: &Self::Value) -> Self::Value {
        ()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SumKey(u32);

impl Union for SumKey {
    type Value = i32;

    fn into_u32(self) -> u32 {
        self.0
    }
    fn from_u32(i: u32) -> Self {
        SumKey(i)
    }
    fn union(a: &Self::Value, b: &Self::Value) -> Self::Value {
        *a + *b
    }
}

#[test]
fn union_find0() {
    let mut uf = UnionFind::<Key>::new();
    for i in 0..10 {
        assert_eq!(i, uf.make(()).0);
    }
}

#[test]
fn union_find1() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    assert_eq!(a, uf.find(a));
}

#[test]
fn union_find2() {
    let mut uf = UnionFind::<Key>::new();
    let mut a = uf.make(());
    for _ in 0..5 {
        let b = uf.make(());
        uf.union(a, b);
        a = b;
        assert_eq!(uf.find(a), uf.find(b));
    }
}

#[test]
fn union_find3() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    uf.union(a, b);
    assert_eq!(uf.find(a), uf.find(b));
}

#[test]
fn union_find4() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    uf.union(a, a);
    assert_eq!(a, uf.find(a));
}

#[test]
fn union_find5() {
    let mut uf = UnionFind::<SumKey>::new();
    let a = uf.make(10);
    let b = uf.make(20);
    uf.union(a, b);
    let rep = uf.find(a);
    assert_eq!(uf.probe(rep), 30);
    uf.union_value(a, 5);
    assert_eq!(uf.probe(a), 35);
}

#[test]
fn union_find6() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    uf.snapshot();
    uf.union(a, b);
    assert_eq!(uf.find(a), uf.find(b));
    uf.rollback();
    assert_eq!(uf.find(a), a);
    assert_eq!(uf.find(b), b);
}

#[test]
fn union_find7() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    uf.snapshot();
    uf.union(a, b);
    uf.commit();
    assert_eq!(uf.find(a), uf.find(b));
}

#[test]
fn union_find8() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    let c = uf.make(());
    uf.snapshot();
    uf.union(a, b);
    uf.snapshot();
    uf.union(b, c);
    assert_eq!(uf.find(a), uf.find(c));
    uf.rollback();
    assert_eq!(uf.find(a), uf.find(b));
    assert_ne!(uf.find(a), c);
    uf.rollback();
    assert_ne!(uf.find(a), uf.find(b));
}

#[test]
fn union_find9() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    let c = uf.make(());
    uf.union(a, b);
    uf.union(b, c);
    uf.find_mut(c);
    assert_eq!(uf.find(a), uf.find(c));
}

#[test]
fn union_find10() {
    let mut uf = UnionFind::<Key>::new();
    let keys: Vec<_> = (0..5).map(|_| uf.make(())).collect();
    uf.union(keys[0], keys[1]);
    uf.union(keys[2], keys[3]);
    uf.union(keys[1], keys[2]);
    uf.union(keys[0], keys[4]);
    for k in &keys {
        assert_eq!(uf.find(keys[0]), uf.find(*k));
    }
}

#[test]
fn union_find11() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    uf.union(a, a);
    assert_eq!(uf.find(a), a);
}

#[test]
fn union_find12() {
    let mut uf = UnionFind::<SumKey>::new();
    let a = uf.make(100);
    let b = uf.make(200);
    uf.snapshot();
    uf.union(a, b);
    assert_eq!(uf.probe(uf.find(a)), 300);
    uf.rollback();
    assert_eq!(uf.probe(a), 100);
    assert_eq!(uf.probe(b), 200);
}

#[test]
fn union_find13() {
    let mut uf = UnionFind::<Key>::new();
    uf.commit();
    let a = uf.make(());
    assert_eq!(uf.find(a), a);
}

#[test]
fn union_find14() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    uf.snapshot();
    uf.snapshot();
    let b = uf.make(());
    uf.union(a, b);
    uf.rollback();
    assert_eq!(uf.find(a), a);
    uf.rollback();
    assert_eq!(uf.find(a), a);
}

#[test]
fn union_find15() {
    let mut uf = UnionFind::<SumKey>::new();
    let a = uf.make(1);
    let b = uf.make(2);
    let c = uf.make(3);
    uf.union(a, b);
    uf.union_value(a, 10);
    let rep = uf.find(a);
    assert_eq!(uf.probe(rep), 13);
    uf.union(rep, c);
    assert_eq!(uf.probe(rep), 16);
}

#[test]
fn union_find16() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    let c = uf.make(());
    uf.union(a, b);
    uf.union(a, c);
    assert_eq!(uf.find(a), uf.find(b));
    assert_eq!(uf.find(a), uf.find(c));
}

#[test]
fn union_find17() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    let c = uf.make(());
    uf.union(a, b);
    assert_ne!(uf.find(a), uf.find(c));
}

#[test]
fn union_find18() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    uf.snapshot();
    uf.union(a, b);
    uf.snapshot();
    let c = uf.make(());
    uf.union(a, c);
    uf.rollback();
    assert_eq!(uf.find(a), uf.find(b));
    uf.rollback();
    assert_ne!(uf.find(a), uf.find(b));
}

#[test]
fn union_find19() {
    let mut uf = UnionFind::<SumKey>::new();
    let a = uf.make(5);
    let b = uf.make(10);
    uf.union_value(a, 20);
    assert_eq!(uf.probe(a), 25);
    assert_eq!(uf.probe(b), 10);
}

#[test]
fn union_find20_large_chain() {
    let mut uf = UnionFind::<Key>::new();
    let n = 1000;
    let keys: Vec<_> = (0..n).map(|_| uf.make(())).collect();
    for i in 1..n {
        uf.union(keys[i - 1], keys[i]);
    }
    let root = uf.find(keys[0]);
    for i in 1..n {
        assert_eq!(uf.find(keys[i]), root);
    }
}

#[test]
fn union_find21_repeated_union() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    uf.union(a, b);
    uf.union(a, b); // no-op
    assert_eq!(uf.find(a), uf.find(b));
}

#[test]
fn union_find22_nested_snapshots() {
    let mut uf = UnionFind::<Key>::new();
    let a = uf.make(());
    let b = uf.make(());
    uf.snapshot();
    uf.union(a, b);
    uf.snapshot();
    let c = uf.make(());
    uf.union(b, c);
    uf.rollback();
    assert_eq!(uf.find(a), uf.find(b));
    assert_ne!(uf.find(a), uf.find(c));
    uf.rollback();
    assert_ne!(uf.find(a), uf.find(b));
}

#[test]
fn union_find23_union_value_negative() {
    let mut uf = UnionFind::<SumKey>::new();
    let a = uf.make(-10);
    let b = uf.make(-5);
    uf.union(a, b);
    assert_eq!(uf.probe(uf.find(a)), -15);
    uf.union_value(a, -20);
    assert_eq!(uf.probe(a), -35);
}
