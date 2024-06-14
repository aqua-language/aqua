mod standard;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

use append_only_vec::AppendOnlyVec;
use serde::Deserialize;
use serde::Serialize;

static SYMBOLS_READONLY: AppendOnlyVec<SmolStr> = AppendOnlyVec::<SmolStr>::new();
static SYMBOLS: SymbolMap = SymbolMap::new();

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(usize);

impl Symbol {
    pub fn as_str(self) -> &'static str {
        &SYMBOLS_READONLY[self.0]
    }

    pub fn suffix(self, suffix: impl std::fmt::Display) -> Symbol {
        SymbolMap::intern(smol_str::format_smolstr!("{}_{}", self.as_str(), suffix))
    }
}

impl<'a> From<&'a str> for Symbol {
    fn from(name: &'a str) -> Symbol {
        SymbolMap::intern(SmolStr::from(name))
    }
}

impl From<SmolStr> for Symbol {
    fn from(name: SmolStr) -> Symbol {
        SymbolMap::intern(name)
    }
}

impl From<String> for Symbol {
    fn from(name: String) -> Symbol {
        SymbolMap::intern(SmolStr::from(name))
    }
}

pub struct SymbolMap(OnceLock<Mutex<HashMap<SmolStr, Symbol>>>);

impl SymbolMap {
    const fn new() -> SymbolMap {
        SymbolMap(OnceLock::new())
    }

    fn intern(name: SmolStr) -> Symbol {
        let mut state = SYMBOLS
            .0
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();
        state.get(&name).copied().unwrap_or_else(|| {
            let sym = Symbol(SYMBOLS_READONLY.push(name.clone()));
            let prev = state.insert(name, sym);
            debug_assert!(prev.is_none());
            sym
        })
    }

    fn resolve(name: Symbol) -> &'static str {
        &SYMBOLS_READONLY[name.0]
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", SymbolMap::resolve(*self))
    }
}

impl Serialize for Symbol {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(SymbolMap::resolve(*self))
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(SymbolMap::intern(SmolStr::from(String::deserialize(
            deserializer,
        )?)))
    }
}
