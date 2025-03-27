use crate::collections::unionfind::Union;

use crate::ast::Impl;

use crate::ast::ImplVar;

impl Union for ImplVar {
    type Value = ImplVarValue;

    fn into_u32(self) -> u32 {
        self.0 as u32
    }

    fn from_u32(i: u32) -> ImplVar {
        ImplVar(i as u32)
    }

    fn union(a: &Self::Value, b: &Self::Value) -> Self::Value {
        use ImplVarValue::*;
        match (a, b) {
            (Unknown, Unknown) => Unknown,
            (Unknown, t) | (t, Unknown) => t.clone(),
            (Known(_), Known(_)) => unreachable!("Cannot merge two known types"),
        }
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

    pub fn is_unknown(&self) -> bool {
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
