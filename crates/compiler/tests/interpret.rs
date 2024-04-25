use compiler::ast::Program;
use compiler::builtins::Value;

#[test]
fn test_interpret_arithmetic0() {
    let a = Program::interpret("1 + 2 + 3;").unwrap();
    let b = Value::I32(6);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_arithmetic1() {
    let a = Program::interpret("1 + 2 * 3;").unwrap();
    let b = Value::I32(7);
    assert_eq!(a, b);
}

#[test]
fn test_interpret_dataflow2() {
    let a = Program::interpret(r#"
        def f(t:Time):Time = t;
        source(file_reader("file.csv"), csv(), f)
    "#);
}
