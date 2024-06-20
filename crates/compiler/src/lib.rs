use std::rc::Rc;

use ast::Program;
use ast::Stmt;
use builtins::Value;
use config::Config;
use diag::Report;
use diag::Sources;
use lexer::Lexer;
use parser::Parser;

pub mod apply;
pub mod ast;
pub mod codegen;
pub mod diag;
pub mod display;
pub mod ssa;
// pub mod ffi;
pub mod builtins;
pub mod controlflow;
pub mod desugar;
pub mod infer;
pub mod interpret;
pub mod lexer;
#[allow(unused)]
pub mod lift;
pub mod parser;
pub mod print;
pub mod query;
pub mod resolve;
pub mod traversal {
    pub mod mapper;
    pub mod visitor;
}
pub mod collections {
    pub mod map;
    pub mod ordmap;
}
pub mod monomorphise;
#[cfg(feature = "optimiser")]
pub mod opt;
pub mod symbol;

#[derive(Debug)]
pub struct Compiler {
    pub sources: Sources,
    pub declarations: Vec<Stmt>,
    pub desugar: desugar::Context,
    pub query: query::Context,
    pub resolve: resolve::Context,
    pub flatten: ssa::Context,
    #[allow(unused)]
    pub lift: lift::Context,
    pub infer: infer::Context,
    pub interpret: interpret::Context,
    pub monomorphise: monomorphise::Context,
    pub report: Report,
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
            desugar: desugar::Context::new(),
            query: query::Context::new(),
            resolve: resolve::Context::new(),
            flatten: ssa::Context::new(),
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
        let stmts: Vec<Stmt> = self.declarations.drain(..).collect();
        let s = stmts.first().unwrap().span_of() + stmts.last().unwrap().span_of();
        let program = Program::new(s, stmts);
        let program = self.desugar.desugar(&program);
        let program = self.query.querycomp(&program);
        let program = self.resolve.resolve(&program);
        let program = self.flatten.flatten(&program);
        self.report.merge(&mut self.resolve.report);
        let program = self.lift.lift(&program);
        self.report.merge(&mut self.lift.report);
        let program = self.infer.infer(&program);
        self.report.merge(&mut self.infer.report);
        let _program = self.monomorphise.monomorphise(&program);
        self.interpret.interpret(&program);
        self.report.merge(&mut self.interpret.report);
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
        let input: Rc<str> = Rc::from(input);
        let id = self.sources.add(name, input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = f(&mut parser);
        self.report.merge(&mut parser.report);
        self.report.merge(&mut lexer.report);
        self.recover(result)
    }

    pub fn desugar(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.parse(name, input, |parser| parser.parse(Parser::program).unwrap())?;
        let result = self.desugar.desugar(&program);
        self.recover(result)
    }

    pub fn querycomp(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.desugar(name, input)?;
        let result = self.query.querycomp(&program);
        self.recover(result)
    }

    pub fn resolve(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.querycomp(name, input)?;
        let result = self.resolve.resolve(&program);
        self.report.merge(&mut self.resolve.report);
        self.recover(result)
    }

    pub fn flatten(&mut self, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.resolve(name, input)?;
        let result = self.flatten.flatten(&program);
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
        let mut result = self.monomorphise(name, input).unwrap();
        let stmt = result.stmts.pop().unwrap();
        let expr = stmt.as_expr().unwrap();
        self.interpret.interpret(&result);
        let value = self.interpret.expr(expr);
        self.report.merge(&mut self.interpret.report);
        self.recover(value)
    }

    pub fn add_report(&mut self, report: &mut Report) {
        self.report.merge(report);
    }

    pub fn recover<T>(&mut self, result: T) -> Result<T, Recovered<T>> {
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

impl Program {
    pub fn parse(input: &str) -> Result<Program, Recovered<Program>> {
        let mut compiler = Compiler::default();
        compiler.parse("test", input, |parser| {
            parser.parse(Parser::program).unwrap()
        })
    }
}
