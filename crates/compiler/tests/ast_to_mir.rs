#[macro_use]
mod common;

use common::passes::ast_to_mir;
use compiler::aqua;

#[test]
fn test_lift0() {
    let _a = ast_to_mir(aqua!("def g(): i32 = 1;")).unwrap();
    // function g() -> i32 {
    //     val _0: i32;
    //
    //     'bb0: {
    //         _0 = 1;
    //     }
    // }
}

#[test]
fn test_lift1() {
    let _a = ast_to_mir(aqua! {
        "val x: i32 = 1;"
    })
    .unwrap();
    // function g() -> i32 {
    //     val _0: i32;
    //
    //     'bb0: {
    //         _0 = 1;
    //     }
    // }
}
