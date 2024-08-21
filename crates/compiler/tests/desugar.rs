#[macro_use]
mod common;

use crate::common::dsl::expr_int;
use crate::common::dsl::expr_unit;
use crate::common::dsl::program;
use crate::common::dsl::stmt_expr;
use crate::common::dsl::unresolved::ty;

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
