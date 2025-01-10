use common::dsl::expr_closure;
use common::dsl::expr_val_place;
use common::dsl::program;
use common::dsl::stmt_expr;
use compiler::aqua;

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
    let a = capture(aqua!("val b = 0; (a) => a + b;")).unwrap();
    let b = capture(aqua!("val b = 0; (a|b) => a + b;")).unwrap();
    check!(a, b);
}
