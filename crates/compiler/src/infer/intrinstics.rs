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
    Type::Cons("bool".into(), vec![])
}

pub fn char() -> Type {
    Type::Cons("char".into(), vec![])
}

pub fn unit() -> Type {
    Type::Tuple(vec![])
}

pub fn string() -> Type {
    Type::Cons("String".into(), vec![])
}

pub fn default_int() -> Type {
    Type::Cons("i32".into(), vec![])
}

pub fn default_float() -> Type {
    Type::Cons("f64".into(), vec![])
}
