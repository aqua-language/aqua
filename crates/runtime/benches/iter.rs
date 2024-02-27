
// use criterion::{black_box, criterion_group, criterion_main, Criterion};
//
// fn impl0(c: &mut Criterion) {
//     c.bench_function("impl_iterator", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000).map(|x| x * 2);
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn dyn0(c: &mut Criterion) {
//     c.bench_function("boxed_iterator", |b| {
//         b.iter(|| {
//             let iter: Box<dyn Iterator<Item = i32>> = Box::new((0..1_000_000).map(|x| x * 2));
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn eager0(c: &mut Criterion) {
//     c.bench_function("eager_iterator", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x * 2);
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn impl1(c: &mut Criterion) {
//     c.bench_function("impl_iterator_complex", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000)
//                 .map(|x| x * 2)
//                 .filter(|&x| x % 3 == 0)
//                 .take(500)
//                 .enumerate()
//                 .map(|(i, x)| x + i as i32);
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn dyn1(c: &mut Criterion) {
//     c.bench_function("boxed_iterator_complex", |b| {
//         b.iter(|| {
//             let iter = Box::new((0..1_000_000).map(|x| x * 2)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.filter(|&x| x % 3 == 0)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.take(500)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.enumerate()) as Box<dyn Iterator<Item = (usize, i32)>>;
//             let iter = Box::new(iter.map(|(i, x)| x + i as i32)) as Box<dyn Iterator<Item = i32>>;
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn eager1(c: &mut Criterion) {
//     c.bench_function("eager_iterator_complex", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x * 2)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .filter(|&x| x % 3 == 0)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .take(500)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .enumerate()
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|(i, x)| x + i as i32);
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn impl2(c: &mut Criterion) {
//     c.bench_function("impl_iterator_more_complex", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000)
//                 .map(|x| x * 2)
//                 .filter(|&x| x % 3 == 0)
//                 .take(500)
//                 .enumerate()
//                 .map(|(i, x)| x + i as i32)
//                 .skip(100)
//                 .zip(100..600)
//                 .fold(0, |acc, (a, b)| acc + a + b);
//             black_box(iter);
//         });
//     });
// }
//
// fn dyn2(c: &mut Criterion) {
//     c.bench_function("boxed_iterator_more_complex", |b| {
//         b.iter(|| {
//             let iter = Box::new(0..1_000_000) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.map(|x| x * 2)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.filter(|&x| x % 3 == 0)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.take(500)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.enumerate()) as Box<dyn Iterator<Item = (usize, i32)>>;
//             let iter = Box::new(iter.map(|(i, x)| x + i as i32)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.skip(100)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.zip(100..600)) as Box<dyn Iterator<Item = (i32, i32)>>;
//             let result = iter.fold(0, |acc, (a, b)| acc + a + b);
//             black_box(result);
//         });
//     });
// }
//
// fn eager2(c: &mut Criterion) {
//     c.bench_function("eager_iterator_more_complex", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x * 2)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .filter(|&x| x % 3 == 0)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .take(500)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .enumerate()
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|(i, x)| x + i as i32)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .skip(100)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .zip(100..600)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .fold(0, |acc, (a, b)| acc + a + b);
//             black_box(iter);
//         });
//     });
// }
//
// fn impl3(c: &mut Criterion) {
//     c.bench_function("impl_iterator_maps", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000)
//                 .map(|x| x * 2)
//                 .map(|x| x + 1)
//                 .map(|x| x * 3)
//                 .map(|x| x - 1)
//                 .map(|x| x / 2)
//                 .map(|x| x + 1)
//                 .map(|x| x * 3);
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn f1(x: i32) -> i32 {
//     x * 2
// }
// fn f2(x: i32) -> i32 {
//     x + 1
// }
// fn f3(x: i32) -> i32 {
//     x * 3
// }
// fn f4(x: i32) -> i32 {
//     x - 1
// }
// fn f5(x: i32) -> i32 {
//     x / 2
// }
// fn f6(x: i32) -> i32 {
//     x + 1
// }
// fn f7(x: i32) -> i32 {
//     x * 3
// }
// fn impl_fn3(c: &mut Criterion) {
//     c.bench_function("impl_iterator_maps", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000)
//                 .map(f1)
//                 .map(f2)
//                 .map(f3)
//                 .map(f4)
//                 .map(f5)
//                 .map(f6)
//                 .map(f7);
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn dyn3(c: &mut Criterion) {
//     c.bench_function("boxed_iterator_maps", |b| {
//         b.iter(|| {
//             let iter = Box::new(0..1_000_000) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.map(|x| x * 2)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.map(|x| x + 1)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.map(|x| x * 3)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.map(|x| x - 1)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.map(|x| x / 2)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.map(|x| x + 1)) as Box<dyn Iterator<Item = i32>>;
//             let iter = Box::new(iter.map(|x| x * 3)) as Box<dyn Iterator<Item = i32>>;
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// fn eager3(c: &mut Criterion) {
//     c.bench_function("eager_iterator_maps", |b| {
//         b.iter(|| {
//             let iter = (0..1_000_000)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x * 2)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x + 1)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x * 3)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x - 1)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x / 2)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x + 1)
//                 .collect::<Vec<_>>()
//                 .into_iter()
//                 .map(|x| x * 3);
//             black_box(iter.collect::<Vec<i32>>());
//         });
//     });
// }
//
// criterion_group!(g0, impl0, dyn0, eager0);
// criterion_group!(g1, impl1, dyn1, eager1);
// criterion_group!(g2, impl2, dyn2, eager2);
// criterion_group!(g3, impl3, dyn3, eager3, impl_fn3);
// criterion_main!(g0, g1, g2, g3);
