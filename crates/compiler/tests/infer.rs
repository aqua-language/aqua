mod common;
use compiler::ast::Program;

#[test]
fn test_infer_literal0() {
    let a = Program::infer("1;").unwrap();
    let b = Program::infer("1:i32;").unwrap();
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
fn test_infer_def8() {
    let a = Program::infer("def f[T](x: T): T = f(x);").unwrap();
    let b = Program::infer("def f[T](x: T): T = (f[T]:fun(T):T)(x:T):T;").unwrap();
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
    assert!(a == b, "{a}\n\n{b}");
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
    assert!(a == b, "{a}\n\n{b}");
}

#[test]
fn test_infer_impl0() {
    let a = Program::infer(
        "trait Foo[T] { def f(x: T): T; }
         impl Foo[i32] { def f(x: i32): i32 = x; }
         f(0);",
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
fn test_infer_impl1() {
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
        "Error: Unsolved goal
            ╭─[test:3:1]
            │
          3 │ f(0);
            │ ┬
            │ ╰── Could not solve goal
         ───╯"
    );
}

#[test]
fn test_infer_impl2() {
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
fn test_infer_impl3() {
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
fn test_infer_impl4() {
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
fn test_infer_impl5() {
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
fn test_infer_impl6() {
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
fn test_infer_add0() {
    let a = Program::infer("1 + 1;").unwrap();
    let b = Program::infer("Add[i32,i32]::add(1:i32, 1:i32);").unwrap();
    check!(a, b);
}
