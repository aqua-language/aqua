use std::rc::Rc;

use linkme::distributed_slice;
use runtime::builtins::dict::Dict;

use crate::aqua;
use crate::ast::Codegen;
use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Dict[K,V];",
        codegen: Some(Codegen {
            rust: "Dict",
            java: "Dict",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[K,V] Dict[K,V]",
        decls: &[
            ImplDecl::Def {
                aqua: "def new(): Dict[K,V];",
                codegen: Some(Codegen {
                    rust: "Dict::new",
                    java: "Dict.new",
                    egglog: None,
                }),
                eval: |_ctx, _v| Dict::new().into(),
            },
            ImplDecl::Def {
                aqua: "def get(d: Dict[K,V], k: K): Option[V];",
                codegen: Some(Codegen {
                    rust: "|d, k| d.get(&k).cloned()",
                    java: "(d, k) -> d.get(k)",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let d = v[0].as_dict();
                    let k = v[1].clone();
                    let x: runtime::builtins::option::Option<Rc<Value>> =
                        d.get(&k).map(|v| Rc::new(v.clone())).into();
                    let v: Value = x.into();
                    v
                },
            },
            ImplDecl::Def {
                aqua: "def insert(d: Dict[K,V], k: K, v: V): ();",
                codegen: Some(Codegen {
                    rust: "|d, k, v| d.insert(k, v)",
                    java: "(d, k, v) -> d.put(k, v)",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let d = v[0].as_dict();
                    let k = v[1].clone();
                    let v = v[2].clone();
                    d.insert(k, v);
                    d.into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: aqua! {
            "impl[K,V] Display[Dict[K,V]]
                where Display[K],
                      Display[V]"
        },
        decls: &[ImplDecl::Def {
            aqua: "def toString(d: Dict[K,V]): String;",
            codegen: Some(Codegen {
                rust: "|d| format!(\"{{}}\", d)",
                java: "(d) -> d.toString()",
                egglog: None,
            }),
            eval: |_ctx, v| {
                let a0 = v[0].as_dict();
                runtime::builtins::im_string::String::from(a0.to_string()).into()
            },
        }],
    })
}
