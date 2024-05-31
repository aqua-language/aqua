use ena::snapshot_vec::UndoLog;
use ena::undo_log::VecLog;
use ena::unify::Delegate;
use ena::unify::InPlace;
use ena::unify::InPlaceUnificationTable;
use ena::unify::NoError;
use ena::unify::UnificationStore;
use ena::unify::UnificationTable;
use ena::unify::UnifyKey;
use ena::unify::UnifyValue;
use ena::unify::VarValue;

use crate::ast::Type;
use crate::symbol::Symbol;

pub struct UnionFind(InPlaceUnificationTable<Key>);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(u32);

impl UnifyKey for Key {
    type Value = Type;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Key(u)
    }

    fn tag() -> &'static str {
        "UnionFind"
    }
}

impl UnifyValue for Type {
    type Error = NoError;

    fn unify_values(a: &Type, b: &Type) -> Result<Type, NoError> {
        match (a, b) {
            ()
        }
    }
}

impl UnionFind {
    pub fn new() -> Self {
        UnionFind(UnificationStore::new())
    }

    pub fn union(&mut self, a: Symbol, b: Symbol) {
        self.0.union(a, b);
    }

    pub fn find(&mut self, a: Symbol) -> Symbol {
        self.0.find(a)
    }
}
