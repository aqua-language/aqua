#[macro_use]
mod common;

use crate::common::expr_int;
use crate::common::expr_unit;
use crate::common::program;
use crate::common::stmt_expr;
use crate::common::unresolved::ty;

#[test]
fn test_desugar_expr_binop_add0() {
    let a = desugar_program!("1 + 2;").unwrap();
    let b = parse_program!("Add::add(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_sub0() {
    let a = desugar_program!("1 - 2;").unwrap();
    let b = parse_program!("Sub::sub(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_mul0() {
    let a = desugar_program!("1 * 2;").unwrap();
    let b = parse_program!("Mul::mul(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_div0() {
    let a = desugar_program!("1 / 2;").unwrap();
    let b = parse_program!("Div::div(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_eq0() {
    let a = desugar_program!("1 == 2;").unwrap();
    let b = parse_program!("PartialEq::eq(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_ne0() {
    let a = desugar_program!("1 != 2;").unwrap();
    let b = parse_program!("PartialEq::ne(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_le0() {
    let a = desugar_program!("1 <= 2;").unwrap();
    let b = parse_program!("PartialOrd::le(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_ge0() {
    let a = desugar_program!("1 >= 2;").unwrap();
    let b = parse_program!("PartialOrd::ge(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_lt0() {
    let a = desugar_program!("1 < 2;").unwrap();
    let b = parse_program!("PartialOrd::lt(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_gt0() {
    let a = desugar_program!("1 > 2;").unwrap();
    let b = parse_program!("PartialOrd::gt(1, 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_and0() {
    let a = desugar_program!("1 and 2;").unwrap();
    let b = parse_program!("match 1 { true => 2, _ => false };").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_or0() {
    let a = desugar_program!("1 or 2;").unwrap();
    let b = parse_program!("match 1 { true => true, _ => 2 };").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_add_mul0() {
    let a = desugar_program!("1 + 2 * 3;").unwrap();
    let b = parse_program!("Add::add(1, Mul::mul(2, 3));").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_mul_add0() {
    let a = desugar_program!("1 * 2 + 3;").unwrap();
    let b = parse_program!("Add::add(Mul::mul(1, 2), 3);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_add_div0() {
    let a = desugar_program!("1 + 2 / 3;").unwrap();
    let b = parse_program!("Add::add(1, Div::div(2, 3));").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_div_add0() {
    let a = desugar_program!("1 / 2 + 3;").unwrap();
    let b = parse_program!("Add::add(Div::div(1, 2), 3);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_mul_div0() {
    let a = desugar_program!("1 * 2 / 3;").unwrap();
    let b = parse_program!("Div::div(Mul::mul(1, 2), 3);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_add_add0() {
    let a = desugar_program!("1 + 2 + 3;").unwrap();
    let b = parse_program!("Add::add(Add::add(1, 2), 3);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_eq_eq0() {
    let a = desugar_program!("1 == 2 == 3;").unwrap();
    let b = parse_program!("PartialEq::eq(PartialEq::eq(1, 2), 3);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_eq_le0() {
    let a = desugar_program!("1 == 2 <= 3;").unwrap();
    let b = parse_program!("PartialOrd::le(PartialEq::eq(1, 2), 3);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_unop_neg0() {
    let a = desugar_program!("-1;").unwrap();
    let b = parse_program!("Neg::neg(1);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_unop_neg_add0() {
    let a = desugar_program!("-1 + 2;").unwrap();
    let b = parse_program!("Add::add(Neg::neg(1), 2);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_unop_not0() {
    let a = desugar_program!("!1;").unwrap();
    let b = parse_program!("Not::not(1);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_annotate0() {
    let a = desugar_program!("1:i32;").unwrap();
    let b = program([stmt_expr(expr_int("1").with_type(ty("i32")))]);
    check!(a, b);
}

#[test]
fn test_desugar_program_paren1() {
    let a = desugar_program!("(());").unwrap();
    let b = program([stmt_expr(expr_unit())]);
    check!(a, b);
}

#[test]
fn test_desugar_dot() {
    let a = desugar_program!("a.b();").unwrap();
    let b = parse_program!("b(a);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_int_suffix() {
    let a = desugar_program!("10s;").unwrap();
    let b = parse_program!("postfix_s(10);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_float_suffix() {
    let a = desugar_program!("1.0s;").unwrap();
    let b = parse_program!("postfix_s(1.0);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from0() {
    let a = desugar_program!("from x in e;").unwrap();
    let b = desugar_program!("e.map(fun(x) = record(x));").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_where1() {
    let a = desugar_program!("from x in e where x;").unwrap();
    let b = desugar_program!("e.map(fun(x) = record(x)).filter(fun(r) = r.x);").unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_from0() {
    let a = desugar_program!("from x0 in e0 from x1 in e1;").unwrap();
    let b = desugar_program!(
        "e0.map(fun(x0) = record(x0))
           .flatMap(fun(r) = e1.map(fun(x1) = record(x1 = x1, x0 = r.x0)));"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_select() {
    let a = desugar_program!("from x in e select y=f(x);").unwrap();
    let b = desugar_program!(
        "e.map(fun(x) = record(x = x))
          .map(fun(r) = record(y = f(r.x)));"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_group() {
    let a = desugar_program!(
        "from x in e0
         group k=x
            over tumbling(1min)
            compute a = sum of x, b = count of x;"
    )
    .unwrap();
    let b = desugar_program!(
        "e0.map(fun(x) = record(x = x))
           .keyBy(fun(r) = r.x)
           .window(
               tumbling(1min),
               fun(k, r) = record(
                   k = k,
                   a = r.map(fun(r) = r.x).sum(),
                   b = r.map(fun(r) = r.x).count()
               )
           );"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_from_join() {
    let a = desugar_program!(
        "from x in e0
         join y in e1 on x.a == y.b;"
    )
    .unwrap();
    let b = desugar_program!(
        "e0.map(fun(x) = record(x = x))
           .flatMap(fun(r) = e1.filter(fun(y) = r.x.a == y.b)
                               .map(fun(y) = record(y = y, x = r.x)));"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_query_over_compute() {
    let a = desugar_program!(
        "from x in e
         over tumbling(1min)
         compute a = sum of x,
                 b = count of x;"
    )
    .unwrap();
    let b = desugar_program!(
        "e.map(fun(x) = record(x = x))
          .window(
              tumbling(1min),
              fun(r) = record(
                  a = r.map(fun(r) = r.x).sum(),
                  b = r.map(fun(r) = r.x).count()
              )
          );"
    )
    .unwrap();
    check!(a, b);
}
