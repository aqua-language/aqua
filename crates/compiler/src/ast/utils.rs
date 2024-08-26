use std::rc::Rc;

use super::Expr;
use super::Name;
use super::StmtDef;
use super::StmtImpl;
use super::StmtTrait;
use super::StmtTraitDef;
use super::StmtTraitType;
use super::StmtType;

impl Expr {
    pub fn is_braced(&self) -> bool {
        matches!(
            self,
            Expr::Block(..) | Expr::Match(..) | Expr::While(..) | Expr::For(..) | Expr::IfElse(..)
        )
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Expr::Tuple(_, _, v) if v.is_empty())
    }

    pub fn is_place(&self) -> bool {
        match self {
            Expr::Var(_, _, _) => true,
            Expr::Field(_, _, e, _) => e.is_place(),
            Expr::Index(_, _, e, _) => e.is_place(),
            _ => false,
        }
    }
}

impl StmtImpl {
    pub fn get_def(&self, x: Name) -> Option<&Rc<StmtDef>> {
        self.defs.iter().find(|stmt| stmt.name == x)
    }

    pub fn get_type(&mut self, x: Name) -> Option<&Rc<StmtType>> {
        self.types.iter().find(|stmt| stmt.name == x)
    }
}

impl StmtTrait {
    pub fn get_def(&self, x: Name) -> Option<&Rc<StmtTraitDef>> {
        self.defs.iter().find(|stmt| stmt.name == x)
    }

    pub fn get_type(&self, x: Name) -> Option<&Rc<StmtTraitType>> {
        self.types.iter().find(|stmt| stmt.name == x)
    }
}
