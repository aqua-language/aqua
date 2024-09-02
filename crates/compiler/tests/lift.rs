#[macro_use]
mod common;

use common::dsl::expr_block;
use common::dsl::expr_call_direct;
use common::dsl::expr_int;
use common::dsl::program;
use common::dsl::stmt_def;
use common::dsl::types::ty_i32;

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
        "def foo1(): i32 = {
            def foo2(): i32 = 1;
            foo2()
        }"
    ))
    .unwrap();
    let b = program([
        stmt_def("foo2", [], [], ty_i32(), [], expr_int("1")),
        stmt_def(
            "foo1",
            [],
            [],
            ty_i32(),
            [],
            expr_block([], expr_call_direct("foo2", [], [])),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_lift2() {
    let a = lift(aqua!(
        "def foo(): i32 = {
            def foo(): i32 = 1;
            foo()
        }"
    ))
    .unwrap();
    let b = program([
        stmt_def("foo_1", [], [], ty_i32(), [], expr_int("1")),
        stmt_def(
            "foo",
            [],
            [],
            ty_i32(),
            [],
            expr_block([], expr_call_direct("foo_1", [], [])),
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_lift3() {
    let a = lift(aqua!(
        "def f(): i32 = 1;
         def f(): i32 = 1;"
    ))
    .unwrap();
    let b = program([
        stmt_def("f", [], [], ty_i32(), [], expr_int("1")),
        stmt_def("f", [], [], ty_i32(), [], expr_int("1")),
    ]);
    check!(a, b);
}

#[test]
fn test_lift4() {
    let a = lift(aqua!(
        "def f(): i32 = {
            def f(): i32 = 1;
            def f(): i32 = 2;
            f()
        }
        def f(): i32 = 3;"
    ))
    .unwrap();
    let b = program([
        stmt_def("f_2", [], [], ty_i32(), [], expr_int("1")),
        stmt_def("f_2", [], [], ty_i32(), [], expr_int("2")),
        stmt_def(
            "f",
            [],
            [],
            ty_i32(),
            [],
            expr_block([], expr_call_direct("f_2", [], [])),
        ),
        stmt_def("f", [], [], ty_i32(), [], expr_int("3")),
    ]);
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
