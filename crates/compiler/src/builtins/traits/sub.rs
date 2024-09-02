use crate::builtins::value::Value;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_sub(&mut self) {
        self.declare_trait(
            "trait Sub[A,B] {
                 type Output;
                 def sub(a:A, b:B): Sub[A,B]::Output;
             }",
        );
    }
}

impl std::ops::Sub for Value {
    type Output = Value;
    fn sub(self, other: Value) -> Value {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a - b),
            (Value::F32(a), Value::F32(b)) => Value::F32(a - b),
            _ => unreachable!(),
        }
    }
}
