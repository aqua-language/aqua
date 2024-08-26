#[macro_use]
mod common;

#[test]
fn test_int() {
    let a = monomorphise_program!("1;").unwrap();
    let b = infer_program!("1;").unwrap();
    check!(a, b);
}

#[test]
fn test_float() {
    let a = monomorphise_program!("1.0;").unwrap();
    let b = infer_program!("1.0;").unwrap();
    check!(a, b);
}

#[test]
fn test_monomorphise0() {
    let a = monomorphise_program!(
        "def foo[T](x:T): T = x;
         foo(1);"
    )
    .unwrap();
    let b = infer_program!(
        "def fooi32(x: i32): i32 = x;
         fooi32(1);"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_monomorphise1() {
    let a = monomorphise_program!(
        "def foo[T0,T1](x: T0, y: T1): T0 = x;
         foo(1,2);"
    )
    .unwrap();
    let b = infer_program!(
        "def fooi32i32(x: i32, y: i32): i32 = x;
         fooi32i32(1,2);"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_monomorphise2() {
    let a = monomorphise_program!(
        "trait Foo[T] { def bar(x: T): T; }
         impl Foo[i32] { def bar(x: i32): i32 = 1; } 
         Foo[i32]::bar(1);"
    )
    .unwrap();
    let b = infer_program!(
        "def Fooi32bar(x: i32): i32 = 1;
         Fooi32bar(1);"
    )
    .unwrap();
    check!(a, b);
}

#[test]
#[ignore]
fn test_monomorphise3() {
    let a = monomorphise_program!(
        "trait Foo[T0] { def bar[T1](x: T0, y: T1): T0; }
         impl Foo[i32] { def bar[T1](x: i32, y: T1): i32 = 1; } 
         Foo[i32]::bar(1,2);"
    )
    .unwrap();
    let b = monomorphise_program!(
        "def Fooi32bar[T1](x: i32, y: T1): i32 = 1;
         Fooi32bar(1,2);"
    )
    .unwrap();
    assert_eq!(a, b);
}

#[test]
#[ignore]
fn test_monomorphise4() {
    let a = monomorphise_program!(
        "trait Foo[T] { def foo(x: T): T; }
         impl Foo[i32] { def foo(x: i32): i32 = 1; } 
         1.foo();"
    )
    .unwrap();
    let b = monomorphise_program!(
        "def foo(x: i32): i32 = 1;
         foo(1);"
    )
    .unwrap();
    assert_eq!(a, b);
}
