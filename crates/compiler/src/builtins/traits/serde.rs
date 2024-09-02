use crate::ast::Name;
use crate::ast::Type;
use crate::builtins::types::array::Array;
use crate::builtins::types::record::Record;
use crate::builtins::types::tuple::Tuple;
use crate::builtins::value::Value;
use crate::span::Span;
use crate::symbol::Symbol;
use crate::Compiler;
use serde::de::DeserializeSeed;
use serde::de::MapAccess;
use serde::de::VariantAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::collections::HashMap;
use std::rc::Rc;

impl Compiler {
    pub(super) fn declare_serde(&mut self) {
        self.declare_trait("trait Serde[T] { }");
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Aggregator(_) => unreachable!(),
            Value::Array(v) => v.serialize(serializer),
            Value::Blob(v) => v.serialize(serializer),
            Value::Bool(v) => v.serialize(serializer),
            Value::Char(v) => v.serialize(serializer),
            Value::Dict(v) => v.serialize(serializer),
            Value::Assigner(v) => v.serialize(serializer),
            Value::Duration(v) => v.serialize(serializer),
            Value::Encoding(v) => v.serialize(serializer),
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
            Value::String(v) => v.serialize(serializer),
            Value::Time(v) => v.serialize(serializer),
            Value::TimeSource(_) => unreachable!(),
            Value::Tuple(v) => v.serialize(serializer),
            Value::U128(v) => v.serialize(serializer),
            Value::U16(v) => v.serialize(serializer),
            Value::U32(v) => v.serialize(serializer),
            Value::U64(v) => v.serialize(serializer),
            Value::U8(v) => v.serialize(serializer),
            Value::Usize(v) => v.serialize(serializer),
            Value::Variant(v) => v.serialize(serializer),
            Value::Vec(v) => v.serialize(serializer),
            Value::Writer(v) => v.serialize(serializer),
            Value::Dataflow(_) => unreachable!(),
            Value::Instance(_) => unreachable!(),
            Value::Ordering(_) => unreachable!(),
            Value::Backend(_) => unreachable!(),
            Value::Range(v) => v.serialize(serializer),
        }
    }
}

impl<'de> DeserializeSeed<'de> for Type {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self {
            Type::Fun(_, _) => unreachable!(),
            Type::Tuple(ts) if ts.is_empty() => <() as Deserialize>::deserialize(deserializer)
                .map(|()| Value::from(Tuple::new(vec![]))),
            Type::Tuple(ts) => {
                struct TupleVisitor(Vec<Type>);
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
                            v.push(seq.next_element_seed(t)?.unwrap());
                        }
                        Ok(Value::from(Tuple::new(v)))
                    }
                }
                deserializer.deserialize_tuple(ts.len(), TupleVisitor(ts.clone()))
            }
            Type::Record(xts) => {
                struct RecordVisitor(HashMap<Name, Type>);
                impl<'de> Visitor<'de> for RecordVisitor {
                    type Value = Value;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "a record with fields {:?}", self.0)
                    }

                    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        let mut result = HashMap::new();
                        while !self.0.is_empty() {
                            let k = map.next_key()?.unwrap();
                            if let Some(t) = self.0.remove(&k) {
                                result.insert(k, map.next_value_seed(t)?);
                            } else {
                                return Err(serde::de::Error::custom("Found unexpected field"));
                            }
                        }
                        Ok(Value::from(Record::new(result.into_iter().collect())))
                    }
                }
                deserializer.deserialize_map(RecordVisitor(xts.into_iter().collect()))
            }
            Type::Cons(x, ts) => match x.data.as_str() {
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
                    struct DictVisitor(Type, Type);
                    impl<'de> Visitor<'de> for DictVisitor {
                        type Value = Value;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            write!(formatter, "a dict")
                        }

                        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                        where
                            A: MapAccess<'de>,
                        {
                            #[allow(clippy::mutable_key_type)]
                            let mut result = std::collections::HashMap::default();
                            while let Some((k, v)) =
                                map.next_entry_seed(self.0.clone(), self.1.clone())?
                            {
                                result.insert(k, v);
                            }
                            Ok(Value::from(runtime::builtins::dict::Dict::from(result)))
                        }
                    }
                    let k = ts[0].clone();
                    let v = ts[1].clone();
                    deserializer.deserialize_map(DictVisitor(k, v))
                }
                "Set" => {
                    struct SetVisitor(Type);
                    impl<'de> Visitor<'de> for SetVisitor {
                        type Value = Value;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            write!(formatter, "a set")
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            #[allow(clippy::mutable_key_type)]
                            let mut result = std::collections::HashSet::new();
                            while let Some(v) = seq.next_element_seed(self.0.clone())? {
                                result.insert(v);
                            }
                            Ok(Value::from(runtime::builtins::set::Set::from(result)))
                        }
                    }
                    let t = ts[0].clone();
                    deserializer.deserialize_seq(SetVisitor(t))
                }
                "Time" => runtime::builtins::time::Time::deserialize(deserializer).map(Value::from),
                "Duration" => runtime::builtins::duration::Duration::deserialize(deserializer)
                    .map(Value::from),
                // "Url" => runtime::builtins::url::Url::deserialize(deserializer).map(Value::from),
                "Path" => runtime::builtins::path::Path::deserialize(deserializer).map(Value::from),
                "Blob" => runtime::builtins::blob::Blob::deserialize(deserializer).map(Value::from),
                "Option" => {
                    struct OptionVisitor(Type);
                    impl<'de> Visitor<'de> for OptionVisitor {
                        type Value = Value;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            write!(formatter, "an option")
                        }

                        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: Deserializer<'de>,
                        {
                            let v = self.0.deserialize(deserializer)?;
                            Ok(Value::from(runtime::builtins::option::Option::some(
                                Rc::new(v),
                            )))
                        }

                        fn visit_none<E>(self) -> Result<Self::Value, E> {
                            Ok(Value::from(runtime::builtins::option::Option::none()))
                        }
                    }
                    let t = ts[0].clone();
                    deserializer.deserialize_option(OptionVisitor(t))
                }
                "Result" => {
                    struct ResultVisitor(Type);
                    impl<'de> Visitor<'de> for ResultVisitor {
                        type Value = Value;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            write!(formatter, "a result")
                        }

                        fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::EnumAccess<'de>,
                        {
                            let (v, variant) = data.variant()?;
                            match v {
                                "Ok" => {
                                    let v = variant.newtype_variant_seed(self.0.clone())?;
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
                    let t = ts[0].clone();
                    deserializer.deserialize_enum("Result", &["Ok", "Err"], ResultVisitor(t))
                }
                "Vec" => {
                    struct VecVisitor(Type);
                    impl<'de> Visitor<'de> for VecVisitor {
                        type Value = Value;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            write!(formatter, "a vec")
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            let mut result = Vec::new();
                            while let Some(v) = seq.next_element_seed(self.0.clone())? {
                                result.push(v);
                            }
                            Ok(Value::from(runtime::builtins::vec::Vec::from(result)))
                        }
                    }
                    let t = ts[0].clone();
                    deserializer.deserialize_seq(VecVisitor(t))
                }
                // "Matrix" => {
                //     let Type::Cons(x, _) = &ts[0] else {
                //         unreachable!()
                //     };
                //     let m = match x.data.as_str() {
                //         "i8" => Matrix::I8(runtime::builtins::matrix::Matrix::<i8>::deserialize(
                //             deserializer,
                //         )?),
                //         "i16" => Matrix::I16(
                //             runtime::builtins::matrix::Matrix::<i16>::deserialize(deserializer)?,
                //         ),
                //         "i32" => Matrix::I32(
                //             runtime::builtins::matrix::Matrix::<i32>::deserialize(deserializer)?,
                //         ),
                //         "i64" => Matrix::I64(
                //             runtime::builtins::matrix::Matrix::<i64>::deserialize(deserializer)?,
                //         ),
                //         "u8" => Matrix::U8(runtime::builtins::matrix::Matrix::<u8>::deserialize(
                //             deserializer,
                //         )?),
                //         "u16" => Matrix::U16(
                //             runtime::builtins::matrix::Matrix::<u16>::deserialize(deserializer)?,
                //         ),
                //         "u32" => Matrix::U32(
                //             runtime::builtins::matrix::Matrix::<u32>::deserialize(deserializer)?,
                //         ),
                //         "u64" => Matrix::U64(
                //             runtime::builtins::matrix::Matrix::<u64>::deserialize(deserializer)?,
                //         ),
                //         "f32" => Matrix::F32(
                //             runtime::builtins::matrix::Matrix::<f32>::deserialize(deserializer)?,
                //         ),
                //         "f64" => Matrix::F64(
                //             runtime::builtins::matrix::Matrix::<f64>::deserialize(deserializer)?,
                //         ),
                //         _ => unreachable!(),
                //     };
                //     Ok(Value::from(m))
                // }
                _ => unreachable!("Attempted to deserialize undeserializable type {:?}", x),
            },
            Type::Generic(_) => unreachable!(),
            Type::Array(t, n) => {
                struct ArrayVisitor(Type);
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
                        while let Some(v) = seq.next_element_seed(self.0.clone())? {
                            result.push(v);
                        }
                        Ok(Value::from(Array(result)))
                    }
                }
                let n = n.unwrap();
                deserializer.deserialize_tuple(n, ArrayVisitor(t.as_ref().clone()))
            }
            Type::Never => unreachable!(),
            Type::Var(_) => Err(serde::de::Error::custom(
                "Attempted to deserialize a type variable",
            )),
            Type::Err => unreachable!(),
            Type::Alias(_, _) => unreachable!(),
            Type::Assoc(_, _, _) => unreachable!(),
            Type::Unknown => unreachable!(),
            Type::Path(_) => unreachable!(),
            Type::Paren(_) => unreachable!(),
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
