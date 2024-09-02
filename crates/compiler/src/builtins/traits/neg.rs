use crate::builtins::value::Value;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_neg(&mut self) {
        self.declare_trait(
            "trait Neg[T] {
                 type Output;
                 def neg(v:T): Neg[T]::Output;
             }",
        );
    }
}

impl std::ops::Neg for Value {
    type Output = Value;
    fn neg(self) -> Value {
        match self {
            Value::I32(a) => Value::I32(-a),
            Value::F32(a) => Value::F32(-a),
            _ => unreachable!(),
        }
    }
}
