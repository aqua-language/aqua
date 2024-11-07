use common::passes::monomorphise;

#[macro_use]
mod common;
mod splice;

#[test]
fn test_codegen_int() {
    let aqua = monomorphise("1;").unwrap();
    let rust = "1;";
    let java = "1;";
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_float() {
    let aqua = monomorphise("1.0;").unwrap();
    let rust = "1.0;";
    let java = "1.0;";
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_bool_true() {
    let aqua = monomorphise("true;").unwrap();
    let rust = "true;";
    let java = "true;";
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_bool_false() {
    let aqua = monomorphise("false;").unwrap();
    let rust = "false;";
    let java = "false;";
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_string() {
    let aqua = monomorphise("\"hello\";").unwrap();
    let rust = "\"hello\";";
    let java = "\"hello\";";
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_char() {
    let aqua = monomorphise("'a';").unwrap();
    let rust = "'a';";
    let java = "'a';";
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_var() {
    let aqua = monomorphise("var x = 1;").unwrap();
    let rust = "let x: i32 = 1;";
    let java = "int x = 1;";
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_unused_def() {
    let aqua = monomorphise("def f(x: i32): i32 = x;").unwrap();
    let rust = "";
    let java = "";
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_call() {
    let aqua = monomorphise("def f(x: i32): i32 = x; f(1);").unwrap();
    let rust = indoc::indoc! {
        "fn f(x: i32) -> i32 { x.clone() }
         f(1);"
    };
    let java = indoc::indoc! {
        "int f(int x) { return x; }
         f(1);"
    };
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_block() {
    let aqua = monomorphise("{ }").unwrap();
    let rust = indoc::indoc! {
        "{
             ()
         };"
    };
    let java = indoc::indoc! {
        "((Supplier<Tuple0<>>) () -> {
             return new Tuple0();
         }).get();"
    };
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_if() {
    let aqua = monomorphise("if true { 1 } else { 2 }").unwrap();
    let rust = indoc::indoc! {
        "if true {
             1
         } else {
             2
         };"
    };
    let java = indoc::indoc! {
        "(true) ? ((Supplier<i32>) () -> {
                    return 1;
                }).get() : ((Supplier<i32>) () -> {
                    return 2;
                }).get();"
    };
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}

#[test]
fn test_codegen_arith() {
    let aqua = monomorphise("1 + 2 * 3 / 4 - 5;").unwrap();
    let rust = indoc::indoc! {
        "const Subi32i32sub : fn(i32, i32) -> i32 = |a,b| a-b;
         const Addi32i32add : fn(i32, i32) -> i32 = |a,b| a+b;
         const Divi32i32div : fn(i32, i32) -> i32 = |a,b| a/b;
         const Muli32i32mul : fn(i32, i32) -> i32 = |a,b| a*b;
         Subi32i32sub(Addi32i32add(1, Divi32i32div(Muli32i32mul(2, 3), 4)), 5);"
    };
    let java = indoc::indoc! {
        "static final Function2<Integer, Integer, Integer> Subi32i32sub = (a,b) -> a-b;
         static final Function2<Integer, Integer, Integer> Addi32i32add = (a,b) -> a+b;
         static final Function2<Integer, Integer, Integer> Divi32i32div = (a,b) -> a/b;
         static final Function2<Integer, Integer, Integer> Muli32i32mul = (a,b) -> a*b;
         Subi32i32sub(Addi32i32add(1, Divi32i32div(Muli32i32mul(2, 3), 4)), 5);"
    };
    assert_eq!(aqua.rust().to_string(), rust);
    assert_eq!(aqua.java().to_string(), java);
}
