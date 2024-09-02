use crate::builtins::value::Value;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_mul(&mut self) {
        self.declare_trait(
            "trait Mul[A,B] {
                 type Output;
                 def mul(a:A, b:B): Mul[A,B]::Output;
             }",
        );
    }
}

impl std::ops::Mul for Value {
    type Output = Value;
    fn mul(self, other: Value) -> Value {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a * b),
            (Value::F32(a), Value::F32(b)) => Value::F32(a * b),
            _ => unreachable!(),
        }
    }
}
