use crate::ast::Program;
use crate::diag::Report;

pub mod controlflow;
pub mod desugar;
pub mod expand;
pub mod flatten;
pub mod infer;
#[allow(unused)]
pub mod lift;
pub mod loops;
pub mod monomorphise;
pub mod query_desugar;
pub mod resolve;

pub trait Pass: std::fmt::Debug {
    fn run(&mut self, program: &Program) -> Program;
    fn report(&mut self) -> &mut Report;
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
