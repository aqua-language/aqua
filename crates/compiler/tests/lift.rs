#[macro_use]
mod common;

use common::dsl::expr_block;
use common::dsl::expr_call_direct;
use common::dsl::expr_int;
use common::dsl::program;
use common::dsl::stmt_def;
use common::dsl::types::ty_i32;
use compiler::aqua;

use crate::common::passes::lift;

#[test]
fn test_lift0() {
    let a = lift(aqua!("def g(): i32 = 1;")).unwrap();
    let b = program([stmt_def("g", [], [], ty_i32(), [], expr_int("1"))]);
    check!(a, b);
}

#[test]
fn test_lift1() {
    let a = lift(aqua!(
        "def f(): i32 = {
            def g(): i32 = 1;
            g()
        }"
    ))
    .unwrap();
    let b = lift(aqua!(
        "def g(): i32 = 1;
         def f(): i32 = { g() };"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_lift2() {
    let a = lift(aqua!(
        "def f(): i32 = {
            def f(): i32 = 1;
            f()
        }"
    ))
    .unwrap();
    let b = lift(aqua!(
        "def f_1(): i32 = 1;
         def f(): i32 = { f_1() };"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_lift3() {
    let a = lift(aqua!(
        "def f(): i32 = {
             def g(): i32 = 1;
             g()
         }
         def g(): i32 = 2;"
    ))
    .unwrap();
    println!("{a}");
    // let b = lift(aqua!(
    //     "def g(): i32 = 1;
    //      def f(): i32 = {
    //          g()
    //      }
    //      def g_1(): i32 = 1;"
    // ))
    // .unwrap();
    // check!(a, b);
}

#[test]
fn test_lift4() {
    let a = lift(aqua!(
        "def f(): i32 = {
            def f(): i32 = 1;
            def g(): i32 = 2;
            f()
        }
        def g(): i32 = 3;"
    ))
    .unwrap();
    let b = lift(aqua!(
        "def f_1(): i32 = 1;
         def g(): i32 = 2;
         def f(): i32 = {
            f_1()
        }
        def g_1(): i32 = 3;"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_lift5() {
    let a = lift(aqua!(
        "def f(): i32 = {
            def f(): i32 = {
                def f(): i32 = 1;
                2
            }
            3
        }"
    ))
    .unwrap();
    let b = program([
        stmt_def("f_2", [], [], ty_i32(), [], expr_int("1")),
        stmt_def("f_1", [], [], ty_i32(), [], expr_block([], expr_int("2"))),
        stmt_def("f", [], [], ty_i32(), [], expr_block([], expr_int("3"))),
    ]);
    check!(a, b);
}

#[test]
fn test_lift6() {
    let a = lift(aqua!(
        "def f(): i32 = {
            def f(): i32 = {
                def f(): i32 = 1;
                f()
            }
            f()
        }"
    ))
    .unwrap();
    let b = program([
        stmt_def("f_2", [], [], ty_i32(), [], expr_int("1")),
        stmt_def(
            "f_1",
            [],
            [],
            ty_i32(),
            [],
            expr_block([], expr_call_direct("f_2", [], [])),
        ),
        stmt_def(
            "f",
            [],
            [],
            ty_i32(),
            [],
            expr_block([], expr_call_direct("f_1", [], [])),
        ),
    ]);
    check!(a, b);
}
