use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        aqua: "trait Mul[A,B] {
             type Output;
             def mul(a:A, b:B): Mul[A,B]::Output;
         }",
    });
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
