use linkme::distributed_slice;
use runtime::builtins::file::File;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type File;",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl File",
        decls: &[
            ImplDecl::Def {
                aqua: "def open(path:Path): File;",
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_path();
                    File::open(v0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def read_to_string(file:File): String;",
                codegen: None,
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_file();
                    // v0.read_to_string().into()
                },
            },
            ImplDecl::Def {
                aqua: "def read_to_bytes(file:File): Blob;",
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_file();
                    v0.read_to_bytes().into()
                },
            },
            ImplDecl::Def {
                aqua: "def inspect(file:File): ();",
                codegen: None,
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_file();
                    // v0.inspect().into()
                },
            },
        ],
    });
}
