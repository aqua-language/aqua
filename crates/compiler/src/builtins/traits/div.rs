use crate::builtins::value::Value;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_div(&mut self) {
        self.declare_trait(
            "trait Div[A,B] {
                 type Output;
                 def div(a:A, b:B): Div[A,B]::Output;
             }",
        );
    }
}

impl std::ops::Div for Value {
    type Output = Value;
    fn div(self, other: Value) -> Value {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a / b),
            (Value::F32(a), Value::F32(b)) => Value::F32(a / b),
            _ => unreachable!(),
        }
    }
}
