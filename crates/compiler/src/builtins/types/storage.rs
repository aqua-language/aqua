use runtime::prelude::DeepClone;
use serde::Serialize;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, DeepClone)]
pub enum Storage {
    // Ephemeral
    Memory,
    // Local Persistent
    Sled,
    // Remote Persistent
    S3,
    AzBlob,
    Gcs,
}

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "Storage backend.",
        aqua: "type Storage;",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Storage",
        decls: &[
            ImplDecl::Def {
                docs: "Memory storage backend.",
                aqua: "def memory(): Storage;",
                eval: |_, _| Storage::Memory.into(),
                codegen: None,
            },
            ImplDecl::Def {
                docs: "Sled storage backend.",
                aqua: "def sled(): Storage;",
                eval: |_, _| Storage::Sled.into(),
                codegen: None,
            },
            ImplDecl::Def {
                docs: "S3 storage backend.",
                aqua: "def s3(): Storage;",
                eval: |_, _| Storage::S3.into(),
                codegen: None,
            },
            ImplDecl::Def {
                docs: "Azure Blob storage backend.",
                aqua: "def az_blob(): Storage;",
                eval: |_, _| Storage::AzBlob.into(),
                codegen: None,
            },
            ImplDecl::Def {
                docs: "Google Cloud Storage backend.",
                aqua: "def gcs(): Storage;",
                eval: |_, _| Storage::Gcs.into(),
                codegen: None,
            },
        ],
    });
}
