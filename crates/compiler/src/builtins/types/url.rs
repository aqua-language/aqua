use std::rc::Rc;

use linkme::distributed_slice;
use runtime::builtins::url::Url;

use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Url;",
        codegen: None,
    });
    ctx.declare(Decl::Impl {
        aqua: "impl Url",
        decls: &[ImplDecl::Def {
            aqua: "def parse(s: String): Result[Url];",
            codegen: None,
            eval: |_ctx, v| {
                let v0 = v[0].as_string();
                Url::parse(v0).map(|v| Rc::new(Value::from(v))).into()
            },
        }],
    });
}
