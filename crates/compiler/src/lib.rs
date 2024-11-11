use anyhow::Result;
use pass::Pass as _;
use std::rc::Rc;

use ast::Program;
use config::CompilerConfig;
use diag::Report;
use lexer::Lexer;
use parser::Parser;
use span::Span;

pub mod ast;
pub mod backend;
pub mod diag;
pub mod display;
pub mod pass;
pub mod analysis {
    pub mod check;
    pub mod declare;
    pub mod lookup;
    pub mod reachable;
}
// pub mod ffi;
pub mod builtins;
pub mod interpret;
pub mod lexer;
pub mod parser;
pub mod print;
pub mod source;
pub mod span;
pub mod spanned;
pub mod splice;
pub mod token;
pub mod traversal {
    pub mod mapper;
    pub mod visitor;
}
pub mod collections {
    pub mod concurrent;
    pub mod map;
    pub mod ordmap;
    pub mod set;
}
#[cfg(feature = "optimiser")]
pub mod opt;
pub mod symbol;

#[macro_export]
macro_rules! aqua {
    ($($code:tt)*) => {
        indoc::indoc!($($code)*)
    };
}

#[derive(Debug)]
pub struct Compiler {
    pub sources: source::Cache,
    // pub passes: Vec<Box<dyn pass::Pass>>,
    pub desugar: pass::desugar::Context,
    pub query: pass::query_desugar::Context,
    pub resolve: pass::resolve::Context,
    pub flatten: pass::flatten::Context,
    pub lift: pass::lift::Context,
    pub infer: pass::infer::Context,
    pub monomorphise: pass::monomorphise::Context,
    pub interpret: interpret::Context,
    pub report: Report,
    pub config: CompilerConfig,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new(CompilerConfig::default())
    }
}

impl Drop for Compiler {
    fn drop(&mut self) {
        if !self.report.is_empty() {
            self.print_report();
        }
    }
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        Compiler {
            sources: source::Cache::default(),
            desugar: pass::desugar::Context::new(),
            query: pass::query_desugar::Context::new(),
            resolve: pass::resolve::Context::new(),
            infer: pass::infer::Context::new(),
            flatten: pass::flatten::Context::new(),
            lift: pass::lift::Context::new(),
            monomorphise: pass::monomorphise::Context::new(),
            interpret: interpret::Context::new(),
            report: Report::new(),
            config,
        }
    }

    pub fn init(&mut self) -> &mut Self {
        let stmts = crate::builtins::declare(&mut self.sources);
        let program = Program::new(Span::default(), stmts);
        let program = self.desugar.run(&program);
        let program = self.query.run(&program);
        let program = self.resolve.run(&program);
        let program = self.infer.run(&program);
        let _program = self.monomorphise.monomorphise(&program);
        self.report.append(&mut self.resolve.report);
        self.report.append(&mut self.infer.report);
        self.interpret.interpret(&program);
        if !self.report.is_empty() {
            self.print_report();
        }
        self
    }

    pub fn check(&mut self, name: impl ToString, input: &str) -> Result<()> {
        let input: Rc<str> = Rc::from(input);
        let id = self.sources.add(name, input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let program = parser.parse(Parser::program).unwrap();
        let program = self.desugar.run(&program);
        let program = self.query.run(&program);
        let program = self.resolve.run(&program);
        let program = self.infer.run(&program);
        let mut report = analysis::check::check(&program);
        self.report.append(&mut parser.report);
        self.report.append(&mut lexer.report);
        self.report.append(&mut self.resolve.report);
        self.report.append(&mut self.infer.report);
        self.report.append(&mut report);
        if self.report.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Compilation failed"))
        }
    }

    pub fn run(&mut self, name: impl ToString, input: &str) -> Result<()> {
        let input: Rc<str> = Rc::from(input);
        let id = self.sources.add(name, input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let program = parser.parse(Parser::program).unwrap();
        let program = self.desugar.run(&program);
        let program = self.query.run(&program);
        let program = self.resolve.run(&program);
        let program = self.infer.run(&program);
        let mut report = analysis::check::check(&program);
        self.report.append(&mut parser.report);
        self.report.append(&mut lexer.report);
        self.report.append(&mut self.resolve.report);
        self.report.append(&mut self.infer.report);
        self.report.append(&mut report);
        if self.report.is_empty() {
            let program = self.monomorphise.monomorphise(&program);
            self.interpret.interpret(&program);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Compilation failed"))
        }
    }

    pub fn add_report(&mut self, report: &mut Report) {
        self.report.append(report);
    }

    pub fn report_to_string(&mut self) -> String {
        self.report.string(&mut self.sources).unwrap()
    }

    pub fn print_report(&mut self) {
        self.report.print(&mut self.sources).unwrap();
    }
}
