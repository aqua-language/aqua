mod common;
use common::expr_var;
use common::program;
use common::stmt_def;
use common::ty;
use compiler::ast::Program;

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
