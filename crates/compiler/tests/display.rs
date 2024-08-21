mod common;

use common::dsl::expr_bool;
use common::dsl::expr_int;
use common::dsl::expr_string;
use common::dsl::expr_tuple;
use common::dsl::stmt_struct;
use common::dsl::ty;
use common::dsl::ty_fun;
use common::dsl::ty_tuple;
use common::dsl::desugared::expr_add;
use common::dsl::desugared::expr_div;
use common::dsl::desugared::expr_eq;
use common::dsl::desugared::expr_mul;
use common::dsl::desugared::expr_ne;
use common::dsl::desugared::expr_sub;

#[test]
fn test_display_int0() {
    let a = expr_int("1").to_string();
    let b = "1";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_float1() {
    let a = expr_int("1.0").to_string();
    let b = "1.0";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_string1() {
    let a = expr_string("abc").to_string();
    let b = "\"abc\"";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_string2() {
    let a = expr_string("abc\ndef").to_string();
    let b = "\"abc\ndef\"";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_bool0() {
    let a = expr_bool(true).to_string();
    let b = "true";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_bool1() {
    let a = expr_bool(false).to_string();
    let b = "false";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop0() {
    let a = expr_add(expr_int("1"), expr_int("2")).to_string();
    let b = "Add::add(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop1() {
    let a = expr_sub(expr_int("1"), expr_int("2")).to_string();
    let b = "Sub::sub(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop2() {
    let a = expr_mul(expr_int("1"), expr_int("2")).to_string();
    let b = "Mul::mul(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop3() {
    let a = expr_div(expr_int("1"), expr_int("2")).to_string();
    let b = "Div::div(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop4() {
    let a = expr_eq(expr_int("1"), expr_int("2")).to_string();
    let b = "PartialEq::eq(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop5() {
    let a = expr_ne(expr_int("1"), expr_int("2")).to_string();
    let b = "PartialEq::ne(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop6() {
    let a = expr_add(expr_int("1"), expr_add(expr_int("2"), expr_int("3"))).to_string();
    let b = "Add::add(1, Add::add(2, 3))";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_expr_tuple0() {
    let a = expr_tuple([]).to_string();
    let b = "()";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_expr_tuple1() {
    let a = expr_tuple([expr_int("1")]).to_string();
    let b = "(1,)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_expr_tuple2() {
    let a = expr_tuple([expr_int("1"), expr_int("2")]).to_string();
    let b = "(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_type_tuple0() {
    let a = ty_tuple([]).to_string();
    let b = "()";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_type_tuple1() {
    let a = ty_tuple([ty("i32")]).to_string();
    let b = "(i32,)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_type_tuple2() {
    let a = ty_tuple([ty("i32"), ty("i32")]).to_string();
    let b = "(i32, i32)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_type_fun0() {
    let a = ty_fun([ty("i32")], ty("i32")).to_string();
    let b = "fun(i32): i32";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_stmt_struct0() {
    let a = stmt_struct("S", [], []).to_string();
    let b = "struct S;";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_stmt_struct1() {
    let a = stmt_struct("S", [], [("x", ty("i32"))]).to_string();
    let b = "struct S(x:i32);";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_stmt_struct2() {
    let a = stmt_struct("S", [], [("x", ty("i32")), ("y", ty("i32"))]).to_string();
    let b = "struct S(x:i32, y:i32);";
    assert!(a == b, "{}\n{}", a, b);
}
