use crate::passes::infer::solver::Constraint;

use super::BuiltinDef;
use super::BuiltinType;
use super::Expr;
use super::ExprBody;
use super::Impl;
use super::Local;
use super::Name;
use super::Pat;
use super::Path;
use super::Place;
use super::Stmt;
use super::StmtDef;
use super::StmtEnum;
use super::StmtImpl;
use super::StmtStruct;
use super::StmtTrait;
use super::StmtType;
use super::StmtLocal;
use super::Trait;
use super::Type;
use super::TypeBody;

impl Stmt {
    pub fn as_var(&self) -> Option<&StmtLocal> {
        if let Stmt::Local(v) = self {
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

impl ExprBody {
    pub fn as_udf(&self) -> Option<&Expr> {
        if let ExprBody::UserDefined(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_bif(&self) -> Option<&BuiltinDef> {
        if let ExprBody::Builtin(b) = self {
            Some(b)
        } else {
            None
        }
    }
}

impl Impl {
    pub fn as_type(&self) -> Option<&Type> {
        match self {
            Impl::Path(_, _) => None,
            Impl::Trait(_) => None,
            Impl::Type(t) => Some(t),
            Impl::Err => None,
            Impl::Var(..) => None,
            Impl::Unknown => None,
        }
    }
    pub fn as_trait(&self) -> Option<&Trait> {
        match self {
            Impl::Path(..) => None,
            Impl::Trait(tr) => Some(tr),
            Impl::Type(_) => None,
            Impl::Err => None,
            Impl::Var(..) => None,
            Impl::Unknown => None,
        }
    }
}

impl Path {
    pub fn as_name(&self) -> Option<&Name> {
        if self.segments.len() == 1 && self.segments[0].ts.is_empty() {
            Some(&self.segments[0].x)
        } else {
            None
        }
    }
}

impl Expr {
    fn as_local(&self) -> Option<Local> {
        match self {
            Expr::Path(_, _, p) => {
                let Some(x) = p.as_name() else { return None };
                Some(Local::new(x.span, *x, Type::Unknown, false))
            }
            Expr::Annotate(_, t, e) => {
                let Expr::Path(_, _, p) = e.as_ref() else {
                    return None;
                };
                let x = *p.as_name()?;
                Some(Local::new(x.span, x, t.clone(), false))
            }
            _ => None,
        }
    }

    pub fn as_locals(&self) -> Option<Vec<Local>> {
        match self {
            Expr::Tuple(_, _, es) => es.iter().map(|e| e.as_local()).collect(),
            Expr::Paren(_, _, e) => {
                let xt = e.as_local()?;
                Some(vec![xt])
            }
            Expr::Unit(_, _) => Some(vec![]),
            _ => {
                let xt = self.as_local()?;
                Some(vec![xt])
            }
        }
    }

    pub fn as_place(&self) -> Option<&Place> {
        match self {
            Expr::Place(_, _, p) => Some(p),
            _ => None,
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

    pub fn as_params(&self) -> Vec<Type> {
        match self {
            Type::Tuple(ts) => ts.clone(),
            Type::Unit => vec![],
            _ => vec![self.clone()],
        }
    }

    pub fn as_path(&self) -> Option<&Path> {
        if let Type::Path(p) = self {
            Some(p)
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

    pub fn as_field(&self) -> Option<(&Name, &Expr)> {
        match self {
            // x = e
            Expr::Assign(_, _, e0, e1) => {
                let x = e0.as_name()?;
                Some((x, e1))
            }
            // e.x
            Expr::Field(_, _, _, x) => Some((x, self)),
            // x
            Expr::Path(_, _, p) => {
                let x = p.as_name()?;
                Some((x, self))
            }
            _ => None,
        }
    }
}

impl TypeBody {
    pub fn as_udt(&self) -> Option<&Type> {
        match self {
            TypeBody::UserDefined(t) => Some(t),
            TypeBody::Builtin(_) => None,
        }
    }
    pub fn as_bit(&self) -> Option<&BuiltinType> {
        match self {
            TypeBody::UserDefined(_) => None,
            TypeBody::Builtin(b) => Some(b),
        }
    }
}

impl Constraint {
    pub fn impl_of(&self) -> Option<&Impl> {
        match self {
            Constraint::WhereClause(_, i) => Some(i),
            Constraint::AssocDef(_, _, i, ..) => Some(i),
            Constraint::AssocType(_, _, i, ..) => Some(i),
            Constraint::PlaceElem(..) => None,
        }
    }
}
