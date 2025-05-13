use common::dsl::expr_closure;
use common::dsl::expr_int;
use common::dsl::expr_val_place;
use common::dsl::expr_var_place;
use common::dsl::program;
use common::dsl::stmt_expr;
use common::dsl::stmt_var;
use compiler::aqua;
use compiler::mir::Type;

use crate::common::passes::capture;

#[macro_use]
mod common;

#[test]
fn test_capture0() {
    let a = capture(aqua!("(a) => a;")).unwrap();
    let b = program([stmt_expr(expr_closure(["a"], [], expr_val_place("a")))]);
    check!(a, b);
}

#[test]
fn test_capture1() {
    let a = capture(aqua!("var b = 1; (a) => b;")).unwrap();
    let b = program([
        stmt_var("b", Type::Unknown, expr_int("1")),
        stmt_expr(expr_closure(["a"], ["b"], expr_var_place("b"))),
    ]);
    check!(a, b);
}
