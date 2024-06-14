#![allow(unused)]

use std::sync::OnceLock;

use super::Symbol;

pub struct Cache(OnceLock<Symbols>);

impl Cache {
    const fn new() -> Cache {
        Cache(OnceLock::new())
    }

    pub fn get(&self) -> &Symbols {
        self.0.get_or_init(Symbols::new)
    }
}

pub struct Symbols {
    pub string: Symbol,
    pub i8: Symbol,
    pub i16: Symbol,
    pub i32: Symbol,
    pub i64: Symbol,
    pub i128: Symbol,
    pub f32: Symbol,
    pub f64: Symbol,
    pub bool: Symbol,
    pub char: Symbol,
    pub vec: Symbol,
    pub hashmap: Symbol,
    pub iter: Symbol,
    pub intoiter: Symbol,
    pub item: Symbol,
}

pub static CACHE: Cache = Cache::new();

impl Symbols {
    pub fn new() -> Symbols {
        Symbols {
            string: "String".into(),
            i8: "i8".into(),
            i16: "i16".into(),
            i32: "i32".into(),
            i64: "i64".into(),
            i128: "i128".into(),
            f32: "f32".into(),
            f64: "f64".into(),
            bool: "bool".into(),
            char: "char".into(),
            vec: "Vec".into(),
            hashmap: "HashMap".into(),
            iter: "Iter".into(),
            intoiter: "IntoIter".into(),
            item: "Item".into(),
        }
    }
}
