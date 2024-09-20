use linkme::distributed_slice;
use serde::Deserialize;
use serde::Serialize;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Backend;",
        codegen: None,
    });
    ctx.declare(Decl::Impl {
        aqua: "impl Backend",
        decls: &[
            ImplDecl::Def {
                aqua: "def rust(): Backend;",
                codegen: None,
                fun: |_ctx, _v| Backend::rust().into(),
            },
            ImplDecl::Def {
                aqua: "def java(): Backend;",
                codegen: None,
                fun: |_ctx, _v| Backend::java().into(),
            },
        ],
    });
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Backend {
    Rust,
    Java,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Backend::Rust => write!(f, "Rust"),
            Backend::Java => write!(f, "Java"),
        }
    }
}

impl Backend {
    pub fn rust() -> Self {
        Self::Rust
    }
    pub fn java() -> Self {
        Self::Java
    }
}
