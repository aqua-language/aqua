#[macro_use]
mod common;

use crate::common::passes::expand;
use common::dsl::program;
use compiler::aqua;

#[test]
fn test_expand0() {
    let a = expand(aqua!("type T = i32; var x: T = 1;")).unwrap();
    let b = expand(aqua!("var x: i32 = 1;")).unwrap();
    check!(a, b);
}

#[test]
fn test_expand1() {
    let mut s = String::new();
    for i in 1..49 {
        s.push_str(&format!("type T{} = T{};", i, i + 1));
    }
    s.push_str("type T49 = i32;");
    s.push_str("var x: T1 = 1;");

    let a = expand(&s).unwrap();
    let b = expand(aqua!("var x: i32 = 1;")).unwrap();

    check!(a, b);
}

#[test]
fn test_expand2() {
    let a = expand(aqua!("type T = T;")).unwrap_err();
    let b = program([]);
    check!(
        a,
        b,
        "Error: Potential cycle detected when trying to expand type alias.
            ╭─[test:1:10]
            │
          1 │ type T = T;
            │ ─────┬───┬
            │      ╰────── Tried to expand this type alias.
            │          │
            │          ╰── Type aliases can only be nested up to 50 times.
         ───╯"
    );
}
