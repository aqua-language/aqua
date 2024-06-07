mod common;
use common::expr_var;
use common::program;
use common::stmt_def;
use common::ty;
use compiler::ast::Program;

#[ignore]
#[test]
fn test_monomorphise0() {
    let a = Program::monomorphise(
        "def foo[T](x:T): T = x;
         foo(1);",
    )
    .unwrap();
    let b = program([stmt_def(
        "foo",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_var("x"),
    )]);
    check!(a, b);
}

#[test]
fn test_monomorphise1() {
    let a = Program::monomorphise(
        "trait Foo[T] { def foo(x: T): T; }
         impl Foo[i32] { def foo(x: i32): i32 = 1; } 
         1.foo();",
    )
    .unwrap();
    let b = Program::monomorphise(
        "def foo(x: i32): i32 = 1;
         foo(1);",
    )
    .unwrap();
    assert_eq!(a, b);
}
