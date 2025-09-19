use common::dsl::expr_closure;
use common::dsl::expr_int;
use common::dsl::expr_var_place;
use common::dsl::program;
use common::dsl::stmt_expr;
use common::dsl::stmt_var;
use common::passes::expand;
use compiler::aqua;
use compiler::mir::Type;

use crate::common::passes::capture;

#[macro_use]
mod common;

#[test]
fn test_capture0() {
    let a = capture(aqua!("(a) => a;")).unwrap();
    let b = expand(aqua!(
        "C0;
         struct C0;
         impl Fn[C0, (_,), _] {
             def call(env: C0, args: (_,)): _ = {
                 val a = args.0;
                 a
             }
         }
         "
    ))
    .unwrap();
    check!(a, b);
}

#[test]
fn test_capture1() {
    let a = capture(aqua!("var b = 1; (a) => b;")).unwrap();
    let b = program([
        stmt_var("b", Type::Unknown, expr_int("1")),
        stmt_expr(expr_closure("C0", ["a"], ["b"], expr_var_place("b"))),
    ]);
    check!(a, b);
}
