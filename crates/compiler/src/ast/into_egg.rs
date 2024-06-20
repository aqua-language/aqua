use egglog::EGraph;

use crate::symbol::Symbol;

use super::Program;

type EggExpr = egglog::ast::GenericExpr<Symbol, Symbol, ()>;

struct Context {
    egraph: EGraph,
}

impl Context {
    fn new() -> Self {
        Context {
            egraph: EGraph::default(),
        }
    }

    fn program(&self, _program: &Program) -> EggExpr {
        todo!()
    }
}
