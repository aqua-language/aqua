#![allow(unused)]

use std::rc::Rc;

use compiler::ast::Ast;
use compiler::ast::Expr;
use compiler::ast::Pat;
use compiler::ast::Stmt;
use compiler::ast::Type;
use compiler::builtins::value::Value;
use compiler::syntax::lexer::Lexer;
use compiler::syntax::parser::Parser;
use compiler::syntax::span::Span;
use compiler::syntax::spanned::Spanned;
use compiler::syntax::token::Token;
use compiler::Compiler;

pub struct Recovered<T> {
    pub val: T,
    pub msg: String,
}

impl<T> Recovered<T> {
    pub fn new(val: T, msg: String) -> Self {
        Self { val, msg }
    }
}

impl<T: std::fmt::Display> std::fmt::Debug for Recovered<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.val)?;
        write!(f, "\n{}", &self.msg)
    }
}

struct Tester(Compiler);

impl Tester {
    pub fn new() -> Self {
        Self(Compiler::default())
    }

    pub fn recover<T>(&mut self, result: T) -> Result<T, Recovered<T>> {
        if self.0.report.is_empty() {
            Ok(result)
        } else {
            let s = self
                .0
                .report
                .to_string(&mut self.0.sources)
                .unwrap()
                .trim()
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n");
            Err(Recovered::new(result, s))
        }
    }

    pub fn init(&mut self) -> &mut Self {
        self.0.init();
        self
    }

    pub fn run<T>(
        &mut self,
        s: &str,
        f: impl FnOnce(&mut Compiler, &Ast) -> T,
    ) -> Result<T, Recovered<T>> {
        let program = self.0.parse("test", s);
        let result = f(&mut self.0, &program);
        self.recover(result)
    }

    pub fn parse<T>(
        &mut self,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>, Token) -> Result<Spanned<T>, Span>,
    ) -> Result<T, Recovered<T>> {
        let input: Rc<str> = Rc::from(input);
        let id = self.0.sources.add("test", input.clone());
        let mut lexer = Lexer::new(id, &input);
        let mut parser = Parser::new(&input, &mut lexer);
        let result = parser.parse(f);
        self.0.report.append(&mut parser.report);
        self.0.report.append(&mut lexer.report);
        self.recover(result.expect("Should not fail").v)
    }
}

pub fn parse(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().parse(s, |p, f| p.program(f))
}

pub fn parse_expr(s: &str) -> Result<Expr, Recovered<Expr>> {
    Tester::new().parse(s, |p, f| p.expr(f))
}

pub fn parse_stmt(s: &str) -> Result<Stmt, Recovered<Stmt>> {
    Tester::new().parse(s, |p, f| p.stmt(f))
}

pub fn parse_type(s: &str) -> Result<Type, Recovered<Type>> {
    Tester::new().parse(s, |p, f| p.ty(f))
}

pub fn parse_pat(s: &str) -> Result<Pat, Recovered<Pat>> {
    Tester::new().parse(s, |p, f| p.pat(f))
}

pub fn desugar(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_desugar)
}

pub fn querycomp(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_query_desugar)
}

pub fn resolve(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_resolve)
}

pub fn lift(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_lift)
}

pub fn flatten(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_flatten)
}

pub fn expand(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_expand)
}

pub fn capture(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_capture)
}

pub fn infer(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_infer)
}

pub fn ast_to_mir(s: &str) -> Result<Ast, Recovered<Ast>> {
    todo!()
}

pub fn monomorphise(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, Compiler::run_monomorphise)
}

pub fn interpret(s: &str) -> Result<Value, Recovered<Value>> {
    Tester::new().init().run(s, |c, p| {
        let mut p = c.run_monomorphise(&p);
        let stmt = p.stmts.pop().unwrap();
        let expr = stmt.as_expr().unwrap();
        c.interpreter.interpret(&p);
        c.interpreter.eval_expr(expr)
    })
}
