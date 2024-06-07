use ena::unify::NoError;
use ena::unify::UnifyKey;
use ena::unify::UnifyValue;

use crate::ast::Type;
use crate::ast::TypeVar;

impl UnifyKey for TypeVar {
    type Value = TypeVarValue;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(i: u32) -> Self {
        TypeVar(i)
    }

    fn tag() -> &'static str {
        "TypeVar"
    }
}

#[derive(Debug, Clone)]
pub enum TypeVarValue {
    Known(Type),
    Unknown(TypeVarKind),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TypeVarKind {
    General,
    Int,
    Float,
}

impl std::fmt::Display for TypeVarKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TypeVarKind::General => write!(f, "{{general}}"),
            TypeVarKind::Int => write!(f, "{{int}}"),
            TypeVarKind::Float => write!(f, "{{float}}"),
        }
    }
}

impl TypeVarKind {
    pub fn is_mergeable(self, other: TypeVarKind) -> bool {
        match (self, other) {
            (TypeVarKind::General, _) | (_, TypeVarKind::General) => true,
            (TypeVarKind::Int, TypeVarKind::Int) => true,
            (TypeVarKind::Float, TypeVarKind::Float) => true,
            _ => false,
        }
    }

    pub fn merge(self, other: TypeVarKind) -> TypeVarKind {
        match (self, other) {
            (TypeVarKind::General, k) | (k, TypeVarKind::General) => k,
            (TypeVarKind::Int, TypeVarKind::Int) => TypeVarKind::Int,
            (TypeVarKind::Float, TypeVarKind::Float) => TypeVarKind::Float,
            _ => unreachable!(),
        }
    }

    pub fn is_int(self) -> bool {
        match self {
            TypeVarKind::Int => true,
            _ => false,
        }
    }

    pub fn is_float(self) -> bool {
        match self {
            TypeVarKind::Float => true,
            _ => false,
        }
    }

    pub fn is_general(self) -> bool {
        match self {
            TypeVarKind::General => true,
            _ => false,
        }
    }
}

impl TypeVarValue {
    pub fn known(self) -> Option<Type> {
        match self {
            TypeVarValue::Known(t) => Some(t),
            TypeVarValue::Unknown(_) => None,
        }
    }

    pub fn unknown(self) -> Option<TypeVarKind> {
        match self {
            TypeVarValue::Known(_) => None,
            TypeVarValue::Unknown(k) => Some(k),
        }
    }
}

impl std::fmt::Display for TypeVarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TypeVarValue::Known(t) => write!(f, "{}", t),
            TypeVarValue::Unknown(_) => write!(f, "<unknown>"),
        }
    }
}

impl UnifyValue for TypeVarValue {
    type Error = NoError;

    fn unify_values(t0: &TypeVarValue, t1: &TypeVarValue) -> Result<TypeVarValue, NoError> {
        use TypeVarValue::*;
        match (t0, t1) {
            (Unknown(k1), Unknown(k2)) => Ok(Unknown(k1.merge(*k2))),
            (Unknown(_), t) | (t, Unknown(_)) => Ok(t.clone()),
            (Known(_), Known(_)) => unreachable!(),
        }
    }
}
