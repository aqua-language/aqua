use common::passes::codegen;

#[macro_use]
mod common;

#[test]
fn test_codegen_int() {
    let a = codegen("1;");
    let b = "1;";
    assert_eq!(a, b);
}

#[test]
fn test_codegen_float() {
    let a = codegen("1.0;");
    let b = "1.0;";
    assert_eq!(a, b);
}

#[test]
fn test_codegen_bool_true() {
    let a = codegen("true;");
    let b = "true;";
    assert_eq!(a, b);
}

#[test]
fn test_codegen_bool_false() {
    let a = codegen("false;");
    let b = "false;";
    assert_eq!(a, b);
}

#[test]
fn test_codegen_string() {
    let a = codegen("\"hello\";");
    let b = "\"hello\";";
    assert_eq!(a, b);
}

#[test]
fn test_codegen_char() {
    let a = codegen("'a';");
    let b = "'a';";
    assert_eq!(a, b);
}

#[test]
fn test_codegen_var() {
    let a = codegen("var x = 1;");
    let b = "let x: i32 = 1;";
    assert_eq!(a, b);
}

#[test]
fn test_codegen_def() {
    let a = codegen("def f(x: i32): i32 = x;");
    let b = "fn f(x: i32) -> i32 { x }";
    assert_eq!(a, b);
}

#[test]
fn test_codegen_call() {
    let a = codegen("def f(x: i32): i32 = x; f(1);");
    let b = indoc::indoc! {
        "fn f(x: i32) -> i32 { x }
         f(1);"
    };
    assert_eq!(a, b);
}

#[test]
fn test_codegen_if() {
    let a = codegen("if true { 1 } else { 2 }");
    let b = indoc::indoc! {
        "if true {
             1
         } else {
             2
         };"
    };
    assert_eq!(a, b);
}

#[test]
fn test_codegen_arith() {
    let a = codegen("1 + 2 * 3 / 4 - 5;");
    let b = indoc::indoc! {
        "1 + 2 * 3 / 4 - 5;"
    };
    assert_eq!(a, b);
}
