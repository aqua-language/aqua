use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use std::cell::RefCell;
use std::rc::Rc;

fn for_each_iter(c: &mut Criterion) {
    c.bench_function("for_each_iter", |b| {
        b.iter(|| {
            let iter = 0..1_000_000;
            iter.for_each(|x| {
                black_box(x);
            });
        });
    });
}

fn for_each_dyn_iter(c: &mut Criterion) {
    c.bench_function("for_each_dyn_iter", |b| {
        b.iter(|| {
            let iter: Box<dyn Iterator<Item = i32>> = Box::new(0..1_000_000);
            iter.for_each(|x| {
                black_box(x);
            });
        });
    });
}

fn push_vec(c: &mut Criterion) {
    c.bench_function("push_vec", |b| {
        b.iter(|| {
            let mut v = std::vec::Vec::<i32>::with_capacity(1_000_000);
            for i in 0..1_000_000 {
                let i = black_box(i);
                v.push(i);
            }
        });
    });
}

fn push_rc_refcell_vec(c: &mut Criterion) {
    c.bench_function("push_rc_refcell_vec", |b| {
        b.iter(|| {
            let v = Rc::new(RefCell::new(Vec::<i32>::with_capacity(1_000_000)));
            for i in 0..1_000_000 {
                let mut v = black_box(v.borrow_mut());
                let i = black_box(i);
                v.push(i);
            }
        });
    });
}

criterion_group!(b0, for_each_iter, for_each_dyn_iter);
criterion_group!(b1, push_vec, push_rc_refcell_vec);
criterion_main!(b0, b1);
