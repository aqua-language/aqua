
use crate::builtins::value::Value;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_add(&mut self) {
        self.declare_trait(
            "trait Add[A,B] {
                 type Output;
                 def add(a:A, b:B): Add[A,B]::Output;
             }",
        );
    }
}

impl std::ops::Add for Value {
    type Output = Value;
    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a + b),
            (Value::F32(a), Value::F32(b)) => Value::F32(a + b),
            (Value::Tuple(a), Value::Tuple(b)) => Value::Tuple(a + b),
            _ => unreachable!(),
        }
    }
}
