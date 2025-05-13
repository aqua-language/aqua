use crate::ast::Ast;
use crate::report::source::Cache;
use crate::report::Report;

pub mod capture;
pub mod desugar;
pub mod expand;
pub mod flatten;
pub mod infer;
pub mod lift;
pub mod monomorphise;
pub mod query_desugar;
pub mod resolve;

pub trait Pass: std::fmt::Debug {
    fn run(&mut self, program: &Ast, sources: &mut Cache) -> Ast;
    fn report(&mut self) -> &mut Report;
}
