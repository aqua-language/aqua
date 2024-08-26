#[macro_use]
mod common;

use compiler::ast::Map;
use compiler::builtins::Record;
use compiler::builtins::Tuple;
use compiler::builtins::Value;
use compiler::builtins::Variant;

#[test]
fn test_interpret_arith0() {
    let a = interpret_program!("1 + 2 + 3;").unwrap();
    let b = Value::I32(6);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_arith1() {
    let a = interpret_program!("1 + 2 * 3;").unwrap();
    let b = Value::I32(7);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_var0() {
    let a = interpret_program!("var x = 1; var y = 2; x + y;").unwrap();
    let b = Value::I32(3);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_def0() {
    let a = interpret_program!("def f(x:i32):i32 = x; f(1);").unwrap();
    let b = Value::I32(1);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_struct0() {
    let a = interpret_program!(
        "struct Point(x:i32, y:i32);
         Point(x=1, y=2);"
    )
    .unwrap();
    let b = Record::new(Map::from(vec![
        ("x".into(), Value::I32(1)),
        ("y".into(), Value::I32(2)),
    ]))
    .into();
    assert_eq!(a, b);
}

#[test]
fn test_interpret_tuple0() {
    let a = interpret_program!("(1, 2);").unwrap();
    let b = Tuple::new(vec![Value::I32(1), Value::I32(2)]).into();
    assert_eq!(a, b);
}

#[test]
fn test_interpret_enum0() {
    let a = interpret_program!(
        "enum Maybe[T] { Just(T), Nothing }
         Maybe::Just(1);"
    )
    .unwrap();
    let b = Variant::new("Just".into(), Value::I32(1)).into();
    assert_eq!(a, b);
}

#[test]
fn test_interpret_if0() {
    let a = interpret_program!("if true { 1 } else { 2 };").unwrap();
    let b = Value::I32(1);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_if1() {
    let a = interpret_program!("if false { 1 } else { 2 };").unwrap();
    let b = Value::I32(2);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_assign0() {
    let a = interpret_program!("var x = 1; x = 2; x;").unwrap();
    let b = Value::I32(2);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_while0() {
    let a = interpret_program!("while false { }").unwrap();
    let b = Value::Tuple(Tuple::new(vec![]));
    assert_eq!(a, b);
}

#[test]
fn test_interpret_while1() {
    let a = interpret_program!("var x = 0; while x < 10 { x = x + 1; }; x;").unwrap();
    let b = Value::I32(10);
    assert_eq!(a, b);
}

#[test]
#[ignore]
fn test_interpret_dataflow0() {
    let _a = interpret_program!(
        r#"struct Item(price:i32, ts:Time);
           def extractTimestamp(t:Time):Time = t;
           source(file_reader("file.csv"), csv(), extractTimestamp);"#
    );
    // let b = Value::Dataflow
}
