use ena::unify::NoError;
use ena::unify::UnifyKey;
use ena::unify::UnifyValue;

use crate::ast::Impl;
use crate::ast::ImplVar;

impl UnifyKey for ImplVar {
    type Value = ImplVarValue;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(i: u32) -> Self {
        ImplVar(i)
    }

    fn tag() -> &'static str {
        "ImplVar"
    }
}

#[derive(Debug, Clone)]
pub enum ImplVarValue {
    Known(Impl),
    Unknown,
}

impl ImplVarValue {
    pub fn known(self) -> Option<Impl> {
        match self {
            ImplVarValue::Known(t) => Some(t),
            ImplVarValue::Unknown => None,
        }
    }

    pub fn is_unknown(self) -> bool {
        match self {
            ImplVarValue::Known(_) => false,
            ImplVarValue::Unknown => true,
        }
    }
}

impl std::fmt::Display for ImplVarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImplVarValue::Known(t) => write!(f, "{}", t),
            ImplVarValue::Unknown => write!(f, "<unknown>"),
        }
    }
}

impl UnifyValue for ImplVarValue {
    type Error = NoError;

    fn unify_values(t0: &ImplVarValue, t1: &ImplVarValue) -> Result<ImplVarValue, NoError> {
        use ImplVarValue::*;
        match (t0, t1) {
            (Unknown, Unknown) => Ok(Unknown),
            (Unknown, t) | (t, Unknown) => Ok(t.clone()),
            (Known(_), Known(_)) => unreachable!(),
        }
    }
}
