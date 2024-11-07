use linkme::distributed_slice;

use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        aqua: indoc::indoc! {
            "trait Add[A,B] {
                 type Output;
                 def add(a:A, b:B): Add[A,B]::Output;
             }"
        },
    });
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
