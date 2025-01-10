use std::rc::Rc;

use runtime::prelude::Bag;
use runtime::prelude::Duration;
use runtime::prelude::Format;
use runtime::prelude::Reader;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

use super::function::Function;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LazyBag(pub Rc<Operator>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operator {
    Source(Reader, Format, Function, Duration, Duration),
    Take(LazyBag, i32),
    Map(LazyBag, Function),
    Filter(LazyBag, Function),
    Flatten(LazyBag),
    FlatMap(LazyBag, Function),
    Keyby(LazyBag, Function),
    Window(LazyBag, LazyBag, Function),
    IncrWindow(LazyBag, LazyBag, Function, Function, Function),
    Merge(LazyBag, LazyBag),
}

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Bag[T];",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Serde[Bag[T]] where Serde[T]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Bag[T]",
        decls: &[ImplDecl::Def {
            docs: "",
            aqua: "def new(): Bag[T];",
            codegen: Some(Codegen {
                rust: "Bag::new",
                java: "Bag.new",
                egglog: None,
            }),
            eval: |_ctx, _v| Bag::new().into(),
        }],
    });
}
