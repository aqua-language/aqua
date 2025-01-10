use std::rc::Rc;

use ast::Ast;
use config::CompilerConfig;
use diag::Report;
use passes::Pass;
use syntax::lexer::Lexer;
use syntax::parser::Parser;
use syntax::span::Span;

pub mod analysis;
pub mod ast;
pub mod backend;
pub mod diag;
pub mod passes;
// pub mod ffi;
pub mod builtins;
pub mod collections;
pub mod interpret;
pub mod print;
pub mod syntax;
pub mod traversal;

pub mod ast_to_mir;
pub mod mir;
pub mod mir_to_ast;
#[cfg(feature = "optimiser")]
pub mod opt;

#[macro_export]
macro_rules! aqua {
    ($($code:tt)*) => {
        indoc::indoc!($($code)*)
    };
}

#[derive(Debug)]
pub struct Compiler {
    pub sources: syntax::source::Cache,
    desugar: passes::desugar::Context,
    query_desugar: passes::query_desugar::Context,
    resolve: passes::resolve::Context,
    lift: passes::lift::Context,
    flatten: passes::flatten::Context,
    expand: passes::expand::Context,
    capture: passes::capture::Context,
    infer: passes::infer::Context,
    // ast_to_mir: passes::ast_to_mir::Context,
    monomorphise: passes::monomorphise::Context,
    pub interpreter: interpret::Context,
    pub report: Report,
    pub config: CompilerConfig,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new(CompilerConfig::default())
    }
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        Compiler {
            sources: syntax::source::Cache::new(),
            desugar: passes::desugar::Context::new(),
            query_desugar: passes::query_desugar::Context::new(),
            resolve: passes::resolve::Context::new(),
            lift: passes::lift::Context::new(),
            flatten: passes::flatten::Context::new(),
            expand: passes::expand::Context::new(),
            capture: passes::capture::Context::new(),
            infer: passes::infer::Context::new(),
            // ast_to_mir: passes::ast_to_mir::Context::new(),
            monomorphise: passes::monomorphise::Context::new(),
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
        parser.parse(Parser::program).unwrap().v
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

    pub fn run_desugar(&mut self, program: &Ast) -> Ast {
        Self::run_pass(program, &mut self.desugar, &mut self.report)
    }

    pub fn run_query_desugar(&mut self, program: &Ast) -> Ast {
        let program = self.run_desugar(program);
        Self::run_pass(&program, &mut self.query_desugar, &mut self.report)
    }

    pub fn run_resolve(&mut self, program: &Ast) -> Ast {
        let program = self.run_query_desugar(program);
        Self::run_pass(&program, &mut self.resolve, &mut self.report)
    }

    pub fn run_lift(&mut self, program: &Ast) -> Ast {
        let program = self.run_resolve(program);
        Self::run_pass(&program, &mut self.lift, &mut self.report)
    }

    pub fn run_flatten(&mut self, program: &Ast) -> Ast {
        let program = self.run_lift(program);
        Self::run_pass(&program, &mut self.flatten, &mut self.report)
    }

    pub fn run_expand(&mut self, program: &Ast) -> Ast {
        let program = self.run_flatten(program);
        Self::run_pass(&program, &mut self.expand, &mut self.report)
    }

    pub fn run_capture(&mut self, program: &Ast) -> Ast {
        let program = self.run_expand(program);
        Self::run_pass(&program, &mut self.capture, &mut self.report)
    }

    pub fn run_infer(&mut self, program: &Ast) -> Ast {
        let program = self.run_capture(program);
        Self::run_pass(&program, &mut self.infer, &mut self.report)
    }

    pub fn _run_ast_to_mir(&mut self, _program: &Ast) -> Ast {
        // let program = self.run_infer(program);
        todo!()
        // Self::run_pass(&program, &mut self.ast_to_mir, &mut self.report)
    }

    pub fn run_monomorphise(&mut self, program: &Ast) -> Ast {
        let program = self.run_infer(program);
        if self.report.is_empty() {
            Self::run_pass(&program, &mut self.monomorphise, &mut self.report)
        } else {
            program
        }
    }

    pub fn compile(&mut self, program: &Ast) -> Ast {
        self.run_monomorphise(program)
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
