use compiler::ast::Program;
use compiler::check;
use compiler::dsl::expr_int;
use compiler::dsl::program;
use compiler::dsl::stmt_def;
use compiler::dsl::types::ty_i32;

#[test]
fn test_lift0() {
    let a = Program::lift("def g(): i32 = 1;").unwrap();
    let b = program([stmt_def("g", [], [], ty_i32(), [], expr_int("1"))]);
    check!(a, b);
}

#[test]
fn test_lift1() {
    let a = Program::lift(
        "def foo1(): i32 = {
            def foo2(): i32 = 1;
            foo2()
        }",
    )
    .unwrap();
    let b = program([stmt_def("g", [], [], ty_i32(), [], expr_int("1"))]);
    check!(a, b);
}

#[test]
fn test_lift2() {
    let a = Program::lift(
        "def foo(): i32 = {
            def foo(): i32 = 1;
            foo()
        }",
    )
    .unwrap();
    let b = program([stmt_def("g", [], [], ty_i32(), [], expr_int("1"))]);
    check!(a, b);
}

#[test]
fn test_lift3() {
    let a = Program::lift(
        "def foo(): i32 = 1;
            def foo(): i32 = 1;
            foo()
        }",
    )
    .unwrap();
    let b = program([stmt_def("g", [], [], ty_i32(), [], expr_int("1"))]);
    check!(a, b);
}
