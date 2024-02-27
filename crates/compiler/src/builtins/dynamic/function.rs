use crate::ast::Name;
use crate::ast::Type;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Fun {
    pub name: Name,
    pub ts: Vec<Type>,
}

impl Fun {
    pub fn new(name: Name, ts: Vec<Type>) -> Fun {
        Fun { name, ts }
    }
}
