mod common;

use common::passes::parse_expr;
use common::passes::parse_stmt;
use common::passes::parse_type;

macro_rules! check_roundtrip {
    ($a:literal) => {
        check!(parse($a).unwrap().to_string(), $a);
    };
    (@expr; $a:literal) => {
        check!(@value; parse_expr($a).unwrap().to_string(), $a);
    };
    (@type; $a:literal) => {
        check!(@value; parse_type($a).unwrap().to_string(), $a);
    };
    (@stmt; $a:literal) => {
        check!(@value; parse_stmt($a).unwrap().to_string(), $a);
    };
}

#[test]
fn test_display_int0() {
    check_roundtrip!(@expr; "1");
}

#[test]
fn test_display_float1() {
    check_roundtrip!(@expr; "1.0");
}

#[test]
fn test_display_string1() {
    check_roundtrip!(@expr; "\"abc\"");
}

#[test]
fn test_display_string2() {
    check_roundtrip!(@expr; "\"abc\ndef\"");
}

#[test]
fn test_display_bool0() {
    check_roundtrip!(@expr; "true");
}

#[test]
fn test_display_bool1() {
    check_roundtrip!(@expr; "false");
}

#[test]
fn test_display_binop0() {
    check_roundtrip!(@expr; "1 + 2");
}

#[test]
fn test_display_binop1() {
    check_roundtrip!(@expr; "1 - 2");
}

#[test]
fn test_display_binop2() {
    check_roundtrip!(@expr; "1 * 2");
}

#[test]
fn test_display_binop3() {
    check_roundtrip!(@expr; "1 / 2");
}

#[test]
fn test_display_binop4() {
    check_roundtrip!(@expr; "1 == 2");
}

#[test]
fn test_display_binop5() {
    check_roundtrip!(@expr; "1 != 2");
}

#[test]
fn test_display_binop6() {
    check_roundtrip!(@expr; "1 < 2");
}

#[test]
fn test_display_expr_tuple0() {
    check_roundtrip!(@expr; "()");
}

#[test]
#[ignore]
fn test_display_expr_tuple1() {
    check_roundtrip!(@expr; "(1,)");
}

#[test]
fn test_display_expr_tuple2() {
    check_roundtrip!(@expr; "(1, 2)");
}

#[test]
fn test_display_expr_lambda0() {
    check_roundtrip!(@expr; "x: _ => x");
}

#[test]
fn test_display_type_tuple0() {
    check_roundtrip!(@type; "()");
}

#[test]
#[ignore]
fn test_display_type_tuple1() {
    check_roundtrip!(@type; "(i32,)");
}

#[test]
fn test_display_type_tuple2() {
    check_roundtrip!(@type; "(i32, i32)");
}

#[test]
fn test_display_type_lambda0() {
    check_roundtrip!(@type; "i32 => i32");
}

#[test]
fn test_display_stmt_struct0() {
    check_roundtrip!(@stmt; "struct S;");
}

#[test]
fn test_display_stmt_struct1() {
    check_roundtrip!(@stmt; "struct S(x:i32);");
}

#[test]
fn test_display_stmt_struct2() {
    check_roundtrip!(@stmt; "struct S(x:i32, y:i32);");
}
