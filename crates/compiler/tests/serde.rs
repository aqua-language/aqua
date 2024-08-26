use std::rc::Rc;

use compiler::ast::Map;
use compiler::ast::TypeVar;
use serde::de::DeserializeSeed;

use compiler::ast::Type;
use compiler::builtins::Array;
use compiler::builtins::Record;
use compiler::builtins::Tuple;
use compiler::builtins::Value;

use compiler::builtins::de::Seed;

#[test]
fn test_serde_i32() {
    let v0 = Value::from(1);
    let s = serde_json::to_string(&v0).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t = Type::Cons("i32".into(), vec![]);
    let v1 = Seed(t).deserialize(&mut de).unwrap();
    assert_eq!(v0, v1);
    assert_eq!(s, "1");
}

#[test]
fn test_serde_vec() {
    let v0 = Value::from(1);
    let v1 = Value::from(2);
    let v2 = Value::from(3);
    let v3 = Value::from(runtime::builtins::vec::Vec::from(vec![v0, v1, v2]));
    let s = serde_json::to_string(&v3).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Cons("Vec".into(), vec![t0]);
    let v4 = Seed(t1).deserialize(&mut de).unwrap();
    assert_eq!(v3, v4);
    assert_eq!(s, "[1,2,3]");
}

#[test]
fn test_serde_tuple() {
    let v0 = Value::from(1);
    let v1 = Value::from(2);
    let v2 = Value::from(runtime::builtins::im_string::String::from("Hello"));
    let v3 = Value::from(Tuple::new(vec![v0, v1, v2]));
    let s = serde_json::to_string(&v3).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Cons("i32".into(), vec![]);
    let t2 = Type::Cons("String".into(), vec![]);
    let t3 = Type::Tuple(vec![t0, t1, t2]);
    let v4 = Seed(t3).deserialize(&mut de).unwrap();
    assert_eq!(v3, v4);
    assert_eq!(s, r#"[1,2,"Hello"]"#);
}

#[test]
fn test_serde_record() {
    let v0 = Value::from(1);
    let v1 = Value::from(runtime::builtins::im_string::String::from("Hello"));
    let v2 = Value::from(Record::new(Map::from(vec![
        ("a".into(), v0.clone()),
        ("b".into(), v1.clone()),
    ])));
    let v2_permut = Value::from(Record::new(Map::from(vec![
        ("b".into(), v1),
        ("a".into(), v0),
    ])));
    let s = serde_json::to_string(&v2).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Cons("String".into(), vec![]);
    let t2 = Type::Record(Map::from(vec![("a".into(), t0), ("b".into(), t1)]));
    let v3 = Seed(t2).deserialize(&mut de).unwrap();
    assert!(v2 == v3 || v2_permut == v3);
    assert!((s == r#"{"a":1,"b":"Hello"}"#) || (s == r#"{"b":"Hello","a":1}"#));
}

#[test]
fn test_serde_dict() {
    let k0 = Value::from(runtime::builtins::im_string::String::from("a"));
    let k1 = Value::from(runtime::builtins::im_string::String::from("b"));
    let v0 = Value::from(1);
    let v1 = Value::from(2);
    let v2 = Value::from(runtime::builtins::dict::Dict::from(
        vec![(k0, v0), (k1, v1)]
            .into_iter()
            .collect::<runtime::HashMap<_, _>>(),
    ));
    let s = serde_json::to_string(&v2).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("String".into(), vec![]);
    let t1 = Type::Cons("i32".into(), vec![]);
    let t2 = Type::Cons("Dict".into(), vec![t0, t1]);
    let v3 = Seed(t2).deserialize(&mut de).unwrap();
    assert_eq!(v2, v3);
    assert!((s == r#"{"a":1,"b":2}"#) || (s == r#"{"b":2,"a":1}"#));
}

#[test]
fn test_serde_array() {
    let v0 = Value::from(1);
    let v1 = Value::from(2);
    let v2 = Value::from(3);
    let v3 = Value::from(Array(vec![v0, v1, v2]));
    let s = serde_json::to_string(&v3).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Array(Rc::new(t0), Some(3));
    let v4 = Seed(t1).deserialize(&mut de).unwrap();
    assert_eq!(v3, v4);
    assert_eq!(s, "[1,2,3]");
}

#[test]
fn test_serde_set() {
    let v0 = Value::from(1);
    let v1 = Value::from(2);
    let v2 = Value::from(runtime::builtins::set::Set::from(
        vec![v0, v1]
            .into_iter()
            .collect::<std::collections::HashSet<_>>(),
    ));
    let s = serde_json::to_string(&v2).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Cons("Set".into(), vec![t0]);
    let v3 = Seed(t1).deserialize(&mut de).unwrap();
    assert_eq!(v2, v3);
    assert!((s == r#"[1,2]"#) || (s == r#"[2,1]"#));
}

#[test]
fn test_serde_option_some() {
    let v0 = Value::from(1);
    let v1 = Value::from(runtime::builtins::option::Option::some(Rc::new(v0)));
    let s = serde_json::to_string(&v1).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Cons("Option".into(), vec![t0]);
    let v2 = Seed(t1).deserialize(&mut de).unwrap();
    assert_eq!(v1, v2);
    assert_eq!(s, "1");
}

#[test]
fn test_serde_option_none() {
    let v0 = Value::from(runtime::builtins::option::Option::none());
    let s = serde_json::to_string(&v0).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Cons("Option".into(), vec![t0]);
    let v2 = Seed(t1).deserialize(&mut de).unwrap();
    assert_eq!(v0, v2);
    assert_eq!(s, "null");
}

#[test]
fn test_serde_result_ok() {
    let v0 = Value::from(1);
    let v1 = Value::from(runtime::builtins::result::Result::ok(Rc::new(v0)));
    let s = serde_json::to_string(&v1).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Cons("Result".into(), vec![t0]);
    let v2 = Seed(t1).deserialize(&mut de).unwrap();
    assert_eq!(v1, v2);
    assert_eq!(s, r#"{"Ok":1}"#);
}

#[test]
fn test_serde_result_err() {
    let v0 = runtime::builtins::string::String::from("Hello");
    let v1 = Value::from(runtime::builtins::result::Result::error(v0));
    let s = serde_json::to_string(&v1).unwrap();
    let mut de = serde_json::Deserializer::from_str(&s);
    let t0 = Type::Cons("i32".into(), vec![]);
    let t1 = Type::Cons("Result".into(), vec![t0]);
    let v2 = Seed(t1).deserialize(&mut de).unwrap();
    assert_eq!(v1, v2);
    assert_eq!(s, r#"{"Err":"Hello"}"#);
}

// #[test]
// fn test_serde_matrix() {
//     use compiler::builtins::Matrix;
//     let v9 = Value::from(Matrix::I32(runtime::builtins::matrix::Matrix::zeros(
//         vec![2, 2].into(),
//     )));
//     let s = serde_json::to_string(&v9).unwrap();
//     let mut de = serde_json::Deserializer::from_str(&s);
//     let t0 = Type::Cons("i32".into(), vec![]);
//     let t1 = Type::Cons("Matrix".into(), vec![t0]);
//     let v10 = Seed(t1).deserialize(&mut de).unwrap();
//     assert_eq!(v9, v10);
//     assert_eq!(s, r#"{"v":1,"dim":[2,2],"data":[0,0,0,0]}"#);
// }

#[test]
fn test_serde_type_variable() {
    let mut de = serde_json::Deserializer::from_str("1");
    let t = Type::Var(TypeVar(1).into());
    assert!(Seed(t).deserialize(&mut de).is_err());
}

#[test]
fn test_serde_type_error() {
    let mut de = serde_json::Deserializer::from_str("1.0");
    let t = Type::Cons("i32".into(), vec![]);
    assert!(Seed(t).deserialize(&mut de).is_err());
}
