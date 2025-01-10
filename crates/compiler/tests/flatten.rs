use compiler::aqua;

use crate::common::passes::flatten;

#[macro_use]
mod common;

#[test]
fn test_flatten0() {
    let a = flatten(aqua!("1 + 2;")).unwrap();
    let b = flatten(aqua!("Add[_, _]::add(1, 2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_flatten1() {
    let a = flatten(aqua!("&1;")).unwrap();
    let b = flatten(aqua!("val x0 = 1; &x0;")).unwrap();
    check!(a, b);
}

#[test]
fn test_flatten2() {
    let a = flatten(aqua!("&&1;")).unwrap();
    let b = flatten(aqua!("val x0 = 1; val x1 = &x0; &x1;")).unwrap();
    check!(a, b);
}

#[test]
fn test_flatten3() {
    let a = flatten(aqua!("&1 + 1;")).unwrap();
    let b = flatten(aqua!("val x0 = 1; Add[_, _]::add(&x0, 1);")).unwrap();
    check!(a, b);
}

#[test]
fn test_flatten4() {
    let a = flatten(aqua!("1+2;")).unwrap();
    let b = flatten(aqua!("Add[_, _]::add(1,2);")).unwrap();
    check!(a, b);
}

#[test]
fn test_flatten5() {
    let a = flatten(aqua!("{ &1; }")).unwrap();
    let b = flatten(aqua!("{ val x0 = 1; &x0; }")).unwrap();
    check!(a, b);
}

#[test]
fn test_flatten6() {
    let a = flatten(aqua!("{ &&1; }")).unwrap();
    let b = flatten(aqua!("{ val x0 = 1; val x1 = &x0; &x1; }")).unwrap();
    check!(a, b);
}

#[test]
fn test_flatten7() {
    let a = flatten(aqua!("{ &1 + 1; }")).unwrap();
    let b = flatten(aqua!("{ val x0 = 1; Add[_, _]::add(&x0, 1); }")).unwrap();
    check!(a, b);
}
