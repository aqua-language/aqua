#[macro_use]
mod common;

use compiler::ast::Type;

use crate::common::dsl::expr_field;
use crate::common::dsl::expr_float;
use crate::common::dsl::expr_int;
use crate::common::dsl::expr_record;
use crate::common::dsl::program;
use crate::common::dsl::stmt_expr;
use crate::common::dsl::ty;
use crate::common::dsl::ty_record;
use crate::common::passes::infer;
use crate::common::passes::resolve;

#[test]
fn test_infer_literal_bool0() {
    let a = infer(aqua!("true;")).unwrap();
    let b = infer(aqua!("true:bool;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_bool1() {
    let a = infer(aqua!("false;")).unwrap();
    let b = infer(aqua!("false:bool;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_int0() {
    let a = infer(aqua!("1;")).unwrap();
    let b = infer(aqua!("1:i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_int1() {
    let a = infer(aqua!("1:i64;")).unwrap();
    let b = program([stmt_expr(expr_int("1").with_type(ty("i64")))]);
    check!(a, b);
}

#[test]
fn test_infer_literal_char() {
    let a = infer(aqua!("'a';")).unwrap();
    let b = infer(aqua!("'a':char;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_float0() {
    let a = infer(aqua!("1.0;")).unwrap();
    let b = infer(aqua!("1.0:f64;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_literal_float1() {
    let a = infer(aqua!("1.0:f32;")).unwrap();
    let b = program([stmt_expr(expr_float("1.0").with_type(ty("f32")))]);
    check!(a, b);
}

#[test]
fn test_infer_literal_string() {
    let a = infer(aqua!("\"foo\";")).unwrap();
    let b = infer(aqua!("\"foo\":String;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def0() {
    let a = infer(aqua!("def f(): i32 = 0;")).unwrap();
    let b = infer(aqua!("def f(): i32 = 0:i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def1() {
    let a = infer(aqua!("def f(x: i32): i32 = 0;")).unwrap();
    let b = infer(aqua!("def f(x: i32): i32 = 0:i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def2() {
    let a = infer(aqua!(
        "def f(x: i32): i32 = x;
         f(0);"
    )).unwrap();
    let b = infer(aqua!(
        "def f(x: i32): i32 = x:i32;
         (f:fun(i32):i32)(0:i32):i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def3() {
    let a = infer(aqua!("def f(x: i32): f32 = x;")).unwrap_err();
    let b = resolve(aqua!("def f(x: i32): f32 = x:i32;")).unwrap();
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
    let a = infer(aqua!("def f[T](x: T): T = x;")).unwrap();
    let b = infer(aqua!("def f[T](x: T): T = x:T;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def5() {
    let a = infer(aqua!(
        "def f[T](x: T): T = x;
         f(0);"
    )).unwrap();
    let b = infer(aqua!(
        "def f[T](x: T): T = x:T;
         (f[i32]:(fun(i32):i32))(0:i32):i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def6() {
    let a = infer(aqua!(
        "def f[T](x: T): T = x;
         f(0);
         f(0.0);"
    )).unwrap();
    let b = infer(aqua!(
        "def f[T](x: T): T = x:T;
         (f[i32]:(fun(i32):i32))(0:i32):i32;
         (f[f64]:(fun(f64):f64))(0.0:f64):f64;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def7() {
    let _a = infer(aqua!(
        "def f[T](x: T): T = x;
         def g[S](x: S): S = f(x);
         f(0);"
    )).unwrap();
    // let b = infer(aqua!(
    //     "def f[T](x: T): T = x:T;
    //      def g[T](x: T): T = (f[T]:fun(T):T)(x:T):T;
    //      (f[i32]:(fun(i32):i32))(0:i32):i32;"
    // )
    // .unwrap();
    // check!(a, b);
}

#[test]
fn test_infer_def_recursive0() {
    let a = infer(aqua!("def f[T](x: T): T = f(x);")).unwrap();
    let b = infer(aqua!("def f[T](x: T): T = (f[T]:fun(T):T)(x:T):T;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_def_recursive1() {
    let a = infer(aqua!(
        "def f[T](x: T): T = g(x);
         def g[T](x: T): T = f(x);"
    )).unwrap();
    let b = infer(aqua!(
        "def f[T](x: T): T = (g[T]:fun(T):T)(x:T):T;
         def g[T](x: T): T = (f[T]:fun(T):T)(x:T):T;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_struct0() {
    let a = infer(aqua!(
        "struct Foo(x:i32);
         Foo(x=0);"
    )).unwrap();
    let b = infer(aqua!(
        "struct Foo(x:i32);
         Foo(x=(0:i32)):Foo;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_struct1() {
    let a = infer(aqua!(
        "struct Foo[T](x:T);
         Foo(x=0);"
    )).unwrap();
    let b = infer(aqua!(
        "struct Foo[T](x:T);
        Foo[i32](x=(0:i32)):Foo[i32];"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_struct2() {
    let a = infer(aqua!(
        "Foo(x=0);
         struct Foo[T](x:T);"
    )).unwrap();
    let b = infer(aqua!(
        "Foo[i32](x=(0:i32)):Foo[i32];
         struct Foo[T](x:T);"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_struct3() {
    let a = infer(aqua!(
        "struct Foo[T](x:T);
         var s = Foo(x=0);
         s.x;"
    )).unwrap();
    let b = infer(aqua!(
        "struct Foo[T](x:T);
         var s:Foo[i32] = Foo[i32](x=(0:i32)):Foo[i32];
         ((s:Foo[i32]).x):i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_record0() {
    let a = infer(aqua!("record();")).unwrap();
    let b = infer(aqua!("record():record();")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_record1() {
    let a = infer(aqua!("record(x=0);")).unwrap();
    let b = infer(aqua!("record(x=0):record(x:i32);")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_record2() {
    let a = infer(aqua!("record(x=0, y=1);")).unwrap();
    let b = infer(aqua!("record(x=0, y=1):record(x:i32, y:i32);")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_record3() {
    let a = infer(aqua!("record(x=0, y=1).x;")).unwrap();
    let b = infer(aqua!("record(x=0, y=1).x:i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_record4() {
    let a = infer(aqua!("record(x=0, y=1).z;")).unwrap_err();
    let b = program([stmt_expr(
        expr_field(
            expr_record([
                ("x", expr_int("0").with_type(ty("i32"))),
                ("y", expr_int("1").with_type(ty("i32"))),
            ])
            .with_type(ty_record([("x", ty("i32")), ("y", ty("i32"))])),
            "z",
        )
        .with_type(Type::Err),
    )]);
    check!(
        a,
        b,
        "Error: Unknown field
           ╭─[test:1:1]
           │
         1 │ record(x=0, y=1).z;
           │ ─────────┬────────
           │          ╰────────── Field z not found in record record(x:'2, y:'3)
        ───╯"
    );
}

#[test]
fn test_infer_enum0() {
    let a = infer(aqua!(
        "enum Foo { Bar(i32), Baz(f32) }
         Foo::Bar(0);"
    )).unwrap();
    let b = infer(aqua!(
        "enum Foo { Bar(i32), Baz(f32) }
         Foo::Bar(0:i32):Foo;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_enum1() {
    let a = infer(aqua!(
        "enum Foo[T] { Bar(T), Baz(T) }
         Foo::Bar(0);"
    )).unwrap();
    let b = infer(aqua!(
        "enum Foo[T] { Bar(T), Baz(T) }
         Foo[i32]::Bar(0:i32):Foo[i32];"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_assoc0() {
    let a = infer(aqua!(
        "trait Foo { def f(): i32; }
         impl Foo { def f(): i32 = 1; }
         Foo::f();"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo { def f(): i32; }
         impl Foo { def f(): i32 = 1:i32; }
         ((Foo::f):(fun():i32))():i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_assoc1() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x; }
         Foo[i32]::f(0);"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x:i32; }
         ((Foo[i32]::f):(fun(i32):i32))(0:i32):i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_infer_impl_assoc2() {
    let a = infer(aqua!(
        "trait Foo[T] { def f[U](x:T, y:U): T; }
         impl Foo[i32] { def f[U](x:i32, y:U): i32 = x; }
         Foo[i32]::f[i32](0, 1);"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] { def f[U](x:T, y:U): T; }
         impl Foo[i32] { def f[U](x:i32, y:U): i32 = x:i32; }
         ((Foo[i32]::f[i32]):(fun(i32,i32):i32))(0:i32, 1:i32):i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_infer_impl_def_generic0() {
    let a = infer(aqua!(
        "trait Foo { def f[T](x: T): T; }
         impl Foo { def f[T](x: T): T = x; }
         Foo::f(1);"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo { def f[T](x: T): T; }
         impl Foo { def f[T](x: T): T = x:T; }
         ((Foo::f[i32]):(fun(i32):i32))(1:i32):i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_infer_impl_unresolved1() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x; }
         f(0);"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x:i32; }
         ((Foo[i32]::f):(fun(i32):i32))(0:i32):i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_infer_impl_unresolved2() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[f32] { def f(x: i32): i32 = x; }
         f(0);"
    )).unwrap_err();
    let b = resolve(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[f32] { def f(x: i32): i32 = x:i32; }
         ((Foo[i32]::f):(fun(i32):i32))(0:i32):i32;",
    )).unwrap();
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
fn test_infer_impl_where_bound3() {
    let a = infer(aqua!(
        "trait Foo[T] {}
         def f[T](x: T): T where Foo[T] = x;"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] {}
         def f[T](x: T): T where Foo[T] = x:T;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_where_bound4() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         def g[T](x: T): T where Foo[T] = g(x);"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         def g[T](x: T): T where Foo[T] = (g[T]:fun(T):T)(x:T):T;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_trait0() {
    let r = infer(aqua!("trait Foo { def f(): i32; }"));
    assert!(r.is_ok());
}

#[test]
fn test_infer_trait1() {
    let r = infer(aqua!("trait Foo[T] { def f(): T; }"));
    assert!(r.is_ok());
}

#[test]
fn test_infer_trait2() {
    let r = infer(aqua!("trait Foo[T] where Foo[T] { def f(x: T): T; }"));
    assert!(r.is_ok());
}

#[test]
fn test_infer_trait3() {
    let r = infer(aqua!(
        "trait Foo[T] { }
         trait Bar[T] where Foo[T] { }"
    ));
    assert!(r.is_ok());
}

#[test]
fn test_infer_trait4() {
    let r = infer(aqua!(
        "trait Foo[T] { }
         trait Bar[T] where Foo[T] { }"
    ));
    assert!(r.is_ok());
}

#[test]
fn test_infer_trait5() {
    let r = infer(aqua!(
        "trait Foo[T] {
            def f[S](x: T): T where Foo[S];
         }"
    ));
    assert!(r.is_ok());
}

#[test]
fn test_infer_trait_impl0() {
    let r = infer(aqua!(
        "trait Foo[T] {
             def f(x:T):T;
         }
         impl Foo[i32] {
             def f(x:i32):i32 = 1;
         }"
    ));
    assert!(r.is_ok());
}

#[test]
#[ignore]
fn test_infer_trait_impl1() {
    let r = infer(aqua!(
        "trait Foo[T] {
             def f(x:T):T;
         }
         impl Foo[i32] {
             def f(x:i32):f32 = 1.0;
         }"
    )).unwrap_err();
    check!(r, "todo");
}

#[test]
fn test_infer_trait_impl_impl0() {
    let r = infer(aqua!(
        "trait Foo[T] { }
         impl Foo[i32] { }
         impl Foo[f32] { }"
    ));
    assert!(r.is_ok());
}

#[test]
#[ignore]
fn test_infer_trait_impl_impl1() {
    let a = infer(aqua!(
        "trait Foo[T] { }
         impl Foo[i32] { }
         impl Foo[i32] { }"
    )).unwrap_err();
    check!(a, "todo");
}

#[test]
#[ignore]
fn test_infer_trait_impl_impl2() {
    let a = infer(aqua!(
        "trait Foo[T] { }
         impl Foo[Vec[i32]] { }
         impl Foo[Vec[f32]] { }"
    )).unwrap_err();
    check!(a, "todo");
}

#[test]
fn test_infer_trait_blanket_impl0() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x:T):T; }
         impl[T] Foo[T] { def f(x:T):T = x; }"
    ));
    assert!(a.is_ok());
}

#[test]
fn test_infer_trait_impl3() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl[T] Foo[T] { def f(x: T): T = x; }"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl[T] Foo[T] { def f(x: T): T = x:T; }"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_trait_def0() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x; }
         def g[T](x: T): T where Foo[T] = Foo[T]::f(x);"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x:i32; }
         def g[T](x: T): T where Foo[T] = (Foo[T]::f:fun(T):T)(x:T):T;"
    )).unwrap();
    check!(a, b);
}

#[test]
#[ignore = "TODO: Fix this"]
fn test_infer_trait_def1() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         def g[T](x: T): T where Foo[T] = f(x);"
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] { def f(x: T): T; }
         def g[T](x: T): T where Foo[T] = (Foo[T]::f:fun(T):T)(x:T):T;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_impl_i32_assoc() {
    let a = infer(aqua!(
        "trait Foo[T] { def f(x:T):T; }
         impl Foo[i32] { def f(x:i32):i32 = x; }
         Foo[_]::f(1);
        "
    )).unwrap();
    let b = infer(aqua!(
        "trait Foo[T] { def f(x:T):T; }
         impl Foo[i32] { def f(x:i32):i32 = x; }
         Foo[i32]::f(1);
        "
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_add() {
    let a = infer(aqua!("1 + 2;")).unwrap();
    let b = infer(aqua!("Add[i32,i32]::add(1:i32, 2:i32):i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_sub() {
    let a = infer(aqua!("1 - 1;")).unwrap();
    let b = infer(aqua!("Sub[i32,i32]::sub(1:i32, 1:i32):i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_mul() {
    let a = infer(aqua!("1 * 1;")).unwrap();
    let b = infer(aqua!("Mul[i32,i32]::mul(1:i32, 1:i32):i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_div() {
    let a = infer(aqua!("1 / 1;")).unwrap();
    let b = infer(aqua!("Div[i32,i32]::div(1:i32, 1:i32):i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_add() {
    let a = infer(aqua!("1.0 + 1.0;")).unwrap();
    let b = infer(aqua!("Add[f64,f64]::add(1.0:f64, 1.0:f64):f64;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_sub() {
    let a = infer(aqua!("1.0 - 1.0;")).unwrap();
    let b = infer(aqua!("Sub[f64,f64]::sub(1.0:f64, 1.0:f64):f64;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_mul() {
    let a = infer(aqua!("1.0 * 1.0;")).unwrap();
    let b = infer(aqua!("Mul[f64,f64]::mul(1.0:f64, 1.0:f64):f64;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_div() {
    let a = infer(aqua!("1.0 / 1.0;")).unwrap();
    let b = infer(aqua!("Div[f64,f64]::div(1.0:f64, 1.0:f64):f64;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_f64_i32_add() {
    let a = infer(aqua!("1.0 + 1;")).unwrap();
    let b = infer(aqua!("Add[f64,i32]::add(1.0:f64, 1:i32):f64;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_f64_add() {
    let a = infer(aqua!("1 + 1.0;")).unwrap();
    let b = infer(aqua!("Add[i32,f64]::add(1:i32, 1.0:f64):f64;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_neg() {
    let a = infer(aqua!("-1;")).unwrap();
    let b = infer(aqua!("Neg[i32]::neg(1:i32):i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_desugar_i32_eq() {
    let a = infer(aqua!("1 == 2;")).unwrap();
    let b = infer(aqua!("PartialEq[i32,i32]::eq(1:i32, 2:i32):bool;")).unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_infer_i32_abs() {
    let a = infer(aqua!("1.abs();")).unwrap();
    let b = infer(aqua!("i32::abs(1:i32):i32;")).unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_infer_i32_postfix() {
    let a = infer(aqua!(
        "def postfix_min(x: i32): i32 = x * 60;
        1min;"
    )).unwrap();
    let b = infer(aqua!(
        "def postfix_min(x: i32): i32 = Mul[i32,i32]::mul(x:i32, 60:i32);
        postfix_min(1:i32):i32;"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_i32_add2() {
    let a = infer(aqua!("1 + 2;")).unwrap();
    let b = infer(aqua!("Add[i32,i32]::add(1:i32, 2:i32):i32;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_i32_add3() {
    let a = infer(aqua!("1 + 2 + 3;")).unwrap();
    let b = infer(aqua!("Add[i32,i32]::add(Add[i32,i32]::add(1:i32, 2:i32):i32, 3:i32):i32;"))
        .unwrap();
    check!(a, b);
}

#[test]
fn test_infer_i32_lt() {
    let a = infer(aqua!("1 < 2;")).unwrap();
    let b = infer(aqua!("PartialOrd[i32,i32]::lt(1:i32, 2:i32):bool;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_i32_gt() {
    let a = infer(aqua!("1 > 2;")).unwrap();
    let b = infer(aqua!("PartialOrd[i32,i32]::gt(1:i32, 2:i32):bool;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_i32_le() {
    let a = infer(aqua!("1 <= 2;")).unwrap();
    let b = infer(aqua!("PartialOrd[i32,i32]::le(1:i32, 2:i32):bool;")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_i32_ge() {
    let a = infer(aqua!("1 >= 2;")).unwrap();
    let b = infer(aqua!("PartialOrd[i32,i32]::ge(1:i32, 2:i32):bool;")).unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_infer_for() {
    let a = infer(aqua!(
        "for i in 0..10 {
            i;
        }"
    )).unwrap();
    let b = infer(aqua!(
        "for i in IntoIterator[Item=i32,IntoIter=]::into_iter(Range[i32]::range(0:i32, 10:i32) {
            i:i32;
        }"
    )).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_while0() {
    let a = infer(aqua!("while false { }")).unwrap();
    let b = infer(aqua!("(while false:bool { }):();")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_while1() {
    let a = infer(aqua!("while 1 < 2 { }")).unwrap();
    let b = infer(aqua!("(while PartialOrd[i32,i32]::lt(1:i32, 2:i32):bool { }):();")).unwrap();
    check!(a, b);
}

#[test]
fn test_infer_while2() {
    let a = infer(aqua!(
        "var x = 0;
         while x < 10 { x = x + 1; };
         x;"
    )).unwrap();
    let b = infer(aqua!(
        "var x:i32 = 0:i32;
         (while PartialOrd[i32,i32]::lt((x:i32), 10:i32):bool {
             x:i32 = Add[i32,i32]::add((x:i32), 1:i32):i32;
         }):();
         x:i32;"
    )).unwrap();
    check!(a, b);
}
