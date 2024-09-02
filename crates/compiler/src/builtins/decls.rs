use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::ast::Stmt;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::span::Span;
use crate::spanned::Spanned;
use crate::token::Token;
use crate::Compiler;
use std::rc::Rc;

impl Compiler {
    pub fn declare_def(&mut self, source: &'static str, body: BuiltinDef) {
        let stmt = self.try_parse(source, |parser, follow| {
            parser.stmt_def_builtin(follow, body)
        });
        if let Some(stmt) = stmt {
            self.declarations.push(Stmt::Def(Rc::new(stmt)))
        }
    }

    pub fn declare_type(&mut self, source: &'static str, body: BuiltinType) {
        let stmt = self.try_parse(source, |parser, follow| {
            parser.stmt_type_builtin(follow, body)
        });
        if let Some(stmt) = stmt {
            self.declarations.push(Stmt::Type(Rc::new(stmt)))
        }
    }

    pub fn declare_trait(&mut self, source: &'static str) {
        let stmt = self.try_parse(source, |parser, follow| parser.stmt_trait(follow));
        if let Some(stmt) = stmt {
            self.declarations.push(Stmt::Trait(Rc::new(stmt)))
        }
    }

    pub fn declare_impl<const N: usize>(&mut self, source: &'static str, defs: [BuiltinDef; N]) {
        let stmt = self.try_parse(source, |parser, follow| {
            parser.stmt_impl_builtin(follow, defs)
        });
        if let Some(stmt) = stmt {
            self.declarations.push(Stmt::Impl(Rc::new(stmt)))
        }
    }

    fn try_parse<T>(
        &mut self,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>, Token) -> Result<Spanned<T>, Span>,
    ) -> Option<T> {
        let input: Rc<str> = Rc::from(input);
        let id = self.sources.add("builtin", input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = parser.parse(f);
        self.report.merge(&mut parser.report);
        self.report.merge(&mut lexer.report);
        result
    }
}
