use crate::ast::Codegen;
use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;

use linkme::distributed_slice;
use runtime::builtins::vec::Vec;

use super::tuple::Tuple;

use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Vec[T];",
        codegen: Some(Codegen {
            rust: "Vec",
            java: "Vector",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Display[Vec[T]] where Display[T]",
        decls: &[ImplDecl::Def {
            aqua: "def toString(v:Vec[T]): String;",
            codegen: Some(Codegen {
                rust: "Vec::to_string",
                java: "Vector.toString",
                egglog: None,
            }),
            fun: |_ctx, v| {
                let a0 = v[0].as_vec();
                runtime::builtins::im_string::String::from(a0.to_string()).into()
            },
        }],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Vec[T]",
        decls: &[
            ImplDecl::Def {
                aqua: "def new(): Vec[T];",
                codegen: Some(Codegen {
                    rust: "Vec::new",
                    java: "Vector.new",
                    egglog: None,
                }),
                fun: |_, _v| Vec::new().into(),
            },
            ImplDecl::Def {
                aqua: "def push(v:Vec[T], x:T): ();",
                codegen: Some(Codegen {
                    rust: "Vec::push",
                    java: "Vector.push",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].clone();
                    a0.push(a1);
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                aqua: "def pop(v:Vec[T]): Option[T];",
                codegen: Some(Codegen {
                    rust: "Vec::pop",
                    java: "Vector.pop",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.pop().map(Into::into).into()
                },
            },
            ImplDecl::Def {
                aqua: "def len(v:Vec[T]): usize;",
                codegen: Some(Codegen {
                    rust: "Vec::len",
                    java: "Vector.len",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.len().into()
                },
            },
            ImplDecl::Def {
                aqua: "def get(v:Vec[T], i:usize): Option[T];",
                codegen: Some(Codegen {
                    rust: "Vec::get",
                    java: "Vector.get",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_usize();
                    a0.get(a1).map(Into::into).into()
                },
            },
            ImplDecl::Def {
                aqua: "def insert(v:Vec[T], i:usize, x:T): ();",
                codegen: Some(Codegen {
                    rust: "Vec::insert",
                    java: "Vector.insert",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_usize();
                    let a2 = v[2].clone();
                    a0.insert(a1, a2);
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                aqua: "def isEmpty(v:Vec[T]): bool;",
                codegen: Some(Codegen {
                    rust: "Vec::is_empty",
                    java: "Vector.isEmpty",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.is_empty().into()
                },
            },
            ImplDecl::Def {
                aqua: "def sort(v:Vec[T]): ();",
                codegen: Some(Codegen {
                    rust: "Vec::sort",
                    java: "Vector.sort",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.sort();
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                aqua: "def remove(v:Vec[T], i:usize): Option[T];",
                codegen: Some(Codegen {
                    rust: "Vec::remove",
                    java: "Vector.remove",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_usize();
                    a0.remove(a1).into()
                },
            },
            ImplDecl::Def {
                aqua: "def clear(v:Vec[T]): ();",
                codegen: Some(Codegen {
                    rust: "Vec::clear",
                    java: "Vector.clear",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.clear();
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                aqua: "def concat(v:Vec[T], x:Vec[T]): ();",
                codegen: Some(Codegen {
                    rust: "Vec::extend",
                    java: "Vector.concat",
                    egglog: None,
                }),
                fun: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_vec();
                    a0.extend(a1);
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                aqua: "def map[T, U](v:Vec[T], f:T => U): Vec[U];",
                codegen: Some(Codegen {
                    rust: "Vec::map",
                    java: "Vector.map",
                    egglog: None,
                }),
                fun: |ctx, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_function();
                    Value::Vec(
                        a0.iter()
                            .map(|a2| a1.call(ctx, &[a2.clone()]))
                            .collect::<Vec<Value>>(),
                    )
                },
            },
        ],
    });
}
