use crate::ast::Codegen;
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
        docs: "A dynamic array.",
        aqua: "type Vec[T];",
        codegen: Some(Codegen {
            rust: "Vec",
            java: "Vector",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Serde[Vec[T]] where Serde[T]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Display[Vec[T]] where Display[T]",
        decls: &[ImplDecl::Def {
            docs: "",
            aqua: "def toString(v:Vec[T]): String;",
            codegen: Some(Codegen {
                rust: "Vec::to_string",
                java: "Vector::toString",
                egglog: None,
            }),
            eval: |_ctx, v| {
                let a0 = v[0].as_vec();
                runtime::builtins::im_string::String::from(a0.to_string()).into()
            },
        }],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Vec[T]",
        decls: &[
            ImplDecl::Def {
                docs: "Create a new vector.",
                aqua: "def new(): Vec[T];",
                codegen: Some(Codegen {
                    rust: "Vec::new",
                    java: "Vector.new",
                    egglog: None,
                }),
                eval: |_, _v| Vec::new().into(),
            },
            ImplDecl::Def {
                docs: "Push an element onto the end of the vector.",
                aqua: "def push(v:Vec[T], x:T): ();",
                codegen: Some(Codegen {
                    rust: "Vec::push",
                    java: "Vector.push",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].clone();
                    a0.push(a1);
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                docs: "Pop an element from the end of the vector.",
                aqua: "def pop(v:Vec[T]): Option[T];",
                codegen: Some(Codegen {
                    rust: "Vec::pop",
                    java: "Vector.pop",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.pop().map(Into::into).into()
                },
            },
            ImplDecl::Def {
                docs: "Get the length of the vector.",
                aqua: "def len(v:Vec[T]): usize;",
                codegen: Some(Codegen {
                    rust: "Vec::len",
                    java: "Vector.len",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.len().into()
                },
            },
            ImplDecl::Def {
                docs: "Get an element from the vector by its index.",
                aqua: "def get(v:Vec[T], i:usize): Option[T];",
                codegen: Some(Codegen {
                    rust: "Vec::get",
                    java: "Vector.get",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_usize();
                    a0.get(a1).map(Into::into).into()
                },
            },
            ImplDecl::Def {
                docs: "Inserts a value into the vector at a given index.",
                aqua: "def insert(v:Vec[T], i:usize, x:T): ();",
                codegen: Some(Codegen {
                    rust: "Vec::insert",
                    java: "Vector.insert",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_usize();
                    let a2 = v[2].clone();
                    a0.insert(a1, a2);
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                docs: "Returns true if the vector contains no elements.",
                aqua: "def isEmpty(v:Vec[T]): bool;",
                codegen: Some(Codegen {
                    rust: "Vec::is_empty",
                    java: "Vector.isEmpty",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.is_empty().into()
                },
            },
            ImplDecl::Def {
                docs: "Sorts the vector in place.",
                aqua: "def sort(v:Vec[T]): ();",
                codegen: Some(Codegen {
                    rust: "Vec::sort",
                    java: "Vector.sort",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.sort();
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                docs: "Removes the element at the specified index.",
                aqua: "def remove(v:Vec[T], i:usize): Option[T];",
                codegen: Some(Codegen {
                    rust: "Vec::remove",
                    java: "Vector.remove",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_usize();
                    a0.remove(a1).into()
                },
            },
            ImplDecl::Def {
                docs: "Clears the vector, removing all elements.",
                aqua: "def clear(v:Vec[T]): ();",
                codegen: Some(Codegen {
                    rust: "Vec::clear",
                    java: "Vector.clear",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    a0.clear();
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                docs: "Appends a vector with another",
                aqua: "def append(v:Vec[T], x:Vec[T]): ();",
                codegen: Some(Codegen {
                    rust: "Vec::extend",
                    java: "Vector.concat",
                    egglog: None,
                }),
                eval: |_, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_vec();
                    a0.extend(a1);
                    Tuple(vec![]).into()
                },
            },
        ],
    });
}
