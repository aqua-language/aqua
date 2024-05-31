use std::rc::Rc;

use crate::lexer::Span;
use crate::symbol::Symbol;

use super::Expr;
use super::Name;
use super::Stmt;
use super::StmtDef;
use super::StmtDefBody;
use super::StmtEnum;
use super::StmtImpl;
use super::StmtStruct;
use super::StmtTrait;
use super::StmtType;
use super::StmtTypeBody;
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

impl From<Expr> for StmtDefBody {
    fn from(e: Expr) -> StmtDefBody {
        StmtDefBody::UserDefined(e)
    }
}

impl From<Type> for StmtTypeBody {
    fn from(t: Type) -> StmtTypeBody {
        StmtTypeBody::UserDefined(t)
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
