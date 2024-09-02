use crate::builtins::value::Value;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_not(&mut self) {
        self.declare_trait(
            "trait Not[T] {
                 type Output;
                 def not(v:T): Not[T]::Output;
             }",
        );
    }
}

impl std::ops::Not for Value {
    type Output = Value;
    fn not(self) -> Value {
        match self {
            Value::Bool(a) => Value::Bool(!a),
            _ => unreachable!(),
        }
    }
}
