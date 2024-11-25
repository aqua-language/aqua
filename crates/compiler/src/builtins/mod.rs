use std::rc::Rc;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::ast::Codegen;
use crate::ast::Stmt;
use crate::diag::Report;
use crate::syntax::lexer::Lexer;
use crate::syntax::parser::Parser;
use crate::syntax::source::Cache;
use crate::syntax::span::Span;
use crate::syntax::spanned::Spanned;
use crate::syntax::token::Token;
use linkme::distributed_slice;
use value::Value;

mod conv;
pub mod value;
pub mod traits {
    mod add;
    mod clone;
    mod copy;
    mod data;
    mod debug;
    mod deep_clone;
    mod default;
    mod display;
    mod div;
    mod eq;
    mod fun;
    mod hash;
    mod into_iterator;
    mod iterator;
    mod mul;
    mod neg;
    mod not;
    mod ord;
    mod partial_eq;
    mod partial_ord;
    pub mod serde;
    mod sub;
}
pub mod types {
    pub mod array;
    pub mod backend;
    pub mod bag;
    pub mod blob;
    pub mod bool;
    pub mod char;
    pub mod dataflow;
    pub mod dict;
    pub mod duration;
    pub mod f32;
    pub mod f64;
    pub mod file;
    pub mod format;
    pub mod function;
    pub mod i128;
    pub mod i16;
    pub mod i32;
    pub mod i64;
    pub mod i8;
    pub mod image;
    pub mod instance;
    pub mod iterator;
    pub mod keyed_stream;
    pub mod matrix;
    pub mod model;
    pub mod never;
    pub mod option;
    pub mod ordering;
    pub mod path;
    pub mod range;
    pub mod reader;
    pub mod record;
    pub mod result;
    pub mod set;
    pub mod socket;
    pub mod storage;
    pub mod stream;
    pub mod string;
    pub mod time;
    pub mod tuple;
    pub mod u128;
    pub mod u16;
    pub mod u32;
    pub mod u64;
    pub mod u8;
    pub mod url;
    pub mod usize;
    pub mod variant;
    pub mod vec;
    pub mod window;
    pub mod writer;
}
mod functions {
    mod io;
}

#[distributed_slice]
pub static DECLS: [fn(&mut Context)];

pub struct Context<'a> {
    pub stmts: Vec<Stmt>,
    pub report: Report,
    pub sources: &'a mut Cache,
}

#[derive(Debug, Clone, Copy)]
pub enum Decl {
    Def {
        docs: &'static str,
        aqua: &'static str,
        fun: fn(&mut crate::interpret::Context, &[Value]) -> Value,
        codegen: Option<Codegen>,
    },
    Type {
        docs: &'static str,
        aqua: &'static str,
        codegen: Option<Codegen>,
    },
    Impl {
        aqua: &'static str,
        decls: &'static [ImplDecl],
    },
    Trait {
        docs: &'static str,
        aqua: &'static str,
    },
}

#[derive(Debug, Clone)]
pub enum ImplDecl {
    Type {
        docs: &'static str,
        aqua: &'static str,
    },
    Def {
        docs: &'static str,
        aqua: &'static str,
        eval: fn(&mut crate::interpret::Context, &[Value]) -> Value,
        codegen: Option<Codegen>,
    },
}

impl<'s> Context<'s> {
    pub fn declare(&mut self, decl: Decl) {
        match decl {
            Decl::Def {
                docs: _,
                aqua,
                fun,
                codegen,
            } => {
                let s = self.parse(aqua, |parser, follow| {
                    parser.stmt_def_builtin(follow, BuiltinDef { codegen, fun })
                });
                self.stmts.push(Stmt::Def(Rc::new(s)))
            }
            Decl::Type {
                docs: _,
                aqua,
                codegen,
            } => {
                let s = self.parse(aqua, |parser, follow| {
                    parser.stmt_type_builtin(follow, BuiltinType { codegen })
                });
                self.stmts.push(Stmt::Type(Rc::new(s)))
            }
            Decl::Impl { aqua, decls } => {
                let mut aqua = aqua.to_string();
                aqua.push_str(" {\n");
                for v in decls.iter().map(|t| match t {
                    ImplDecl::Type { docs: _, aqua } => aqua,
                    ImplDecl::Def { aqua, .. } => aqua,
                }) {
                    for line in v.lines() {
                        aqua.push_str("    ");
                        aqua.push_str(line);
                        aqua.push_str("\n");
                    }
                }
                aqua.push_str("}");
                let defs = decls
                    .iter()
                    .filter_map(|d| match d {
                        ImplDecl::Def {
                            codegen, eval: fun, ..
                        } => Some(BuiltinDef {
                            codegen: codegen.clone(),
                            fun: *fun,
                        }),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let s = self.parse(&aqua, |parser, follow| {
                    parser.stmt_impl_builtin(follow, &defs)
                });

                self.stmts.push(Stmt::Impl(Rc::new(s)))
            }
            Decl::Trait { docs: _, aqua } => {
                let s = self.parse(aqua, |parser, follow| parser.stmt_trait(follow));
                self.stmts.push(Stmt::Trait(Rc::new(s)))
            }
        };
    }

    fn parse<T>(
        &mut self,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>, Token) -> Result<Spanned<T>, Span>,
    ) -> T {
        let input: Rc<str> = Rc::from(input);
        let id = self.sources.add("builtin", input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = parser.parse(f);
        self.report.append(&mut parser.report);
        self.report.append(&mut lexer.report);
        if self.report.is_empty() {
            result.unwrap()
        } else {
            panic!(
                "Internal Compiler Error: {}",
                self.report.string(&mut self.sources).unwrap()
            );
        }
    }
}

impl<'s> Context<'s> {
    fn new(sources: &'s mut Cache) -> Self {
        Self {
            stmts: Vec::new(),
            report: Report::new(),
            sources,
        }
    }
}

pub fn declare(sources: &mut Cache) -> Vec<Stmt> {
    let mut ctx = Context::new(sources);
    DECLS.iter().for_each(|decl| decl(&mut ctx));
    ctx.stmts
}
