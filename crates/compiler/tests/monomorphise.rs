use compiler::ast::Expr;
use compiler::ast::Stmt;
use compiler::ast::Type;

use crate::common::dsl::dummy_body;
use crate::common::dsl::expr_call;
use crate::common::dsl::expr_def;
use crate::common::dsl::expr_int;
use crate::common::dsl::program;
use crate::common::dsl::stmt_def;
use crate::common::dsl::stmt_expr;
use crate::common::dsl::ty;
use crate::common::dsl::ty_fun;
use crate::common::passes::infer;
use crate::common::passes::monomorphise;

#[macro_use]
mod common;

#[test]
fn test_int() {
    let a = monomorphise(aqua!("1;")).unwrap();
    let b = infer(aqua!("1;")).unwrap();
    check!(a, b);
}

#[test]
fn test_float() {
    let a = monomorphise(aqua!("1.0;")).unwrap();
    let b = infer(aqua!("1.0;")).unwrap();
    check!(a, b);
}

#[test]
fn test_monomorphise0() {
    let a = monomorphise(aqua!(
        "def foo[T](x:T): T = x;
         foo(1);"
    ))
    .unwrap();
    let b = infer(aqua!(
        "def fooi32(x: i32): i32 = x;
         fooi32(1);"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_monomorphise1() {
    let a = monomorphise(aqua!(
        "def foo[T0,T1](x: T0, y: T1): T0 = x;
         foo(1,2);"
    ))
    .unwrap();
    let b = infer(aqua!(
        "def fooi32i32(x: i32, y: i32): i32 = x;
         fooi32i32(1,2);"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_monomorphise2() {
    let a = monomorphise(aqua!(
        "trait Foo[T] { def bar(x: T): T; }
         impl Foo[i32] { def bar(x: i32): i32 = 1; } 
         Foo[i32]::bar(1);"
    ))
    .unwrap();
    let b = infer(aqua!(
        "def Fooi32bar(x: i32): i32 = 1;
         Fooi32bar(1);"
    ))
    .unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_monomorphise3() {
    let a = monomorphise(aqua!(
        "trait Foo[T0] { def bar[T1](x: T0, y: T1): T0; }
         impl Foo[i32] { def bar[T1](x: i32, y: T1): i32 = 1; } 
         Foo[i32]::bar(1,2);"
    ))
    .unwrap();
    let b = monomorphise(aqua!(
        "def Fooi32bar[T1](x: i32, y: T1): i32 = 1;
         Fooi32bar(1,2);"
    ))
    .unwrap();
    assert_eq!(a, b);
}

#[test]
#[ignore]
fn test_monomorphise4() {
    let a = monomorphise(aqua!(
        "trait Foo[T] { def foo(x: T): T; }
         impl Foo[i32] { def foo(x: i32): i32 = 1; } 
         1.foo();"
    ))
    .unwrap();
    let b = monomorphise(aqua!(
        "def foo(x: i32): i32 = 1;
         foo(1);"
    ))
    .unwrap();
    assert_eq!(a, b);
}

fn op(x: &'static str, t: Type, a: Expr, b: Expr) -> Expr {
    expr_call(
        expr_def(x, []).with_type(ty_fun([t.clone(), t.clone()], t.clone())),
        [a.with_type(t.clone()), b.with_type(t.clone())],
    )
    .with_type(t.clone())
}

fn stmt_def_op(x: &'static str, t: Type) -> Stmt {
    stmt_def(
        x,
        [],
        [("a", t.clone()), ("b", t.clone())],
        t.clone(),
        [],
        dummy_body(),
    )
}

#[test]
fn test_monomorphise_add2() {
    let a = monomorphise(aqua!("1 + 2;")).unwrap();
    let b = program([
        stmt_def_op("Addi32i32add", ty("i32")),
        stmt_expr(op("Addi32i32add", ty("i32"), expr_int("1"), expr_int("2"))),
    ]);
    check!(a, b);
}

#[test]
fn test_monomorphise_add3() {
    let a = monomorphise(aqua!("1 + 2 + 3;")).unwrap();
    let b = program([
        stmt_def_op("Addi32i32add", ty("i32")),
        stmt_expr(op(
            "Addi32i32add",
            ty("i32"),
            op("Addi32i32add", ty("i32"), expr_int("1"), expr_int("2")),
            expr_int("3"),
        )),
    ]);
    check!(a, b);
}

#[test]
fn test_monomorphise_sub2() {
    let a = monomorphise(aqua!("1 - 2 - 3;")).unwrap();
    let b = program([
        stmt_def_op("Subi32i32sub", ty("i32")),
        stmt_expr(op(
            "Subi32i32sub",
            ty("i32"),
            op("Subi32i32sub", ty("i32"), expr_int("1"), expr_int("2")),
            expr_int("3"),
        )),
    ]);
    check!(a, b);
}

#[test]
fn test_monomorphise_add_sub() {
    let a = monomorphise(aqua!("1 + 2 - 3;")).unwrap();
    let b = program([
        stmt_def_op("Subi32i32sub", ty("i32")),
        stmt_def_op("Addi32i32add", ty("i32")),
        stmt_expr(op(
            "Subi32i32sub",
            ty("i32"),
            op("Addi32i32add", ty("i32"), expr_int("1"), expr_int("2")),
            expr_int("3"),
        )),
    ]);
    check!(a, b);
}
