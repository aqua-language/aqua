use std::rc::Rc;

use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Place;
use crate::ast::PlaceElem;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtImpl;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::StmtType;
use crate::ast::Type;

impl Expr {
    pub fn is_braced(&self) -> bool {
        match self {
            Expr::Block(..)
            | Expr::Match(..)
            | Expr::While(..)
            | Expr::For(..)
            | Expr::IfElse(..) => true,
            _ => false,
        }
    }

    pub fn is_place(&self) -> bool {
        match self {
            Expr::Local(..)
            | Expr::Field(..)
            | Expr::Index(..)
            | Expr::Deref(..)
            | Expr::Place(..) => true,
            _ => false,
        }
    }

    pub fn is_def(&self) -> bool {
        match self {
            Expr::Def(..) | Expr::Assoc(..) => true,
            _ => false,
        }
    }

    pub fn rc(&self) -> Rc<Expr> {
        Rc::new(self.clone())
    }
}

impl Type {
    pub fn rc(&self) -> Rc<Type> {
        Rc::new(self.clone())
    }

    pub fn is_copy(&self) -> bool {
        todo!()
    }

    pub fn downgrade(&self) -> Type {
        if let Type::Ref(loans, t, true) = self {
            Type::Ref(loans.clone(), t.clone(), false)
        } else {
            self.clone()
        }
    }
}

impl Stmt {
    pub fn is_local(&self) -> bool {
        match self {
            Stmt::Local(_) | Stmt::Expr(_) | Stmt::Err(_) => true,
            _ => false,
        }
    }

    pub fn is_global(&self) -> bool {
        match self {
            Stmt::Def(_)
            | Stmt::Trait(_)
            | Stmt::Impl(_)
            | Stmt::Struct(_)
            | Stmt::Enum(_)
            | Stmt::Type(_) => true,
            _ => false,
        }
    }
}

impl StmtImpl {
    pub fn get_def(&self, x: &Name) -> Option<&Rc<StmtDef>> {
        self.defs.iter().find(|stmt| stmt.name == *x)
    }

    pub fn get_type(&mut self, x: &Name) -> Option<&Rc<StmtType>> {
        self.types.iter().find(|stmt| stmt.name == *x)
    }
}

impl StmtTrait {
    pub fn find_def(&self, x: &Name) -> Option<&Rc<StmtTraitDef>> {
        self.defs.iter().find(|stmt| stmt.name == *x)
    }

    pub fn find_type(&self, x: &Name) -> Option<&Rc<StmtTraitType>> {
        self.types.iter().find(|stmt| stmt.name == *x)
    }
}

impl Place {
    pub fn is_prefix_of(&self, other: &Place) -> bool {
        if self.local.name != other.local.name {
            return false;
        }
        let mut iter1 = self.elems.iter();
        let mut iter2 = other.elems.iter();
        loop {
            match (iter1.next(), iter2.next()) {
                (Some(elem1), Some(elem2)) => {
                    if elem1 != elem2 {
                        return false;
                    }
                }
                (None, Some(_)) => return true,
                (Some(_), None) => return false,
                (None, None) => return true,
            }
        }
    }

    pub fn is_mutable(&self) -> bool {
        if self.elems.is_empty() && self.local.mutable {
            return true;
        }
        self.is_mutable_rec()
    }

    fn is_mutable_rec(&self) -> bool {
        let mut t = &self.local.ty;
        for elem in self.elems.iter().rev() {
            t = match elem {
                PlaceElem::Index(_, _, i) => match t {
                    Type::Tuple(ts) => &ts[i.data],
                    _ => return false,
                },
                PlaceElem::Field(_, _, _) => match t {
                    Type::Struct(_, _) => todo!(),
                    _ => return false,
                },
                PlaceElem::Deref(_, _) => match t {
                    Type::Ref(_, _, m) => {
                        if !m {
                            return false;
                        } else {
                            t
                        }
                    }
                    _ => return false,
                },
            };
        }
        true
    }
}
