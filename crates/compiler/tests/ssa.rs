#[macro_use]
mod common;

#[test]
fn test_flatten0() {
    let a = flatten_program!("1+2;").unwrap();
    let b = resolve_program!(
        "var x1 = Add[_, _]::add;
         var x2 = 1;
         var x3 = 2;
         var x4 = x1(x2, x3);"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_flatten1() {
    let a = flatten_program!("var x = 1; var y = x;").unwrap();
    let b = resolve_program!("var x1 = 1;").unwrap();
    check!(a, b);
}

#[test]
fn test_flatten2() {
    let a = flatten_program!("{ 1 }").unwrap();
    let b = resolve_program!(
        "var x1 =
         let x2 = 1 in
         1;"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_flatten3() {
    let _a = flatten_program!("var x = 1; { x }").unwrap();
    let _b = resolve_program!("var x1 = 1;").unwrap();
}
