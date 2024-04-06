use crate::ast::Name;
use crate::ast::TraitBound;
use crate::ast::Type;
use crate::Compiler;

#[derive(Debug, Clone)]
pub enum Fun {
    Def(Name, Vec<Type>),
    Assoc(TraitBound, Name, Vec<Type>),
}

impl Fun {
    pub fn new_def(name: Name, ts: Vec<Type>) -> Fun {
        Fun::Def(name, ts)
    }
    pub fn new_assoc(trait_bound: TraitBound, name: Name, ts: Vec<Type>) -> Fun {
        Fun::Assoc(trait_bound, name, ts)
    }
}

impl Compiler {
    pub(super) fn declare_function(&mut self) {}
}
