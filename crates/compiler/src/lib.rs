use std::rc::Rc;

use ast::Expr;
use ast::Pat;
use ast::Program;
use ast::Stmt;
use ast::Type;
use config::Config;
use diag::Report;
use diag::Sources;
use dsl::trim;
use lexer::Lexer;
use parser::Parser;

pub mod annotate;
pub mod apply;
pub mod ast;
pub mod codegen;
pub mod diag;
pub mod display;
// pub mod ffi;
pub mod infer;
pub mod lexer;
// pub mod lift;
pub mod dsl;
pub mod parser;
pub mod print;
pub mod resolve;

#[cfg(feature = "optimiser")]
pub mod opt;

#[cfg(feature = "interpret")]
pub mod builtins;

#[cfg(feature = "interpret")]
pub mod interpret;

#[derive(Default, Debug)]
pub struct Compiler {
    ctx0: resolve::Context,
    ctx1: infer::Context,
    // ctx2: interpret::Context,
    pub sources: Sources,
    report: Report,
    pub config: Config,
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
        Self {
            ctx0: resolve::Context::new(),
            ctx1: infer::Context::new(),
            // ctx2: interpret::Context::new(),
            sources: Sources::new(),
            report: Report::new(),
            config,
        }
    }

    pub fn parse<T>(
        &mut self,
        name: impl ToString,
        input: impl Into<Rc<str>>,
        f: impl for<'a> Fn(&mut Parser<'a, &mut Lexer<'a>>) -> T,
    ) -> Result<T, Recovered<T>> {
        let input = input.into();
        let id = self.sources.add(name, input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = f(&mut parser);
        self.report.merge(&mut parser.report);
        self.report.merge(&mut lexer.report);
        self.recover(result)
    }

    pub fn resolve(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.parse(name, input, |parser| parser.parse(Parser::program))?;
        let result = self.ctx0.resolve(&program);
        self.report.merge(&mut self.ctx0.report);
        self.recover(result)
    }

    pub fn infer(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let result = self.resolve(name, input)?;
        let result = self.ctx1.infer(&result);
        self.report.merge(&mut self.ctx1.report);
        self.recover(result)
    }

    // pub fn interpret(&mut self, name: &str, input: &str) -> Result<Program, (Program, String)> {
    //     let program = match self.infer(name, input) {
    //         Ok(program) => program,
    //         Err((program, e)) => return Err((program, e)),
    //     };
    //     self.ctx2.interpret(&program);
    //     self.report.merge(&mut self.ctx2.report);
    //     self.ok_or_err(program)
    // }

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

impl Stmt {
    pub fn parse(input: &str) -> Result<Stmt, Recovered<Stmt>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::stmt))
    }
}

impl Expr {
    pub fn parse(input: &str) -> Result<Expr, Recovered<Expr>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::expr))
    }
}

impl Type {
    pub fn parse(input: &str) -> Result<Type, Recovered<Type>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::ty))
    }
}

impl Pat {
    pub fn parse(input: &str) -> Result<Pat, Recovered<Pat>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::pat))
    }
}

impl Program {
    pub fn parse(input: &str) -> Result<Program, Recovered<Program>> {
        Compiler::default().parse("test", input, |parser| parser.parse(Parser::program))
    }
    pub fn resolve(input: &str) -> Result<Program, Recovered<Program>> {
        Compiler::default().resolve("test", input)
    }
    pub fn try_infer(input: &str) -> Result<Program, Recovered<Program>> {
        Compiler::default().infer("test", input)
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
