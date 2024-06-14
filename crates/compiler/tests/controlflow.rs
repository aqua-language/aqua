#[macro_use]
mod common;

#[test]
fn test_val0() {
    let a = desugar_program!("val x = 0;").unwrap();
    let b = desugar_program!("let x = 0 in ()").unwrap();
    check!(a, b);
}

#[test]
fn test_val1() {
    let a = desugar_program!(
        "val x = 0;
         val y = 1;"
    )
    .unwrap();
    let b = desugar_program!(
        "let x = 0 in
         let y = 1 in
         ()"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_val2() {
    let a = desugar_program!(
        "val x = 0;
         val y = 1;
         val z = 2;"
    )
    .unwrap();
    let b = desugar_program!(
        "let x = 0 in
         let y = 1 in
         let z = 2 in
         ()"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_val3() {
    let a = desugar_program!(
        "var x = 0;
         x = 2;
         x;"
    )
    .unwrap();
    let b = desugar_program!(
        "let x = 0 in
         let x = 2 in
         x"
    )
    .unwrap();
    check!(a, b);
}

#[test]
fn test_val4() {
    let a = desugar_program!(
        "var x = 0;
         for y in [1,2,3] {
             x = x + y;
         }"
    )
    .unwrap();
    let b = desugar_program!(
        "let x = 0 in
         let x = [1,2,3].fold(0, fun(y) = let x = x + y in x) in
         ()"
    )
    .unwrap();
    check!(a, b);
}
