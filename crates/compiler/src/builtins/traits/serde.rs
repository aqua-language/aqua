use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Type;
use crate::builtins::types::array::Array;
use crate::builtins::types::record::Record;
use crate::builtins::types::tuple::Tuple;
use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use crate::syntax::span::Span;
use crate::syntax::symbol::Symbol;
use linkme::distributed_slice;
use runtime::prelude::Send;
use runtime::prelude::Sync;
use serde::de::DeserializeSeed;
use serde::de::MapAccess;
use serde::de::VariantAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::rc::Rc;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: "trait Serde[T] { }",
    });
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Array(v) => v.serialize(serializer),
            Value::Blob(v) => v.serialize(serializer),
            Value::Bool(v) => v.serialize(serializer),
            Value::Char(v) => v.serialize(serializer),
            Value::Dict(v) => v.serialize(serializer),
            Value::Window(v) => v.serialize(serializer),
            Value::Duration(v) => v.serialize(serializer),
            Value::Format(v) => v.serialize(serializer),
            Value::F32(v) => v.serialize(serializer),
            Value::F64(v) => v.serialize(serializer),
            Value::File(_) => unreachable!(),
            Value::Fun(_) => unreachable!(),
            Value::I128(v) => v.serialize(serializer),
            Value::I16(v) => v.serialize(serializer),
            Value::I32(v) => v.serialize(serializer),
            Value::I64(v) => v.serialize(serializer),
            Value::I8(v) => v.serialize(serializer),
            // Value::Matrix(v) => v.serialize(serializer),
            // Value::Model(v) => v.serialize(serializer),
            Value::Option(v) => v.serialize(serializer),
            Value::Path(v) => v.serialize(serializer),
            Value::Reader(v) => v.serialize(serializer),
            Value::Record(v) => v.serialize(serializer),
            Value::Result(v) => v.serialize(serializer),
            Value::Set(v) => v.serialize(serializer),
            Value::SocketAddr(v) => v.serialize(serializer),
            Value::Stream(_) => unreachable!(),
            Value::KeyedStream(_) => unreachable!(),
            Value::String(v) => v.serialize(serializer),
            Value::Time(v) => v.serialize(serializer),
            Value::Tuple(v) => v.serialize(serializer),
            Value::U128(v) => v.serialize(serializer),
            Value::U16(v) => v.serialize(serializer),
            Value::U32(v) => v.serialize(serializer),
            Value::U64(v) => v.serialize(serializer),
            Value::U8(v) => v.serialize(serializer),
            Value::Usize(v) => v.serialize(serializer),
            Value::Url(v) => v.serialize(serializer),
            Value::Variant(v) => v.serialize(serializer),
            Value::Vec(v) => v.serialize(serializer),
            Value::Writer(v) => v.serialize(serializer),
            Value::Dataflow(_) => unreachable!(),
            Value::Instance(_) => unreachable!(),
            Value::Ordering(_) => unreachable!(),
            Value::Backend(_) => unreachable!(),
            Value::Range(v) => v.serialize(serializer),
            Value::Iterator(_) => unreachable!(),
            Value::Storage(v) => v.serialize(serializer),
            Value::Bag(_) => todo!(),
            Value::Unit(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        unreachable!()
    }
}

#[derive(Send, Sync, Clone)]
pub struct Seed(Type, Rc<crate::analysis::declare::Context>);

impl Seed {
    pub fn new(type_tag: Type, decls: crate::analysis::declare::Context) -> Self {
        Self(type_tag, Rc::new(decls))
    }
}

struct TupleVisitor(Vec<Type>, Rc<crate::analysis::declare::Context>);

impl<'de> Visitor<'de> for TupleVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a tuple of length {}", self.0.len())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut v = Vec::new();
        for t in self.0 {
            v.push(seq.next_element_seed(Seed(t, self.1.clone()))?.unwrap());
        }
        Ok(Value::from(Tuple::new(v)))
    }
}

struct RecordVisitor(Map<Name, Type>, Rc<crate::analysis::declare::Context>);

impl<'de> Visitor<'de> for RecordVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a record with fields {:?}", self.0)
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = Map::new();
        while !self.0.is_empty() {
            let k = map.next_key()?.unwrap();
            if let Some(t) = self.0.remove(&k) {
                let seed = Seed(t, self.1.clone());
                result.insert(k, map.next_value_seed(seed)?);
            } else {
                return Err(serde::de::Error::custom("Found unexpected field"));
            }
        }
        Ok(Value::from(Record::new(result)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut result = Map::new();
        for (k, t) in self.0 {
            if let Some(v) = seq.next_element_seed(Seed(t, self.1.clone()))? {
                result.insert(k, v);
            } else {
                return Err(serde::de::Error::custom("Found unexpected field"));
            }
        }
        Ok(Value::from(Record::new(result)))
    }
}

struct DictVisitor(Type, Type, Rc<crate::analysis::declare::Context>);
impl<'de> Visitor<'de> for DictVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a dict")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        #[allow(clippy::mutable_key_type)]
        let mut result = std::collections::HashMap::default();
        while let Some((k, v)) = map.next_entry_seed(
            Seed(self.0.clone(), self.2.clone()),
            Seed(self.1.clone(), self.2.clone()),
        )? {
            result.insert(k, v);
        }
        Ok(Value::from(runtime::builtins::dict::Dict::from(result)))
    }
}

struct SetVisitor(Type, Rc<crate::analysis::declare::Context>);
impl<'de> Visitor<'de> for SetVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a set")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        #[allow(clippy::mutable_key_type)]
        let mut result = std::collections::HashSet::new();
        while let Some(v) = seq.next_element_seed(Seed(self.0.clone(), self.1.clone()))? {
            result.insert(v);
        }
        Ok(Value::from(runtime::builtins::set::Set::from(result)))
    }
}

struct OptionVisitor(Type, Rc<crate::analysis::declare::Context>);
impl<'de> Visitor<'de> for OptionVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an option")
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Seed(self.0, self.1).deserialize(deserializer)?;
        Ok(Value::from(runtime::builtins::option::Option::some(
            Rc::new(v),
        )))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::from(runtime::builtins::option::Option::none()))
    }
}

struct ResultVisitor(Type, Rc<crate::analysis::declare::Context>);
impl<'de> Visitor<'de> for ResultVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a result")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let (v, variant) = data.variant()?;
        match v {
            "Ok" => {
                let seed = Seed(self.0.clone(), self.1.clone());
                let v = variant.newtype_variant_seed(seed)?;
                Ok(Value::from(runtime::builtins::result::Result::ok(Rc::new(
                    v,
                ))))
            }
            "Err" => {
                let v = variant.newtype_variant()?;
                Ok(Value::from(runtime::builtins::result::Result::error(v)))
            }
            _ => unreachable!(),
        }
    }
}

struct VecVisitor(Type, Rc<crate::analysis::declare::Context>);
impl<'de> Visitor<'de> for VecVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a vec")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut result = Vec::new();
        while let Some(v) = seq.next_element_seed(Seed(self.0.clone(), self.1.clone()))? {
            result.push(v);
        }
        Ok(Value::from(runtime::builtins::vec::Vec::from(result)))
    }
}

impl<'de> DeserializeSeed<'de> for Seed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self.0 {
            Type::Function(_, _) => unreachable!(),
            Type::Tuple(ts) if ts.is_empty() => <() as Deserialize>::deserialize(deserializer)
                .map(|()| Value::from(Tuple::new(vec![]))),
            Type::Tuple(ts) => {
                deserializer.deserialize_tuple(ts.len(), TupleVisitor(ts.clone(), self.1.clone()))
            }
            Type::Record(xts) => {
                deserializer.deserialize_map(RecordVisitor(xts.into_iter().collect(), self.1))
            }
            Type::Struct(_, _) => todo!(),
            Type::Enum(_, _) => todo!(),
            Type::Builtin(x, ts) => match x.data.as_str() {
                "i8" => i8::deserialize(deserializer).map(Value::from),
                "i16" => i16::deserialize(deserializer).map(Value::from),
                "i32" => i32::deserialize(deserializer).map(Value::from),
                "i64" => i64::deserialize(deserializer).map(Value::from),
                "u8" => u8::deserialize(deserializer).map(Value::from),
                "u16" => u16::deserialize(deserializer).map(Value::from),
                "u32" => u32::deserialize(deserializer).map(Value::from),
                "u64" => u64::deserialize(deserializer).map(Value::from),
                "usize" => usize::deserialize(deserializer).map(Value::from),
                "f32" => f32::deserialize(deserializer).map(Value::from),
                "f64" => f64::deserialize(deserializer).map(Value::from),
                "bool" => bool::deserialize(deserializer).map(Value::from),
                "char" => char::deserialize(deserializer).map(Value::from),
                "String" => String::deserialize(deserializer)
                    .map(runtime::builtins::im_string::String::from)
                    .map(Value::from),
                "Dict" => {
                    let k = ts[0].clone();
                    let v = ts[1].clone();
                    deserializer.deserialize_map(DictVisitor(k, v, self.1.clone()))
                }
                "Set" => {
                    let t = ts[0].clone();
                    deserializer.deserialize_seq(SetVisitor(t, self.1))
                }
                "Time" => runtime::builtins::time::Time::deserialize(deserializer).map(Value::from),
                "Duration" => runtime::builtins::duration::Duration::deserialize(deserializer)
                    .map(Value::from),
                // "Url" => runtime::builtins::url::Url::deserialize(deserializer).map(Value::from),
                "Path" => runtime::builtins::path::Path::deserialize(deserializer).map(Value::from),
                "Blob" => runtime::builtins::blob::Blob::deserialize(deserializer).map(Value::from),
                "Option" => {
                    let t = ts[0].clone();
                    deserializer.deserialize_option(OptionVisitor(t, self.1.clone()))
                }
                "Result" => {
                    let t = ts[0].clone();
                    deserializer.deserialize_enum(
                        "Result",
                        &["Ok", "Err"],
                        ResultVisitor(t, self.1.clone()),
                    )
                }
                "Vec" => {
                    let t = ts[0].clone();
                    deserializer.deserialize_seq(VecVisitor(t, self.1.clone()))
                }
                _ => {
                    if let Some(stmt) = self.1.structs.get(&x) {
                        return deserializer
                            .deserialize_map(RecordVisitor(stmt.fields.clone(), self.1.clone()));
                    }
                    if let Some(_stmt) = self.1.enums.get(&x) {
                        todo!();
                    }
                    unreachable!()
                }
            },
            Type::Generic(_) => unreachable!(),
            Type::Array(t, n) => {
                struct ArrayVisitor(Type, Rc<crate::analysis::declare::Context>);
                impl<'de> Visitor<'de> for ArrayVisitor {
                    type Value = Value;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "an array")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let mut result = Vec::new();
                        while let Some(v) =
                            seq.next_element_seed(Seed(self.0.clone(), self.1.clone()))?
                        {
                            result.push(v);
                        }
                        Ok(Value::from(Array(result)))
                    }
                }
                let n = n.unwrap();
                deserializer.deserialize_tuple(n, ArrayVisitor(t.as_ref().clone(), self.1.clone()))
            }
            Type::Never => unreachable!(),
            Type::Var(_) => Err(serde::de::Error::custom(
                "Attempted to deserialize a type variable",
            )),
            Type::Err => unreachable!(),
            Type::Assoc(_, _, _) => unreachable!(),
            Type::Unknown => unreachable!(),
            Type::Path(_) => unreachable!(),
            Type::Paren(_) => unreachable!(),
            Type::Alias(..) => unreachable!(),
            Type::Ref(_, _, _) => todo!(),
            Type::Unit => todo!(),
        }
    }
}

impl serde::Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|data| Self {
            span: Span::default(),
            data: Symbol::from(data),
        })
    }
}
