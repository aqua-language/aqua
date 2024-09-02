#[macro_use]
mod common;

use crate::common::dsl::expr_int;
use crate::common::dsl::expr_unit;
use crate::common::dsl::program;
use crate::common::dsl::stmt_expr;
use crate::common::dsl::unresolved::ty;
use crate::common::passes::desugar;
use crate::common::passes::parse;

#[test]
fn test_desugar_expr_binop_add0() {
    let a = desugar(aqua!("1 + 2;")).unwrap();
    let b = parse(aqua!("Add::add(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_sub0() {
    let a = desugar(aqua!("1 - 2;")).unwrap();
    let b = parse(aqua!("Sub::sub(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_mul0() {
    let a = desugar(aqua!("1 * 2;")).unwrap();
    let b = parse(aqua!("Mul::mul(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_div0() {
    let a = desugar(aqua!("1 / 2;")).unwrap();
    let b = parse(aqua!("Div::div(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_eq0() {
    let a = desugar(aqua!("1 == 2;")).unwrap();
    let b = parse(aqua!("PartialEq::eq(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_ne0() {
    let a = desugar(aqua!("1 != 2;")).unwrap();
    let b = parse(aqua!("PartialEq::ne(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_le0() {
    let a = desugar(aqua!("1 <= 2;")).unwrap();
    let b = parse(aqua!("PartialOrd::le(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_ge0() {
    let a = desugar(aqua!("1 >= 2;")).unwrap();
    let b = parse(aqua!("PartialOrd::ge(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_lt0() {
    let a = desugar(aqua!("1 < 2;")).unwrap();
    let b = parse(aqua!("PartialOrd::lt(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_gt0() {
    let a = desugar(aqua!("1 > 2;")).unwrap();
    let b = parse(aqua!("PartialOrd::gt(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_and0() {
    let a = desugar(aqua!("1 and 2;")).unwrap();
    let b = parse(aqua!("match 1 { true => 2, _ => false };")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_or0() {
    let a = desugar(aqua!("1 or 2;")).unwrap();
    let b = parse(aqua!("match 1 { true => true, _ => 2 };")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_add_mul0() {
    let a = desugar(aqua!("1 + 2 * 3;")).unwrap();
    let b = parse(aqua!("Add::add(1, Mul::mul(2, 3));")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_mul_add0() {
    let a = desugar(aqua!("1 * 2 + 3;")).unwrap();
    let b = parse(aqua!("Add::add(Mul::mul(1, 2), 3);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_add_div0() {
    let a = desugar(aqua!("1 + 2 / 3;")).unwrap();
    let b = parse(aqua!("Add::add(1, Div::div(2, 3));")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_div_add0() {
    let a = desugar(aqua!("1 / 2 + 3;")).unwrap();
    let b = parse(aqua!("Add::add(Div::div(1, 2), 3);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_mul_div0() {
    let a = desugar(aqua!("1 * 2 / 3;")).unwrap();
    let b = parse(aqua!("Div::div(Mul::mul(1, 2), 3);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_add_add0() {
    let a = desugar(aqua!("1 + 2 + 3;")).unwrap();
    let b = parse(aqua!("Add[]::add(Add::add(1, 2), 3);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_eq_eq0() {
    let a = desugar(aqua!("1 == 2 == 3;")).unwrap();
    let b = parse(aqua!("PartialEq::eq(PartialEq::eq(1, 2), 3);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_binop_eq_le0() {
    let a = desugar(aqua!("1 == 2 <= 3;")).unwrap();
    let b = parse(aqua!("PartialOrd::le(PartialEq::eq(1, 2), 3);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_unop_neg0() {
    let a = desugar(aqua!("-1;")).unwrap();
    let b = parse(aqua!("Neg::neg(1);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_unop_neg_add0() {
    let a = desugar(aqua!("-1 + 2;")).unwrap();
    let b = parse(aqua!("Add::add(Neg::neg(1), 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_unop_not0() {
    let a = desugar(aqua!("!1;")).unwrap();
    let b = parse(aqua!("Not::not(1);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_range1() {
    let a = desugar(aqua!("1..2;")).unwrap();
    let b = parse(aqua!("Range::range(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_range2() {
    let a = desugar(aqua!("1..;")).unwrap();
    let b = parse(aqua!("Range::range_from(1);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_range3() {
    let a = desugar(aqua!("..2;")).unwrap();
    let b = parse(aqua!("Range::range_to(None, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_match0() {
    let a = desugar(aqua!("match 1 { 1 => 2, _ => 3 };")).unwrap();
    let b = parse(aqua!("match 1 { 1 => 2, _ => 3 };")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_expr_annotate0() {
    let a = desugar(aqua!("1:i32;")).unwrap();
    let b = program([stmt_expr(expr_int("1").with_type(ty("i32")))]);
    check!(a, b);
}

#[test]
fn test_desugar_program_paren1() {
    let a = desugar(aqua!("(());")).unwrap();
    let b = program([stmt_expr(expr_unit())]);
    check!(a, b);
}

#[test]
fn test_desugar_dot() {
    let a = desugar(aqua!("a.b();")).unwrap();
    let b = parse(aqua!("b(a);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_int_suffix() {
    let a = desugar(aqua!("10s;")).unwrap();
    let b = parse(aqua!("postfix_s(10);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_float_suffix() {
    let a = desugar(aqua!("1.0s;")).unwrap();
    let b = parse(aqua!("postfix_s(1.0);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_anon0() {
    let a = desugar(aqua!("foo(_);")).unwrap();
    let b = parse(aqua!("foo(fun(_0) = _0);")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_anon1() {
    let a = desugar(aqua!("foo(bar(_, _));")).unwrap();
    let b = parse(aqua!("foo(fun(_0, _1) = bar(_0, _1));")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_anon2() {
    let a = desugar(aqua!("foo(_ + _);")).unwrap();
    let b = parse(aqua!("foo(fun(_0, _1) = Add::add(_0, _1));")).unwrap();
    check!(a, b);
}

#[test]
fn test_desugar_anon3() {
    let a = desugar(aqua!("foo(_ + _ + bar(_));")).unwrap();
    let b = parse(aqua!("foo(fun(_0, _1, _2) = Add::add(Add::add(_0, _1), bar(_2)));")).unwrap();
    check!(a, b);
}
