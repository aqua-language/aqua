mod common;
use compiler::ast::Program;

use crate::common::expr_float;
use crate::common::expr_int;
use crate::common::program;
use crate::common::stmt_expr;
use crate::common::ty;

#[test]
fn test_infer_literal_bool0() {
    let a = Program::infer("true;").unwrap();
    let b = Program::infer("true:bool;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_bool1() {
    let a = Program::infer("false;").unwrap();
    let b = Program::infer("false:bool;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_int0() {
    let a = Program::infer("1;").unwrap();
    let b = Program::infer("1:i32;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_int1() {
    let a = Program::infer("1:i64;").unwrap();
    let b = program([stmt_expr(expr_int("1").with_ty(ty("i64")))]);
    check!(a, b);
}

#[test]
fn test_infer_literal_char() {
    let a = Program::infer("'a';").unwrap();
    let b = Program::infer("'a':char;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_float0() {
    let a = Program::infer("1.0;").unwrap();
    let b = Program::infer("1.0:f64;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_float1() {
    let a = Program::infer("1.0:f32;").unwrap();
    let b = program([stmt_expr(expr_float("1.0").with_ty(ty("f32")))]);
    check!(a, b);
}

#[test]
fn test_infer_literal_string() {
    let a = Program::infer("\"foo\";").unwrap();
    let b = Program::infer("\"foo\":String;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def0() {
    let a = Program::infer("def f(): i32 = 0;").unwrap();
    let b = Program::infer("def f(): i32 = 0:i32;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def1() {
    let a = Program::infer("def f(x: i32): i32 = 0;").unwrap();
    let b = Program::infer("def f(x: i32): i32 = 0:i32;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def2() {
    let a = Program::infer(
        "def f(x: i32): i32 = x;
         f(0);",
    )
    .unwrap();
    let b = Program::infer(
        "def f(x: i32): i32 = x:i32;
         (f:fun(i32):i32)(0:i32):i32;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def3() {
    let a = Program::infer("def f(x: i32): f32 = x;").unwrap_err();
    let b = Program::resolve("def f(x: i32): f32 = x:i32;").unwrap();
    check!(
        a,
        b,
        "Error: Type mismatch
            ╭─[test:1:1]
            │
          1 │ def f(x: i32): f32 = x;
            │ ───────────┬─────────┬─
            │            ╰───────────── Expected f32
            │                      │
            │                      ╰─── Found i32
         ───╯"
    )
}

#[test]
fn test_infer_def4() {
    let a = Program::infer("def f[T](x: T): T = x;").unwrap();
    let b = Program::infer("def f[T](x: T): T = x:T;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def5() {
    let a = Program::infer(
        "def f[T](x: T): T = x;
         f(0);",
    )
    .unwrap();
    let b = Program::infer(
        "def f[T](x: T): T = x:T;
         (f[i32]:(fun(i32):i32))(0:i32):i32;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def6() {
    let a = Program::infer(
        "def f[T](x: T): T = x;
         f(0);
         f(0.0);",
    )
    .unwrap();
    let b = Program::infer(
        "def f[T](x: T): T = x:T;
         (f[i32]:(fun(i32):i32))(0:i32):i32;
         (f[f64]:(fun(f64):f64))(0.0:f64):f64;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def7() {
    let a = Program::infer(
        "def f[T](x: T): T = x;
         def g[T](x: T): T = f(x);
         f(0);",
    )
    .unwrap();
    let b = Program::infer(
        "def f[T](x: T): T = x:T;
         def g[T](x: T): T = (f[T]:fun(T):T)(x:T):T;
         (f[i32]:(fun(i32):i32))(0:i32):i32;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def_recursive0() {
    let a = Program::infer("def f[T](x: T): T = f(x);").unwrap();
    let b = Program::infer("def f[T](x: T): T = (f[T]:fun(T):T)(x:T):T;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def_recursive1() {
    let a = Program::infer(
        "def f[T](x: T): T = g(x);
         def g[T](x: T): T = f(x);",
    )
    .unwrap();
    let b = Program::infer(
        "def f[T](x: T): T = (g[T]:fun(T):T)(x:T):T;
         def g[T](x: T): T = (f[T]:fun(T):T)(x:T):T;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_struct0() {
    let a = Program::infer(
        "struct Foo(x:i32);
         Foo(x=0);",
    )
    .unwrap();
    let b = Program::infer(
        "struct Foo(x:i32);
         Foo(x=(0:i32)):Foo;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_struct1() {
    let a = Program::infer(
        "struct Foo[T](x:T);
         Foo(x=0);",
    )
    .unwrap();
    let b = Program::infer(
        "struct Foo[T](x:T);
        Foo[i32](x=(0:i32)):Foo[i32];",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_struct2() {
    let a = Program::infer(
        "Foo(x=0);
         struct Foo[T](x:T);",
    )
    .unwrap();
    let b = Program::infer(
        "Foo[i32](x=(0:i32)):Foo[i32];
         struct Foo[T](x:T);",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_struct3() {
    let a = Program::infer(
        "struct Foo[T](x:T);
         var s = Foo(x=0);
         s.x;",
    )
    .unwrap();
    let b = Program::infer(
        "struct Foo[T](x:T);
         var s:Foo[i32] = Foo[i32](x=(0:i32)):Foo[i32];
         ((s:Foo[i32]).x):i32;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_record0() {
    let a = Program::infer("record();").unwrap();
    let b = Program::infer("record():record();").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_record1() {
    let a = Program::infer("record(x=0);").unwrap();
    let b = Program::infer("record(x=0):record(x:i32);").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_record2() {
    let a = Program::infer("record(x=0, y=1);").unwrap();
    let b = Program::infer("record(x=0, y=1):record(x:i32, y:i32);").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_enum0() {
    let a = Program::infer(
        "enum Foo { Bar(i32), Baz(f32) }
         Foo::Bar(0);",
    )
    .unwrap();
    let b = Program::infer(
        "enum Foo { Bar(i32), Baz(f32) }
         Foo::Bar(0:i32):Foo;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_enum1() {
    let a = Program::infer(
        "enum Foo[T] { Bar(T), Baz(T) }
         Foo::Bar(0);",
    )
    .unwrap();
    let b = Program::infer(
        "enum Foo[T] { Bar(T), Baz(T) }
         Foo[i32]::Bar(0:i32):Foo[i32];",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_assoc1() {
    let a = Program::infer(
        "trait Foo { def f(): i32; }
         impl Foo { def f(): i32 = 1; }
         Foo::f();",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo { def f(): i32; }
         impl Foo { def f(): i32 = 1:i32; }
         ((Foo::f):(fun():i32))():i32;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_assoc2() {
    let a = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x; }
         Foo[i32]::f(0);",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x:i32; }
         ((Foo[i32]::f):(fun(i32):i32))(0:i32):i32;
    ",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_assoc3() {
    let a = Program::infer(
        "trait Foo[T] { def f[U](x:T, y:U): T; }
         impl Foo[i32] { def f[U](x:i32, y:U): i32 = x; }
         Foo[i32]::f[i32](0, 1);",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] { def f[U](x:T, y:U): T; }
         impl Foo[i32] { def f[U](x:i32, y:U): i32 = x:i32; }
         ((Foo[i32]::f[i32]):(fun(i32,i32):i32))(0:i32, 1:i32):i32;
    ",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_unresolved1() {
    let a = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x; }
         f(0);",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x:i32; }
         ((Foo[i32]::f):(fun(i32):i32))(0:i32):i32;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_unresolved2() {
    let a = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[f32] { def f(x: i32): i32 = x; }
         f(0);",
    )
    .unwrap_err();
    let b = Program::resolve(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[f32] { def f(x: i32): i32 = x:i32; }
         ((Foo[i32]::f):(fun(i32):i32))(0:i32):i32;",
    )
    .unwrap();
    check!(
        a,
        b,
        "Error: Trait is not implemented
            ╭─[test:3:1]
            │
          3 │ f(0);
            │ ┬
            │ ╰── Found no implementation for trait
         ───╯"
    );
}

#[test]
fn test_infer_impl_unresolved3() {
    let a = Program::infer(
        "trait Foo[T] {}
         def f[T](x: T): T where Foo[T] = x;",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] {}
         def f[T](x: T): T where Foo[T] = x:T;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_unresolved4() {
    let a = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         def g[T](x: T): T where Foo[T] = g(x);",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         def g[T](x: T): T where Foo[T] = (g[T]:fun(T):T)(x:T):T;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl5() {
    let a = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl[T] Foo[T] { def f(x: T): T = x; }",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl[T] Foo[T] { def f(x: T): T = x:T; }",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl6() {
    let a = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         def g[T](x: T): T where Foo[T] = f(x);",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         def g[T](x: T): T where Foo[T] = (Foo[T]::f:fun(T):T)(x:T):T;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl7() {
    let a = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x; }
         def g[T](x: T): T where Foo[T] = Foo[T]::f(x);",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x:i32; }
         def g[T](x: T): T where Foo[T] = (Foo[T]::f:fun(T):T)(x:T):T;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_i32_assoc() {
    let a = Program::infer(
        "trait Foo[T] { def f(x:T):T; }
         impl Foo[i32] { def f(x:i32):i32 = x; }
         Foo[_]::f(1);
        ",
    )
    .unwrap();
    let b = Program::infer(
        "trait Foo[T] { def f(x:T):T; }
         impl Foo[i32] { def f(x:i32):i32 = x; }
         Foo[i32]::f(1);
        ",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_add() {
    // let a = Program::infer("1 + 2;").unwrap();
    let b = Program::infer("Add[i32,i32]::add(1:i32, 2:i32);").unwrap();
    println!("{}", b.verbose());
    // check!(a, b);
}

#[test]
fn test_infer_desugar_i32_add2() {
    // let a = Program::infer("1 + 2;").unwrap();
    let b = Program::infer("Add[i32,i32]::add(1:i32, 2:i32):i32;").unwrap();
    println!("{}", b.verbose());
    // check!(a, b);
}

#[test]
fn test_infer_desugar_i32_sub() {
    let a = Program::infer("1 - 1;").unwrap();
    let b = Program::infer("Sub[i32,i32]::sub(1:i32, 1:i32):i32;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_mul() {
    let a = Program::infer("1 * 1;").unwrap();
    let b = Program::infer("Mul[i32,i32]::mul(1:i32, 1:i32):i32;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_div() {
    let a = Program::infer("1 / 1;").unwrap();
    let b = Program::infer("Div[i32,i32]::div(1:i32, 1:i32):i32;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_add() {
    let a = Program::infer("1.0 + 1.0;").unwrap();
    let b = Program::infer("Add[f64,f64]::add(1.0:f64, 1.0:f64):f64;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_sub() {
    let a = Program::infer("1.0 - 1.0;").unwrap();
    let b = Program::infer("Sub[f64,f64]::sub(1.0:f64, 1.0:f64):f64;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_mul() {
    let a = Program::infer("1.0 * 1.0;").unwrap();
    let b = Program::infer("Mul[f64,f64]::mul(1.0:f64, 1.0:f64):f64;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_div() {
    let a = Program::infer("1.0 / 1.0;").unwrap();
    let b = Program::infer("Div[f64,f64]::div(1.0:f64, 1.0:f64):f64;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_i32_add() {
    let a = Program::infer("1.0 + 1;").unwrap();
    let b = Program::infer("Add[f64,i32]::add(1.0:f64, 1:i32):f64;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_f64_add() {
    let a = Program::infer("1 + 1.0;").unwrap();
    let b = Program::infer("Add[i32,f64]::add(1:i32, 1.0:f64):f64;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_neg() {
    let a = Program::infer("-1;").unwrap();
    let b = Program::infer("Neg[i32]::neg(1:i32):i32;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_eq() {
    let a = Program::infer("1 == 2;").unwrap();
    let b = Program::infer("PartialEq[i32,i32]::eq(1:i32, 2:i32):bool;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_i32_abs() {
    let a = Program::infer("1.abs();").unwrap();
    let b = Program::infer("i32::abs(1:i32):i32;").unwrap();
    check!(a, b);
}

#[test]
fn test_infer_i32_postfix() {
    let a = Program::infer(
        "def postfix_min(x: i32): i32 = x * 60;
        1min;",
    )
    .unwrap();
    let b = Program::infer(
        "def postfix_min(x: i32): i32 = Mul[i32,i32]::mul(x:i32, 60:i32);
        postfix_min(1:i32):i32;",
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_for_loop() {
    let a = Program::infer(
        "for i in 0..10 {
            i;
        }",
    )
    .unwrap();
    let b = Program::infer(
        "for i in IntoIterator[Item=i32,IntoIter=]::into_iter(Range[i32]::range(0:i32, 10:i32) {
            i:i32;
        }",
    )
    .unwrap();
    check!(a, b);
}
