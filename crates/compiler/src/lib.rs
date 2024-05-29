use std::rc::Rc;

use ast::Expr;
use ast::Pat;
use ast::Program;
use ast::Stmt;
use ast::Type;
use builtins::Value;
use config::Config;
use diag::Report;
use diag::Sources;
use lexer::Lexer;
use parser::Parser;

pub mod annotate;
pub mod apply;
pub mod ast;
pub mod codegen;
pub mod diag;
pub mod display;
// pub mod ffi;
pub mod builtins;
pub mod infer;
pub mod interpret;
pub mod lexer;
#[allow(unused)]
pub mod lift;
pub mod parser;
pub mod print;
pub mod resolve;

pub mod map;
// pub mod monomorphise;
pub mod monomorphise;
#[cfg(feature = "optimiser")]
pub mod opt;
// pub mod ordmap;
pub mod symbol;
// pub mod union_find;
// mod visitor;

#[derive(Debug)]
pub struct Compiler {
    pub sources: Sources,
    pub(crate) declarations: Vec<Stmt>,
    resolve: resolve::Context,
    #[allow(unused)]
    lift: lift::Context,
    infer: infer::Context,
    interpret: interpret::Context,
    monomorphise: monomorphise::Context,
    report: Report,
    pub config: Config,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Drop for Compiler {
    fn drop(&mut self) {
        if !self.report.is_empty() {
            self.report.print(&mut self.sources).unwrap();
        }
    }
}

impl Compiler {
    pub fn new(config: Config) -> Self {
        Compiler {
            declarations: Vec::new(),
            sources: Sources::new(),
            resolve: resolve::Context::new(),
            lift: lift::Context::new(),
            infer: infer::Context::new(),
            monomorphise: monomorphise::Context::new(),
            interpret: interpret::Context::new(),
            report: Report::new(),
            config,
        }
    }

    pub fn init(&mut self) -> &mut Self {
        self.declare();
        let stmts = self.declarations.drain(..).collect();
        let program = Program::new(stmts);
        let program = self.resolve.resolve(&program);
        self.report.merge(&mut self.resolve.report);
        let program = self.lift.lift(&program);
        self.report.merge(&mut self.lift.report);
        let program = self.infer.infer(&program);
        self.report.merge(&mut self.infer.report);
        let _program = self.monomorphise.monomorphise(&program);
        // self.interpret.interpret(&program);
        // self.report.merge(&mut self.interpret.report);
        self
        // let result = self.inferrer.infer(&result);
        // let result = self.inferrer.infer(&result);
        // self.interpreter.interpret(&result);
        // assert!(self.inferrer.report.is_empty());
        // assert!(self.interpreter.report.is_empty());
    }

    pub fn parse<T>(
        &mut self,
        name: impl ToString,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>) -> T,
    ) -> Result<T, Recovered<T>> {
        let input: Rc<str> = unindent::unindent(input).into();
        let id = self.sources.add(name, input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = f(&mut parser);
        self.report.merge(&mut parser.report);
        self.report.merge(&mut lexer.report);
        self.recover(result)
    }

    pub fn resolve(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.parse(name, input, |parser| parser.parse(Parser::program).unwrap())?;
        let result = self.resolve.resolve(&program);
        self.report.merge(&mut self.resolve.report);
        self.recover(result)
    }

    pub fn lift(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.resolve(name, input)?;
        let program = self.lift.lift(&program);
        self.recover(program)
    }

    pub fn infer(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let result = self.resolve(name, input)?;
        let result = self.infer.infer(&result);
        self.report.merge(&mut self.infer.report);
        self.recover(result)
    }

    pub fn monomorphise(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let result = self.infer(name, input)?;
        let result = self.monomorphise.monomorphise(&result);
        self.recover(result)
    }

    pub fn interpret(&mut self, name: &str, input: &str) -> Result<Value, Recovered<Value>> {
        let mut result = self.infer(name, input).unwrap();
        let stmt = result.stmts.pop().unwrap();
        let expr = stmt.as_expr().unwrap();
        self.interpret.interpret(&result);
        let value = self.interpret.expr(expr);
        self.report.merge(&mut self.interpret.report);
        self.recover(value)
    }

    fn recover<T>(&mut self, result: T) -> Result<T, Recovered<T>> {
        if self.report.is_empty() {
            Ok(result)
        } else {
            Err(Recovered::new(result, self.report()))
        }
    }

    pub fn report(&mut self) -> String {
        trim(&self.report.string(&mut self.sources).unwrap())
    }
}

pub fn trim(s: &str) -> String {
    // Trim space right before \n on each line
    s.trim_end()
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

impl Stmt {
    pub fn parse(input: &str) -> Result<Stmt, Recovered<Stmt>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::stmt).unwrap())
    }
}

impl Expr {
    pub fn parse(input: &str) -> Result<Expr, Recovered<Expr>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::expr).unwrap())
    }
}

impl Type {
    pub fn parse(input: &str) -> Result<Type, Recovered<Type>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::ty).unwrap())
    }
}

impl Pat {
    pub fn parse(input: &str) -> Result<Pat, Recovered<Pat>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::pat).unwrap())
    }
}

impl Program {
    pub fn parse(input: &str) -> Result<Program, Recovered<Program>> {
        Compiler::default().parse("test", input, |parser| {
            parser.parse(Parser::program).unwrap()
        })
    }
    pub fn resolve(input: &str) -> Result<Program, Recovered<Program>> {
        Compiler::default().init().resolve("test", input)
    }
    pub fn lift(input: &str) -> Result<Program, Recovered<Program>> {
        Compiler::default().init().lift("test", input)
    }
    pub fn infer(input: &str) -> Result<Program, Recovered<Program>> {
        Compiler::default().init().infer("test", input)
    }
    pub fn monomorphise(input: &str) -> Result<Program, Recovered<Program>> {
        Compiler::default().init().monomorphise("test", input)
    }
    pub fn interpret(input: &str) -> Result<Value, Recovered<Value>> {
        Compiler::default().init().interpret("test", input)
    }
}

pub struct Recovered<T> {
    pub val: T,
    pub msg: String,
}

impl<T> Recovered<T> {
    pub fn new(value: T, report: String) -> Self {
        Self {
            val: value,
            msg: report,
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Debug for Recovered<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.val)?;
        write!(f, "\n{}", &self.msg)
    }
}
