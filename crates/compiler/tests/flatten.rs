#[macro_use]
mod common;

#[test]
fn test_flatten0() {
    let a = flatten_program!("1+2;").unwrap();
    let b = resolve_program!(
        "var x0 = Add[_, _]::add;
         var x1 = 1;
         var x2 = 2;
         var x3 = x0(x1, x2);"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_flatten1() {
    let a = flatten_program!("var x = 1; var y = x;").unwrap();
    let b = resolve_program!("var x0 = 1;").unwrap();
    check!(a, b);
}

#[test]
fn test_flatten2() {
    let a = flatten_program!(
        "var x = 1;
         var y = x;
         var z = y + x;"
    )
    .unwrap();
    let b = resolve_program!(
        "var x0 = 1;
         var x1 = Add[_, _]::add;
         var x2 = x1(x0, x0);"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_flatten3() {
    let a = flatten_program!("{ 1 }").unwrap();
    let b = resolve_program!("var x0 = 1;").unwrap();
    check!(a, b);
}

#[test]
fn test_flatten4() {
    let a = flatten_program!("var x = 1; { x }").unwrap();
    let b = resolve_program!("var x0 = 1;").unwrap();
    check!(a, b);
}

#[test]
fn test_flatten5() {
    let a = flatten_program!("var x = 1; { var y = x; }").unwrap();
    let b = resolve_program!("var x0 = 1;").unwrap();
    check!(a, b);
}

#[test]
fn test_flatten6() {
    let a = flatten_program!("var x = 1; var y = x + 2;").unwrap();
    let b = resolve_program!(
        "var x0 = 1;
         var x1 = Add[_, _]::add;
         var x2 = 2;
         var x3 = x1(x0, x2);"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_flatten7() {
    let a = flatten_program!("var x = { var y = 1; y };").unwrap();
    let b = resolve_program!("var x0 = 1;").unwrap();
    check!(a, b);
}

#[test]
fn test_flatten8() {
    let a = flatten_program!("var x = 1; var y = 2; var z = x + y;").unwrap();
    let b = resolve_program!(
        "var x0 = 1;
         var x1 = 2;
         var x2 = Add[_, _]::add;
         var x3 = x2(x0, x1);"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_flatten9() {
    let a = flatten_program!("var x = 1; var y = { var z = 2; z };").unwrap();
    let b = resolve_program!("var x0 = 1; var x1 = 2;").unwrap();
    check!(a, b);
}

#[test]
fn test_flatten10() {
    let a = flatten_program!("var x = { var y = { var z = 3; z }; y };").unwrap();
    let b = resolve_program!("var x0 = 3;").unwrap();
    check!(a, b);
}
