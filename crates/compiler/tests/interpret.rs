#[macro_use]
mod common;

use common::dsl::expr_body;
use common::dsl::expr_field;
use common::dsl::expr_var;
use common::dsl::params;
use common::dsl::ty;
use common::passes::interpret;
use compiler::aqua;
use compiler::ast::Map;
use compiler::builtins::types::dataflow::Dataflow;
use compiler::builtins::types::function::Function;
use compiler::builtins::types::record::Record;
use compiler::builtins::types::stream::Operator;
use compiler::builtins::types::tuple::Tuple;
use compiler::builtins::types::variant::Variant;
use compiler::builtins::value::Value;
use runtime::builtins::duration::Duration;
use runtime::builtins::path::Path;
use runtime::builtins::writer::Writer;
use runtime::prelude::Format;
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
           Stream[Item]::source(Reader::file(Path::new("file.csv"), false), Format::csv(','), (i:Item, _) => i.ts, 0s, 1s)
               .sink(Writer::file(Path::new("file.csv")), Format::csv(','));"#
    ))
    .unwrap();
    let b = Dataflow::Sink(
        Operator::Source(
            Reader::file(Path::new("file.csv"), false),
            Format::csv(','),
            Function::new(
                params([("i", ty("Item"))]),
                expr_body(
                    expr_field(expr_var("i").with_type(ty("Item")), "ts").with_type(ty("Time")),
                ),
            ),
            Duration::from_seconds(0),
            Duration::from_seconds(1),
        )
        .to_stream(),
        Writer::file(Path::new("file.csv")),
        Format::csv(','),
    )
    .into();
    assert_eq!(a, b);
}

#[test]
fn test_interpret_dataflow1() {
    let a = interpret(aqua!(
        r#"struct Item(price: i32, ts:Time);
           def extract_time(item:Item,_):Time = item.ts;
           from item in source(file_reader(path("file.csv"), false), csv(','), extract_time, 0s, 1s)
           into sink(file_writer(path("file.csv")), csv(','));"#
    ))
    .unwrap();
    let b = interpret(aqua!(
        r#"struct Item(price:i32, ts:Time);
           def extract_time(item:Item,_):Time = item.ts;
           source(file_reader(path("file.csv"), false), csv(','), extract_time, 0s, 1s)
               .map(fun(item:Item) = record(item=item))
               .sink(file_writer(path("file.csv")), csv(','));"#
    ))
    .unwrap();
    check!(@value; a, b);
}

#[test]
fn test_interpret_dataflow2() {
    let _ = interpret(aqua!(
        r#"struct Item(price: i32, ts:Time);
           def extract_time(item:Item,_):Time = item.ts;
           from item in source(file_reader(path("file.csv"), false), csv(','), extract_time, 0s, 1s)
           into sink(file_writer(path("file.csv")), csv(','));"#
    ))
    .unwrap();
}
