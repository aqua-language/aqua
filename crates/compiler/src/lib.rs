use std::rc::Rc;

use ast::Ast;
use config::CompilerConfig;
use diag::Report;
use pass::Pass;
use syntax::lexer::Lexer;
use syntax::parser::Parser;
use syntax::span::Span;

pub mod analysis;
pub mod ast;
pub mod backend;
pub mod diag;
pub mod pass;
// pub mod ffi;
pub mod builtins;
pub mod collections;
pub mod interpret;
pub mod print;
pub mod syntax;
pub mod traversal;

#[cfg(feature = "optimiser")]
pub mod opt;
pub mod mir;
pub mod ast_to_mir;
pub mod mir_to_ast;

#[macro_export]
macro_rules! aqua {
    ($($code:tt)*) => {
        indoc::indoc!($($code)*)
    };
}

#[derive(Debug)]
pub struct Compiler {
    pub sources: syntax::source::Cache,
    desugar: pass::desugar::Context,
    query_desugar: pass::query_desugar::Context,
    resolve: pass::resolve::Context,
    expand: pass::expand::Context,
    infer: pass::infer::Context,
    monomorphise: pass::monomorphise::Context,
    pub interpreter: interpret::Context,
    pub report: Report,
    pub config: CompilerConfig,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler {
            sources: syntax::source::Cache::default(),
            desugar: pass::desugar::Context::new(),
            query_desugar: pass::query_desugar::Context::new(),
            resolve: pass::resolve::Context::new(),
            expand: pass::expand::Context::new(),
            infer: pass::infer::Context::new(),
            monomorphise: pass::monomorphise::Context::new(),
            report: Report::new(),
            interpreter: interpret::Context::new(),
            config: CompilerConfig::default(),
        }
    }
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        Compiler {
            sources: syntax::source::Cache::default(),
            desugar: pass::desugar::Context::new(),
            query_desugar: pass::query_desugar::Context::new(),
            resolve: pass::resolve::Context::new(),
            expand: pass::expand::Context::new(),
            infer: pass::infer::Context::new(),
            monomorphise: pass::monomorphise::Context::new(),
            report: Report::new(),
            interpreter: interpret::Context::new(),
            config,
        }
    }

    pub fn parse(&mut self, name: impl ToString, input: impl ToString) -> Ast {
        let input = input.to_string();
        let input: Rc<str> = Rc::from(input);
        let id = self.sources.add(name, input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        parser.parse(Parser::program).unwrap()
    }

    pub fn init(&mut self) -> &mut Self {
        let stmts = crate::builtins::declare(&mut self.sources);
        let program = Ast::new(Span::default(), stmts);
        self.compile(&program);
        self
    }

    fn run_pass(program: &Ast, pass: &mut impl Pass, report: &mut Report) -> Ast {
        let program = pass.run(program);
        report.append(pass.report());
        program
    }

    pub fn desugar(&mut self, program: &Ast) -> Ast {
        Self::run_pass(program, &mut self.desugar, &mut self.report)
    }

    pub fn query_desugar(&mut self, program: &Ast) -> Ast {
        let program = self.desugar(program);
        Self::run_pass(&program, &mut self.query_desugar, &mut self.report)
    }

    pub fn resolve(&mut self, program: &Ast) -> Ast {
        let program = self.query_desugar(program);
        Self::run_pass(&program, &mut self.resolve, &mut self.report)
    }

    pub fn expand(&mut self, program: &Ast) -> Ast {
        let program = self.resolve.run(program);
        Self::run_pass(&program, &mut self.expand, &mut self.report)
    }

    pub fn infer(&mut self, program: &Ast) -> Ast {
        let program = self.resolve(program);
        Self::run_pass(&program, &mut self.infer, &mut self.report)
    }

    pub fn monomorphise(&mut self, program: &Ast) -> Ast {
        let program = self.infer(program);
        if self.report.is_empty() {
            Self::run_pass(&program, &mut self.monomorphise, &mut self.report)
        } else {
            program
        }
    }

    pub fn compile(&mut self, program: &Ast) -> Ast {
        self.monomorphise(program)
    }

    pub fn run(&mut self, name: impl ToString, input: impl ToString) {
        let program = self.parse(name, input);
        let program = self.compile(&program);
        if self.report.is_empty() {
            self.interpreter.interpret(&program);
        }
    }
}

impl Drop for Compiler {
    fn drop(&mut self) {
        if !self.report.is_empty() {
            self.report.print(&mut self.sources).unwrap();
        }
    }
}
