use runtime::builtins::file::File;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_file(&mut self) {
        self.declare_type("type File;", BuiltinType { rust: "File" });

        self.declare_def(
            "def open(path:Path): File;",
            BuiltinDef {
                rust: "File::open",
                fun: |_ctx, v| {
                    let v0 = v[0].as_path();
                    File::open(v0).into()
                },
            },
        );

        self.declare_def(
            "def read_to_string(file:File): String;",
            BuiltinDef {
                rust: "File::read_to_string",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_file();
                    // v0.read_to_string().into()
                },
            },
        );

        self.declare_def(
            "def read_to_bytes(file:File): Blob;",
            BuiltinDef {
                rust: "File::read_to_bytes",
                fun: |_ctx, v| {
                    let v0 = v[0].as_file();
                    v0.read_to_bytes().into()
                },
            },
        );

        self.declare_def(
            "def inspect(file:File): ();",
            BuiltinDef {
                rust: "File::inspect",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_file();
                    // v0.inspect().into()
                },
            },
        );
    }
}
