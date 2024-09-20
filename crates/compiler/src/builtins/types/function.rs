use linkme::distributed_slice;
use runtime::prelude::Send;
use runtime::prelude::Sync;

use crate::ast::ExprBody;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Type;
use crate::builtins::value::Value;

use crate::builtins::Context;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(_ctx: &mut Context) {}

#[derive(Debug, Clone, Eq, PartialEq, Send, Sync)]
pub struct Fun {
    pub params: Map<Name, Type>,
    pub body: ExprBody,
}

impl std::fmt::Display for Fun {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        let mut iter = self.params.iter();
        if let Some((k, v)) = iter.next() {
            write!(f, "{}: {}", k, v)?;
            for (k, v) in iter {
                write!(f, ", {}: {}", k, v)?;
            }
        }
        write!(f, ") => {}", self.body)
    }
}

impl Fun {
    pub fn new(xs: Map<Name, Type>, body: ExprBody) -> Self {
        Self { params: xs, body }
    }

    pub fn call(&self, ctx: &mut crate::interpret::Context, args: &[Value]) -> Value {
        match &self.body {
            ExprBody::UserDefined(e) => ctx.scoped(|ctx| {
                for (x, v) in self.params.keys().zip(args) {
                    ctx.stack.bind(*x, v.clone())
                }
                ctx.eval_expr(&e)
            }),
            ExprBody::Builtin(b) => (b.fun)(ctx, args),
        }
    }
}
