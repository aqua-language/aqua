use std::rc::Rc;

use crate::span::Span;
use crate::symbol::Symbol;

use super::Expr;
use super::Name;
use super::Stmt;
use super::StmtDef;
use super::ExprBody;
use super::StmtEnum;
use super::StmtImpl;
use super::StmtStruct;
use super::StmtTrait;
use super::StmtType;
use super::TypeBody;
use super::StmtVar;
use super::Type;

impl From<StmtVar> for Stmt {
    fn from(v: StmtVar) -> Stmt {
        Stmt::Var(Rc::new(v))
    }
}

impl From<StmtDef> for Stmt {
    fn from(d: StmtDef) -> Stmt {
        Stmt::Def(Rc::new(d))
    }
}
impl From<StmtImpl> for Stmt {
    fn from(i: StmtImpl) -> Stmt {
        Stmt::Impl(Rc::new(i))
    }
}

impl From<Expr> for Stmt {
    fn from(e: Expr) -> Stmt {
        Stmt::Expr(Rc::new(e))
    }
}

impl From<StmtStruct> for Stmt {
    fn from(s: StmtStruct) -> Stmt {
        Stmt::Struct(Rc::new(s))
    }
}

impl From<StmtEnum> for Stmt {
    fn from(e: StmtEnum) -> Stmt {
        Stmt::Enum(Rc::new(e))
    }
}

impl From<StmtType> for Stmt {
    fn from(t: StmtType) -> Stmt {
        Stmt::Type(Rc::new(t))
    }
}

impl From<StmtTrait> for Stmt {
    fn from(t: StmtTrait) -> Stmt {
        Stmt::Trait(Rc::new(t))
    }
}

impl From<Expr> for ExprBody {
    fn from(e: Expr) -> ExprBody {
        ExprBody::UserDefined(Rc::new(e))
    }
}

impl From<Type> for TypeBody {
    fn from(t: Type) -> TypeBody {
        TypeBody::UserDefined(t)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Name {
        Name {
            span: Span::default(),
            data: Symbol::from(s),
        }
    }
}

impl From<String> for Name {
    fn from(s: String) -> Name {
        Name {
            span: Span::default(),
            data: Symbol::from(s),
        }
    }
}
