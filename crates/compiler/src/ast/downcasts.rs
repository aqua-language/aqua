use super::BuiltinDef;
use super::BuiltinType;
use super::Expr;
use super::Name;
use super::Pat;
use super::Path;
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
use super::Trait;
use super::Type;

impl Stmt {
    pub fn as_var(&self) -> Option<&StmtVar> {
        if let Stmt::Var(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_def(&self) -> Option<&StmtDef> {
        if let Stmt::Def(d) = self {
            Some(d)
        } else {
            None
        }
    }
    pub fn as_trait(&self) -> Option<&StmtTrait> {
        if let Stmt::Trait(t) = self {
            Some(t)
        } else {
            None
        }
    }
    pub fn as_impl(&self) -> Option<&StmtImpl> {
        if let Stmt::Impl(i) = self {
            Some(i)
        } else {
            None
        }
    }
    pub fn as_struct(&self) -> Option<&StmtStruct> {
        if let Stmt::Struct(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_enum(&self) -> Option<&StmtEnum> {
        if let Stmt::Enum(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_type(&self) -> Option<&StmtType> {
        if let Stmt::Type(t) = self {
            Some(t)
        } else {
            None
        }
    }
    pub fn as_expr(&self) -> Option<&Expr> {
        if let Stmt::Expr(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

impl StmtDefBody {
    pub fn as_expr(&self) -> &Expr {
        let StmtDefBody::UserDefined(e) = self else {
            unreachable!()
        };
        e
    }
    pub fn as_builtin(&self) -> &BuiltinDef {
        let StmtDefBody::Builtin(b) = self else {
            unreachable!()
        };
        b
    }
}

impl Trait {
    pub fn as_type(&self, x: &Name) -> Option<&Type> {
        match self {
            Trait::Path(_, _) => None,
            Trait::Cons(_, _, xts) => xts.get(x),
            Trait::Type(_) => None,
            Trait::Err => None,
            Trait::Var(..) => None,
        }
    }
}


impl Path {
    pub fn as_name(&self) -> Option<&Name> {
        if self.segments.len() == 1 && self.segments[0].ts.is_empty() {
            Some(&self.segments[0].name)
        } else {
            None
        }
    }
}

impl Type {
    pub fn as_name(&self) -> Option<&Name> {
        if let Type::Path(p) = self {
            p.as_name()
        } else {
            None
        }
    }
}

impl Pat {
    pub fn as_name(&self) -> Option<&Name> {
        if let Pat::Path(_, _, p, None) = self {
            p.as_name()
        } else {
            None
        }
    }
}

impl Expr {
    pub fn as_name(&self) -> Option<&Name> {
        if let Expr::Path(_, _, p) = self {
            p.as_name()
        } else {
            None
        }
    }
}

impl StmtTypeBody {
    pub fn as_udt(&self) -> Option<&Type> {
        match self {
            StmtTypeBody::UserDefined(t) => Some(t),
            StmtTypeBody::Builtin(_) => unreachable!(),
        }
    }
    pub fn as_bit(&self) -> Option<&BuiltinType> {
        match self {
            StmtTypeBody::UserDefined(_) => unreachable!(),
            StmtTypeBody::Builtin(b) => Some(b),
        }
    }
}
