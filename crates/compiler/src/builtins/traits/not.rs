use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: "trait Not[T] {
             type Output;
             def not(v:T): Not[T]::Output;
         }",
    });
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
