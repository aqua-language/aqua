use crate::ast::Name;
use crate::ast::Type;
use crate::Compiler;

#[derive(Debug, Clone)]
pub struct Fun {
    pub x: Name,
    pub ts: Vec<Type>,
}

impl Fun {
    pub fn new(x: Name, ts: Vec<Type>) -> Fun {
        Fun { x, ts }
    }
}

impl Compiler {
    pub(super) fn declare_function(&mut self) {}
}
