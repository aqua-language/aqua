use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type String;",
        codegen: Some(Codegen {
            rust: "String",
            java: "String",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Serde[String]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl String",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def new(): String;",
                codegen: Some(Codegen {
                    rust: "String::new",
                    java: "String.new",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // String::new().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def with_capacity(cap: usize): String;",
                codegen: Some(Codegen {
                    rust: "String::with_capacity",
                    java: "String.with_capacity",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_usize();
                    // String::with_capacity(a0).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def push_char(s: String, c: char): ();",
                codegen: Some(Codegen {
                    rust: "String::push",
                    java: "String.push",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_char();
                    // a0.push(a1).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def push(s1: String, s2: String): ();",
                codegen: Some(Codegen {
                    rust: "String::push_string",
                    java: "String.push",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_string();
                    // a0.push_string(a1).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def remove(s: String, idx: usize): char;",
                codegen: Some(Codegen {
                    rust: "String::remove",
                    java: "String.remove",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_usize();
                    // let (a, b) = a0.remove(a1);
                    // Tuple(vector![a.into(), b.into()]).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def insert(s: String, idx: usize, c: char): ();",
                codegen: Some(Codegen {
                    rust: "String::insert",
                    java: "String.insert",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_usize();
                    // let a2 = v[2].as_char();
                    // a0.insert(a1, a2).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_empty(s: String): bool;",
                codegen: Some(Codegen {
                    rust: "String::is_empty",
                    java: "String.isEmpty",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // a0.is_empty().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def split_off(s: String, idx: usize): (String, String);",
                codegen: Some(Codegen {
                    rust: "String::split_off",
                    java: "String.splitOff",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_usize();
                    // let (a, b) = a0.split_off(a1);
                    // Tuple(vector![a.into(), b.into()]).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def clear(s: String): ();",
                codegen: Some(Codegen {
                    rust: "String::clear",
                    java: "String.clear",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let a0 = v[0].as_string();
                    a0.clear().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def len(s: String): usize;",
                codegen: Some(Codegen {
                    rust: "String::len",
                    java: "String.len",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let a0 = v[0].as_string();
                    a0.len().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def concat(a: String, b: String): String;",
                codegen: Some(Codegen {
                    rust: "String::concat",
                    java: "String::concat",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let a0 = v[0].as_string();
                    let a1 = v[1].as_string();
                    a0.concat(&a1).into()
                },
            },
            // ImplDecl::Def {
            //     docs: "",
            //     aqua: "def decode[T](s: String, e: Format): Option[T];",
            //     codegen: Some(Codegen {
            //         rust: "String::decode",
            //         java: "String.decode",
            //         egglog: None,
            //     }),
            //     eval: |_ctx, v| {
            //         let v0 = v[0].as_string();
            //         let v1 = v[1].as_encoding();
            //         v0.decode(v1)
            //     },
            // },
            // ImplDecl::Def {
            //     aqua: "def encode[T](s: T, e: Format): Option[String];",
            //     codegen: Some(Codegen {
            //         rust: "String::decode",
            //         java: "String.decode",
            //         egglog: None,
            //     }),
            //     eval: |_ctx, v| {
            //         let v0 = v[0].as_string();
            //         v0.encode(v[1].as_encoding())
            //     },
            // },
            ImplDecl::Def {
                docs: "",
                aqua: "def lines[T](s: String): Vec[T];",
                codegen: Some(Codegen {
                    rust: "String::lines",
                    java: "String.lines",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a0: Vec<_> = a0
                    //     .as_ref()
                    //     .lines()
                    //     .map(|x| String::from(x).into())
                    //     .collect::<std::vec::Vec<Value>>()
                    //     .into();
                    // a0.into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Display[String]",
        decls: &[ImplDecl::Def {
            docs: "",
            aqua: "def toString(s: String): String;",
            codegen: Some(Codegen {
                rust: "String::to_string",
                java: "String.toString",
                egglog: None,
            }),
            eval: |_ctx, v| v[0].as_string().into(),
        }],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl PartialEq[String]",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def eq(a: String, b: String): bool;",
                codegen: Some(Codegen {
                    rust: "|a,b| a==b",
                    java: "String.eq",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let a0 = v[0].as_string();
                    let a1 = v[1].as_string();
                    a0.eq(&a1).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def ne(a: String, b: String): bool;",
                codegen: Some(Codegen {
                    rust: "|a,b| a!=b",
                    java: "String.ne",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let a0 = v[0].as_string();
                    let a1 = v[1].as_string();
                    a0.ne(&a1).into()
                },
            },
        ],
    })
}
