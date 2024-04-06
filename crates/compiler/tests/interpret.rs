use compiler::ast::Program;
use compiler::builtins::Value;

#[test]
fn test_interpret0() {
    let a = Program::interpret("1 + 2 + 3;").unwrap();
    let b = Value::I32(6);
    assert_eq!(a, b);
}
