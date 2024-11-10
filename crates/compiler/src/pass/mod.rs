use crate::ast::Program;

pub trait Pass {
    fn run(&mut self, program: &Program) -> Program;
}
