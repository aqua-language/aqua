#[macro_use]
mod common;

use common::dsl::expr_body;
use common::dsl::expr_field;
use common::dsl::expr_var;
use common::dsl::params;
use common::dsl::ty;
use common::passes::interpret;
use compiler::ast::Map;
use compiler::builtins::types::dataflow::Dataflow;
use compiler::builtins::types::function::Fun;
use compiler::builtins::types::record::Record;
use compiler::builtins::types::stream::Stream;
use compiler::builtins::types::tuple::Tuple;
use compiler::builtins::types::variant::Variant;
use compiler::builtins::value::Value;
use runtime::builtins::path::Path;
use runtime::builtins::writer::Writer;
use runtime::prelude::Encoding;
use runtime::prelude::Reader;

#[test]
fn test_interpret_arith0() {
    let a = interpret(aqua!("1 + 2 + 3;")).unwrap();
    let b = Value::I32(6);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_arith1() {
    let a = interpret(aqua!("1 + 2 * 3;")).unwrap();
    let b = Value::I32(7);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_var0() {
    let a = interpret(aqua!("var x = 1; var y = 2; x + y;")).unwrap();
    let b = Value::I32(3);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_def0() {
    let a = interpret(aqua!("def f(x:i32):i32 = x; f(1);")).unwrap();
    let b = Value::I32(1);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_struct0() {
    let a = interpret(aqua!("struct Point(x:i32, y:i32); Point(x=1, y=2);")).unwrap();
    let b = Record::new(Map::from(vec![
        ("x".into(), Value::I32(1)),
        ("y".into(), Value::I32(2)),
    ]))
    .into();
    assert_eq!(a, b);
}

#[test]
fn test_interpret_tuple0() {
    let a = interpret(aqua!("(1, 2);")).unwrap();
    let b = Tuple::new(vec![Value::I32(1), Value::I32(2)]).into();
    assert_eq!(a, b);
}

#[test]
fn test_interpret_enum0() {
    let a = interpret(aqua!("enum Maybe[T] { Just(T), Nothing } Maybe::Just(1);")).unwrap();
    let b = Variant::new("Just".into(), Value::I32(1)).into();
    assert_eq!(a, b);
}

#[test]
fn test_interpret_if0() {
    let a = interpret(aqua!("if true { 1 } else { 2 };")).unwrap();
    let b = Value::I32(1);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_if1() {
    let a = interpret(aqua!("if false { 1 } else { 2 };")).unwrap();
    let b = Value::I32(2);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_assign0() {
    let a = interpret(aqua!("var x = 1; x = 2; x;")).unwrap();
    let b = Value::I32(2);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_while0() {
    let a = interpret(aqua!("while false {}")).unwrap();
    let b = Value::Tuple(Tuple::new(vec![]));
    assert_eq!(a, b);
}

#[test]
fn test_interpret_while1() {
    let a = interpret(aqua!(
        "var x = 0;
         while x < 10 { x = x + 1; };
         x;"
    ))
    .unwrap();
    let b = Value::I32(10);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_dataflow0() {
    let a = interpret(aqua!(
        r#"struct Item(price:i32, ts:Time);
           source(file_reader(path("file.csv"), false), csv(','), fun(i:Item):Time = i.ts)
               .sink(file_writer(path("file.csv")), csv(','));"#
    ))
    .unwrap();
    let b = Dataflow::Sink(
        Stream::Source(
            Reader::file(Path::new("file.csv"), false),
            Encoding::csv(','),
            Fun::new(
                params([("i", ty("Item"))]),
                expr_body(
                    expr_field(expr_var("i").with_type(ty("Item")), "ts").with_type(ty("Time")),
                ),
            ),
        ),
        Writer::file(Path::new("file.csv")),
        Encoding::csv(','),
    )
    .into();
    assert_eq!(a, b);
}

#[test]
fn test_interpret_dataflow1() {
    let a = interpret(aqua!(
        r#"struct Item(price: i32, ts:Time);
           def extract_time(item:Item):Time = item.ts;
           from item in source(file_reader(path("file.csv"), false), csv(','), extract_time)
           into sink(file_writer(path("file.csv")), csv(','));"#
    ))
    .unwrap();
    let b = interpret(aqua!(
        r#"struct Item(price:i32, ts:Time);
           def extract_time(item:Item):Time = item.ts;
           source(file_reader(path("file.csv"), false), csv(','), extract_time)
               .map(fun(item:Item) = record(item=item))
               .sink(file_writer(path("file.csv")), csv(','));"#
    ))
    .unwrap();
    check!(@value, a, b);
}
