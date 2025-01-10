use ena::unify::NoError;
use ena::unify::UnifyKey;
use ena::unify::UnifyValue;

use crate::ast::Type;
use crate::ast::TypeVar;

use super::primitives::floats;
use super::primitives::ints;

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
    pub fn is_unifiable_with(self, t: &Type) -> bool {
        self.is_general()
            || matches!(
                t,
                Type::Builtin(x, _)
                if self.is_int_var() && ints().any(|y| *x == y)
                || self.is_float_var() && floats().any(|y| *x == y)
            )
    }
}

impl TypeVarKind {
    pub fn merge(self, other: TypeVarKind) -> Option<TypeVarKind> {
        match (self, other) {
            (TypeVarKind::General, k) | (k, TypeVarKind::General) => Some(k),
            (TypeVarKind::Int, TypeVarKind::Int) => Some(TypeVarKind::Int),
            (TypeVarKind::Float, TypeVarKind::Float) => Some(TypeVarKind::Float),
            _ => None,
        }
    }

    pub fn is_int_var(self) -> bool {
        match self {
            TypeVarKind::Int => true,
            _ => false,
        }
    }

    pub fn is_float_var(self) -> bool {
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
            (Unknown(k1), Unknown(k2)) => {
                let k = k1.merge(*k2).expect("Type variables should be unifiable");
                Ok(Unknown(k))
            }
            (Unknown(_), t) | (t, Unknown(_)) => Ok(t.clone()),
            (Known(_), Known(_)) => unreachable!(),
        }
    }
}
