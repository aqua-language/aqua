use serde::Deserialize;
use serde::Serialize;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

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

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_backend(&mut self) {
        self.declare_type("type Backend;", BuiltinType { rust: "Backend" });
        self.declare_def(
            "def rust(path: Path, watch: bool): Reader;",
            BuiltinDef {
                rust: "unreachable!()",
                fun: |_ctx, _v| {
                    Backend::rust().into()
                },
            },
        );
    }
}
