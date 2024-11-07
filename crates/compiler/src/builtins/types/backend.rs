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
                aqua: "def native(): Backend;",
                codegen: None,
                eval: |_ctx, _v| Backend::native().into(),
            },
            ImplDecl::Def {
                aqua: "def flink(): Backend;",
                codegen: None,
                eval: |_ctx, _v| Backend::flink().into(),
            },
        ],
    });
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Backend {
    Native,
    Flink,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Backend::Native => write!(f, "Native"),
            Backend::Flink => write!(f, "Flink"),
        }
    }
}

impl Backend {
    pub fn native() -> Self {
        Self::Native
    }
    pub fn flink() -> Self {
        Self::Flink
    }
}
