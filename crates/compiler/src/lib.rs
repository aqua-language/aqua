use std::rc::Rc;

use ast::parse::lexer::Lexer;
use ast::parse::parser::Parser;
use ast::passes::Pass;
use ast::Ast;
use config::CompilerConfig;
use report::source::Cache;
use report::span::Span;
use report::Report;

pub mod analysis;
pub mod ast;
pub mod backend;
// pub mod ffi;
pub mod builtins;
pub mod collections;
pub mod interpret;
pub mod mir;
pub mod print;
pub mod report;
pub mod transforms;
pub mod traversal;

pub mod hir;
#[cfg(feature = "optimiser")]
pub mod opt;
pub mod parser;

#[macro_export]
macro_rules! aqua {
    ($($code:tt)*) => {
        indoc::indoc!($($code)*)
    };
}

#[derive(Debug)]
pub struct Compiler {
    pub sources: report::source::Cache,
    desugar: ast::passes::desugar::Context,
    query_desugar: ast::passes::query_desugar::Context,
    resolve: ast::passes::resolve::Context, // Resolve names to their definitions
    lift: ast::passes::lift::Context,       // Lift non-expression statements to the top level
    flatten: ast::passes::flatten::Context, // Flatten nested place-expressions
    expand: ast::passes::expand::Context,   // Expand type aliases
    #[allow(unused)]
    capture: ast::passes::capture::Context, // Capture variables in closures
    infer: ast::passes::infer::Context,
    // ast_to_mir: passes::ast_to_mir::Context,
    monomorphise: ast::passes::monomorphise::Context,
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
            sources: report::source::Cache::new(),
            desugar: ast::passes::desugar::Context::new(),
            query_desugar: ast::passes::query_desugar::Context::new(),
            resolve: ast::passes::resolve::Context::new(),
            lift: ast::passes::lift::Context::new(),
            flatten: ast::passes::flatten::Context::new(),
            expand: ast::passes::expand::Context::new(),
            capture: ast::passes::capture::Context::new(),
            infer: ast::passes::infer::Context::new(),
            // ast_to_mir: passes::ast_to_mir::Context::new(),
            monomorphise: ast::passes::monomorphise::Context::new(),
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
        let r = parser.parse(Parser::program).unwrap();
        self.report.append(&mut parser.report);
        self.report.append(&mut lexer.report);
        r.v
    }

    pub fn init(&mut self) -> &mut Self {
        let stmts = crate::builtins::declare(&mut self.sources);
        let program = Ast::new(Span::default(), stmts);
        self.compile(&program);
        self
    }

    fn run_pass(
        program: &Ast,
        pass: &mut impl Pass,
        report: &mut Report,
        cache: &mut Cache,
    ) -> Ast {
        let program = pass.run(program, cache);
        report.append(pass.report());
        program
    }

    pub fn run_desugar(&mut self, program: &Ast) -> Ast {
        Self::run_pass(
            program,
            &mut self.desugar,
            &mut self.report,
            &mut self.sources,
        )
    }

    pub fn run_query_desugar(&mut self, program: &Ast) -> Ast {
        let program = self.run_desugar(program);
        Self::run_pass(
            &program,
            &mut self.query_desugar,
            &mut self.report,
            &mut self.sources,
        )
    }

    pub fn run_resolve(&mut self, program: &Ast) -> Ast {
        let program = self.run_query_desugar(program);
        Self::run_pass(
            &program,
            &mut self.resolve,
            &mut self.report,
            &mut self.sources,
        )
    }

    pub fn run_lift(&mut self, program: &Ast) -> Ast {
        let program = self.run_resolve(program);
        Self::run_pass(
            &program,
            &mut self.lift,
            &mut self.report,
            &mut self.sources,
        )
    }

    pub fn run_flatten(&mut self, program: &Ast) -> Ast {
        let program = self.run_lift(program);
        Self::run_pass(
            &program,
            &mut self.flatten,
            &mut self.report,
            &mut self.sources,
        )
    }

    pub fn run_expand(&mut self, program: &Ast) -> Ast {
        let program = self.run_flatten(program);
        Self::run_pass(
            &program,
            &mut self.expand,
            &mut self.report,
            &mut self.sources,
        )
    }

    pub fn run_capture(&mut self, program: &Ast) -> Ast {
        let program = self.run_expand(program);
        Self::run_pass(
            &program,
            &mut self.capture,
            &mut self.report,
            &mut self.sources,
        )
    }

    pub fn run_infer(&mut self, program: &Ast) -> Ast {
        let program = self.run_expand(program);
        Self::run_pass(
            &program,
            &mut self.infer,
            &mut self.report,
            &mut self.sources,
        )
    }

    pub fn _run_ast_to_mir(&mut self, _program: &Ast) -> Ast {
        // let program = self.run_infer(program);
        todo!()
        // Self::run_pass(&program, &mut self.ast_to_mir, &mut self.report)
    }

    pub fn run_monomorphise(&mut self, program: &Ast) -> Ast {
        let program = self.run_infer(program);
        if self.report.is_empty() {
            Self::run_pass(
                &program,
                &mut self.monomorphise,
                &mut self.report,
                &mut self.sources,
            )
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
