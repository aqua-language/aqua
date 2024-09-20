use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        aqua: "trait Sub[A,B] {
             type Output;
             def sub(a:A, b:B): Sub[A,B]::Output;
         }",
    });
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
