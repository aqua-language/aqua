use std::rc::Rc;
use std::sync::Arc;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::ast::Codegen;
use crate::ast::Stmt;
use crate::diag::Report;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::source::SourceId;
use crate::span::Span;
use crate::spanned::Spanned;
use crate::token::Token;
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
    pub mod blob;
    pub mod bool;
    pub mod char;
    pub mod dataflow;
    pub mod dict;
    pub mod discretizer;
    pub mod duration;
    pub mod encoding;
    pub mod f32;
    pub mod f64;
    pub mod file;
    pub mod function;
    pub mod i128;
    pub mod i16;
    pub mod i32;
    pub mod i64;
    pub mod i8;
    pub mod image;
    pub mod instance;
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
    pub mod writer;
}
mod functions {
    mod io;
}

#[distributed_slice]
pub static DECLS: [fn(&mut Context)];

pub struct Context {
    pub stmts: Vec<Stmt>,
    pub report: Report,
}

pub enum Decl {
    Def {
        aqua: &'static str,
        fun: fn(&mut crate::interpret::Context, &[Value]) -> Value,
        codegen: Option<Codegen>,
    },
    Type {
        aqua: &'static str,
        codegen: Option<Codegen>,
    },
    Impl {
        aqua: &'static str,
        decls: &'static [ImplDecl],
    },
    Trait {
        aqua: &'static str,
    },
}

pub enum ImplDecl {
    Type {
        aqua: &'static str,
    },
    Def {
        aqua: &'static str,
        fun: fn(&mut crate::interpret::Context, &[Value]) -> Value,
        codegen: Option<Codegen>,
    },
}

impl Context {
    pub fn declare(&mut self, decl: Decl) {
        let stmt = match decl {
            Decl::Def { aqua, fun, codegen } => self
                .try_parse(aqua, |parser, follow| {
                    parser.stmt_def_builtin(follow, BuiltinDef { codegen, fun })
                })
                .map(|s| Stmt::Def(Rc::new(s))),
            Decl::Type { aqua, codegen } => self
                .try_parse(aqua, |parser, follow| {
                    parser.stmt_type_builtin(follow, BuiltinType { codegen })
                })
                .map(|s| Stmt::Type(Rc::new(s))),
            Decl::Impl { aqua, decls } => {
                let mut aqua = aqua.to_string();
                aqua.push_str(" {\n");
                for v in decls.iter().map(|t| match t {
                    ImplDecl::Type { aqua } => aqua,
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
                        ImplDecl::Def { codegen, fun, .. } => Some(BuiltinDef {
                            codegen: codegen.clone(),
                            fun: *fun,
                        }),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                self.try_parse(&aqua, |parser, follow| {
                    parser.stmt_impl_builtin(follow, &defs)
                })
                .map(|s| Stmt::Impl(Rc::new(s)))
            }
            Decl::Trait { aqua } => self
                .try_parse(aqua, |parser, follow| parser.stmt_trait(follow))
                .map(|s| Stmt::Trait(Rc::new(s))),
        };
        if let Some(stmt) = stmt {
            self.stmts.push(stmt)
        }
    }

    fn try_parse<T>(
        &mut self,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>, Token) -> Result<Spanned<T>, Span>,
    ) -> Option<T> {
        let input: Arc<str> = Arc::from(input);
        let id = SourceId::new("builtin", input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = parser.parse(f);
        self.report.merge(&mut parser.report);
        self.report.merge(&mut lexer.report);
        result
    }
}

impl Context {
    fn new() -> Self {
        Self {
            stmts: Vec::new(),
            report: Report::new(),
        }
    }
}

pub fn declare() -> Vec<Stmt> {
    let mut ctx = Context::new();
    DECLS.iter().for_each(|decl| decl(&mut ctx));
    ctx.stmts
}
