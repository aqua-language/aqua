use std::rc::Rc;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::ast::Stmt;
use crate::lexer::Lexer;
use crate::lexer::Span;
use crate::lexer::Spanned;
use crate::lexer::Token;
use crate::parser::Parser;
use crate::Compiler;

pub mod traits;
pub mod types;

#[rustfmt::skip]
macro_rules! builtin_file_name {
    () => {{
        #[cfg(not(debug_assertions))] { "builtin" }
        #[cfg(debug_assertions)] { format!("{}:", std::panic::Location::caller()) }
    }};
}

impl Compiler {
    pub(crate) fn declare(&mut self) {
        self.declare_types();
        self.declare_traits();
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn declare_def(&mut self, source: &'static str, body: BuiltinDef) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_def_builtin(follow, body)
        }) {
            self.declarations.push(Stmt::Def(Rc::new(stmt)))
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn declare_type(&mut self, source: &'static str, body: BuiltinType) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_type_builtin(follow, body)
        }) {
            self.declarations.push(Stmt::Type(Rc::new(stmt)))
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn declare_trait(&mut self, source: &'static str) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_trait(follow)
        }) {
            self.declarations.push(Stmt::Trait(Rc::new(stmt)))
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn declare_impl<const N: usize>(&mut self, source: &'static str, defs: [BuiltinDef; N]) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_impl_builtin(follow, defs)
        }) {
            self.declarations.push(Stmt::Impl(Rc::new(stmt)))
        }
    }

    pub fn define_def(&mut self, source: &'static str) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_def(follow)
        }) {
            self.declarations.push(Stmt::Def(Rc::new(stmt)))
        }
    }

    fn try_parse<T>(
        &mut self,
        name: impl ToString,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>, Token) -> Result<Spanned<T>, Span>,
    ) -> Option<T> {
        let input: Rc<str> = unindent::unindent(input).into();
        let id = self.sources.add(name, input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = parser.parse(f);
        self.report.merge(&mut parser.report);
        self.report.merge(&mut lexer.report);
        result
    }
}
