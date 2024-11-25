use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: "trait Div[A,B] {
             type Output;
             def div(a:A, b:B): Div[A,B]::Output;
         }",
    });
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
