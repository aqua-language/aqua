use compiler::ast::Bound;
use compiler::ast::StmtImpl;
use compiler::infer::Context;
use compiler::infer::solve;
use compiler::infer::unify;
use compiler::dsl::traits::impl_add;
use compiler::dsl::traits::impl_clone;
use compiler::dsl::traits::impl_intoiterator;
use compiler::dsl::traits::impl_iterator;
use compiler::dsl::traits::tr_add;
use compiler::dsl::traits::tr_clone;
use compiler::dsl::traits::tr_intoiterator;
use compiler::dsl::traits::tr_iterator;
use compiler::dsl::traits::ty_add_output;
use compiler::dsl::traits::ty_intoiterator_intoiter;
use compiler::dsl::traits::ty_iterator_item;
use compiler::dsl::ty;
use compiler::dsl::ty_con;
use compiler::dsl::ty_gen;
use compiler::dsl::types::ty_i32;
use compiler::dsl::types::ty_i64;
use compiler::dsl::types::ty_stream;
use compiler::dsl::types::ty_vec;

fn debug(impls: &[StmtImpl], goal: &Bound) {
    println!("impls:");
    for i in impls {
        println!("  {}", i);
    }
    println!("goal: {}", goal);
}

// impl[T] Clone[Vec[T]] where Clone[T] {}
// Goal: Clone[Vec[i32]]
#[test]
fn test_trait1() {
    let impls = [impl_clone(["T"], ty_gen("T"), [])];
    let mut sub = vec![];
    let goal = tr_clone(ty_i32());
    let mut ctx = Context::new();

    debug(&impls, &goal);
    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
}

// impl Clone[i32] {}
// Goal: Clone[i32]
#[test]
fn test_trait2() {
    let impls = [impl_clone([], ty_i32(), [])];
    let mut sub = vec![];

    let goal = tr_clone(ty_i32());
    let mut ctx = Context::new();

    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
}

// impl[T] Clone[Vec[T]] where Clone[T] {}
// impl Clone[i32] {}
// Goal: Clone[Vec[i32]]
#[test]
fn test_trait3() {
    let impls = [
        impl_clone(["T"], ty_vec(ty_gen("T")), [tr_clone(ty_gen("T"))]),
        impl_clone([], ty_i32(), []),
    ];
    let mut sub = vec![];

    let goal = tr_clone(ty_vec(ty_i32()));
    let mut ctx = Context::new();

    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
}

// impl[T] Clone[Vec[T]] where Clone[T] {}
// impl Clone[i32] {}
// Goal: Clone[Vec[Vec[i32]]]
#[test]
fn test_trait4() {
    let impls = [
        impl_clone(["T"], ty_vec(ty_gen("T")), [tr_clone(ty_gen("T"))]),
        impl_clone([], ty_i32(), []),
    ];

    let mut sub = vec![];
    let goal = tr_clone(ty_vec(ty_vec(ty_i32())));

    let mut ctx = Context::new();

    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
}

// impl[T] Iterator[Vec[?T]] { type Item = T; }
// Goal: Iterator[Vec[i32], Item = ?A]
// Unify: i32 and Iterator[Vec[i32]]::Item
#[ignore]
#[test]
fn test_trait5() {
    let impls = [impl_iterator(["T"], ty_vec(ty_gen("T")), ty_gen("T"), [])];
    let mut sub = vec![];
    let goal = tr_iterator(ty_vec(ty_i32()));
    let mut ctx = Context::new();
    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
    let t0 = ty_iterator_item(ty_vec(ty_i32()));
    let t1 = ty_i32();
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

// impl Add[i32, i32] { type Output = i32; }
// impl Add[f32, f32] { type Output = f32; }
// Goal: Add[i32, i32, Output = ?X]
#[ignore]
#[test]
fn test_trait6() {
    let impls = [
        impl_add([], [ty_i32(), ty_i32()], ty_i32(), []),
        impl_add([], [ty("f32"), ty("f32")], ty("f32"), []),
    ];
    let mut sub = vec![];
    let goal = tr_add([ty_i32(), ty_i32()]);
    let mut ctx = Context::new();
    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
    let t0 = ty_add_output([ty_i32(), ty_i32()]);
    let t1 = ty_i32();
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

// impl Add[i32, i32] { type Output = i32; }
// impl[T, R] Add[Vec[?T], ?R] where Add[?T, ?R] {
//     type Output = Vec[Add[?T, ?R]::Output];
// }
// Goal: Add[Vec[i32], i32, Output = ?Y]
// Unify: Vec[i32] and Add[Vec[i32], i32]::Output
#[ignore]
#[test]
fn test_trait7() {
    let impls = [
        impl_add([], [ty_i32(), ty_i32()], ty_i32(), []),
        impl_add(
            ["T", "R"],
            [ty_vec(ty_gen("T")), ty_gen("R")],
            ty_vec(ty_add_output([ty_gen("T"), ty_gen("R")])),
            [tr_add([ty_gen("T"), ty_gen("R")])],
        ),
    ];
    let mut sub = vec![];
    let goal = tr_add([ty_vec(ty_i32()), ty_i32()]);
    let mut ctx = Context::new();
    let impls = impls
        .into_iter()
        .map(|i| i.annotate(&mut ctx))
        .collect::<Vec<_>>();
    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
    let t0 = ty_add_output([ty_vec(ty_i32()), ty_i32()]);
    let t1 = ty_vec(ty_i32());
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

// impl Add[i32, i32] { type Output = i32; }
// impl Add[i64, i32] { type Output = i64; }
// Goal: Add[i64, i32, Output = ?X]
// Unify: i64 and Add[i64, i32]::Output
#[ignore]
#[test]
fn test_trait8() {
    let impls = [
        impl_add([], [ty_i32(), ty_i32()], ty_i32(), []),
        impl_add([], [ty_i64(), ty_i32()], ty_i64(), []),
    ];
    let mut sub = vec![];
    let goal = tr_add([ty_i64(), ty_i32()]);
    let mut ctx = Context::new();
    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
    let t0 = ty_add_output([ty_i64(), ty_i32()]);
    let t1 = ty_i64();
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

// impl[T] IntoIterator[Vec[?T]] {
//    type Item = ?T;
//    type IntoIter = VecIterator[?T];
// }
// impl[T] Iterator[VecIterator[?T]] {
//    type Item = ?T;
// }
// Goal: IntoIterator[Vec[i32], Item = ?A, IntoIter = ?B]
// Unify: VecIterator[i32] and Vec[i32]::IntoIter
#[ignore]
#[test]
fn test_trait9() {
    let impls = [
        impl_intoiterator(
            ["T"],
            ty_vec(ty_gen("T")),
            ty_gen("T"),
            ty_con("VecIterator", [ty_gen("T")]),
            [],
        ),
        impl_iterator(["T"], ty_con("VecIterator", [ty_gen("T")]), ty_gen("T"), []),
    ];
    let mut sub = vec![];
    let goal = tr_intoiterator(ty_vec(ty_i32()));
    let mut ctx = Context::new();
    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
    let t0 = ty_intoiterator_intoiter(ty_vec(ty_i32()));
    let t1 = ty_con("VecIterator", [ty_i32()]);
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

// impl[T] IntoIterator[Vec[?T]] {
//    type Item = ?T;
//    type IntoIter = VecIterator[?T];
// }
// impl IntoIterator[Stream[?T]] {
//    type Item = ?T;
//    type IntoIter = Stream[?T];
// }
#[test]
fn test_trait10() {
    let impls = [
        impl_intoiterator(
            ["T"],
            ty_vec(ty_gen("T")),
            ty_gen("T"),
            ty_con("VecIterator", [ty_gen("T")]),
            []
        ),
        impl_intoiterator(
            ["T"],
            ty_stream(ty_gen("T")),
            ty_gen("T"),
            ty_stream(ty_gen("T")),
            []
        ),
    ];

    let mut sub = vec![];
    let goal = tr_intoiterator(ty_vec(ty_i32()));
    let mut ctx = Context::new();
    assert!(solve(&goal, &impls, &[], &mut sub, &mut ctx).is_some());
}
