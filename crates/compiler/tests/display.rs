use compiler::util::expr_add_desugared;
use compiler::util::expr_bool;
use compiler::util::expr_div_desugared;
use compiler::util::expr_eq_desugared;
use compiler::util::expr_int;
use compiler::util::expr_mul_desugared;
use compiler::util::expr_ne_desugared;
use compiler::util::expr_string;
use compiler::util::expr_sub_desugared;
use compiler::util::expr_tuple;
use compiler::util::stmt_struct;
use compiler::util::ty;
use compiler::util::ty_fun;
use compiler::util::ty_tuple;

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
    let a = expr_add_desugared(expr_int("1"), expr_int("2")).to_string();
    let b = "Add::add(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop1() {
    let a = expr_sub_desugared(expr_int("1"), expr_int("2")).to_string();
    let b = "Sub::sub(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop2() {
    let a = expr_mul_desugared(expr_int("1"), expr_int("2")).to_string();
    let b = "Mul::mul(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop3() {
    let a = expr_div_desugared(expr_int("1"), expr_int("2")).to_string();
    let b = "Div::div(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop4() {
    let a = expr_eq_desugared(expr_int("1"), expr_int("2")).to_string();
    let b = "PartialEq::eq(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop5() {
    let a = expr_ne_desugared(expr_int("1"), expr_int("2")).to_string();
    let b = "PartialEq::ne(1, 2)";
    assert!(a == b, "{}\n{}", a, b);
}

#[test]
fn test_display_binop6() {
    let a = expr_add_desugared(expr_int("1"), expr_add_desugared(expr_int("2"), expr_int("3"))).to_string();
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
