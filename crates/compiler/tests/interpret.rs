#[macro_use]
mod common;

use compiler::builtins::Value;

#[ignore]
#[test]
fn test_interpret_arithmetic0() {
    let a = interpret_program!("1 + 2 + 3;").unwrap();
    let b = Value::I32(6);
    assert_eq!(a, b);
}

#[ignore]
#[test]
fn test_interpret_arithmetic1() {
    let a = interpret_program!("1 + 2 * 3;").unwrap();
    let b = Value::I32(7);
    assert_eq!(a, b);
}

#[ignore]
#[test]
fn test_interpret_dataflow2() {
    let _a = interpret_program!(
        r#"def f(t:Time):Time = t;
           source(file_reader("file.csv"), csv(), f)"#
    );
}
