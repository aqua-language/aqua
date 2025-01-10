use crate::ast::Name;
use crate::ast::Type;

pub fn ints() -> impl Iterator<Item = Name> {
    [
        "i8".into(),
        "i32".into(),
        "i64".into(),
        "i128".into(),
        "u8".into(),
        "u32".into(),
        "u64".into(),
        "u128".into(),
    ]
    .into_iter()
}

pub fn floats() -> impl Iterator<Item = Name> {
    ["f32".into(), "f64".into()].into_iter()
}

pub fn bool() -> Type {
    Type::Builtin("bool".into(), vec![])
}

pub fn char() -> Type {
    Type::Builtin("char".into(), vec![])
}

pub fn string() -> Type {
    Type::Builtin("String".into(), vec![])
}

pub fn default_int() -> Type {
    Type::Builtin("i32".into(), vec![])
}

pub fn default_float() -> Type {
    Type::Builtin("f64".into(), vec![])
}
