use serde::Serialize;

use crate::ast::BuiltinDef;
use crate::builtins::value::Value;
use crate::Compiler;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Array(pub Vec<Value>);

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        let mut iter = self.0.iter();
        if let Some(v) = iter.next() {
            write!(f, "{}", v)?;
            for v in iter {
                write!(f, ", {}", v)?;
            }
        }
        write!(f, "]")
    }
}

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_array(&mut self) {
        self.declare_def(
            "def array_get[T](array:[T;1], idx:usize): T;",
            BuiltinDef {
                rust: "Array::get",
                fun: |_ctx, v| {
                    let v0 = v[0].as_array();
                    let v1 = v[1].as_usize();
                    v0.0[v1].clone()
                },
            },
        );

        // self.decl_def(
        //     "def array_set(array:[T;1], idx:usize, value:T): ();",
        //     BuiltinDef {
        //         rust: "Array::set",
        //         fun: |_ctx, _t, v| {
        //             let mut v0 = v[0].as_array();
        //             let v1 = v[1].as_usize();
        //             let v2 = v[2].clone();
        //             v0.0[v1] = v2;
        //             v0.into()
        //         },
        //     },
        // );

        self.declare_def(
            "def array_into_vec[T](array:[T;1]): Vec[T];",
            BuiltinDef {
                rust: "Array::into_vec",
                fun: |_ctx, v| {
                    let v0 = v[0].as_array();
                    runtime::builtins::vec::Vec::from(
                        v0.0.into_iter().collect::<std::vec::Vec<_>>(),
                    )
                    .into()
                },
            },
        );

        self.declare_def(
            "def array_into_set[T](array:[T;1]): Set[T];",
            BuiltinDef {
                rust: "Array::into_set",
                fun: |_ctx, v| {
                    let v0 = v[0].as_array();
                    runtime::builtins::set::Set::from(
                        v0.0.into_iter().collect::<std::collections::HashSet<_>>(),
                    )
                    .into()
                },
            },
        );

        self.declare_def(
            "def array_into_dict[K,V](array:[(K,V);1]): Dict[K,V];",
            BuiltinDef {
                rust: "Array::into_dict",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_array();
                    // runtime::builtins::dict::Dict::from(
                    //     v0.0.into_iter()
                    //         .map(|v| {
                    //             let v = v.as_tuple();
                    //             (v.0[0].clone(), v.0[1].clone())
                    //         })
                    //         .collect::<std::collections::HashMap<_, _>>(),
                    // )
                    // .into()
                },
            },
        );

        // self.decl_def(
        //     "def array_iter(array:[T;1]): Vec[T];",
        //     BuiltinDef {
        //         rust: "Array::iter",
        //         fun: |_ctx, _t, _v| {
        //             todo!()
        //             // let v0 = v[0].as_array();
        //             // let v0 = v0.0.clone();
        //             // let v0 = v0.into_iter().collect::<Vec<_>>();
        //             // v0.into()
        //         },
        //     },
        // );
    }
}
