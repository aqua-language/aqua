use crate::ast::Index;
use crate::builtins::Context;

use crate::builtins::value::Value;
use crate::builtins::DECLS;
use linkme::distributed_slice;
use runtime::prelude::DeepClone;
use serde::Serialize;

#[distributed_slice(DECLS)]
fn declare(_ctx: &mut Context) {}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Tuple(pub Vec<Value>);

impl DeepClone for Tuple {
    fn deep_clone(&self) -> Self {
        Tuple(self.0.iter().map(|v| v.deep_clone()).collect())
    }
}

impl std::fmt::Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        let mut iter = self.0.iter();
        if let Some(v) = iter.next() {
            write!(f, "{}", v)?;
            for v in iter {
                write!(f, ", {}", v)?;
            }
        }
        write!(f, ")")
    }
}

impl std::ops::Add for Tuple {
    type Output = Tuple;

    fn add(self, other: Tuple) -> Tuple {
        assert_eq!(
            self.0.len(),
            other.0.len(),
            "Tuples should be the same length",
        );
        Tuple(
            self.0
                .into_iter()
                .zip(other.0.into_iter())
                .map(|(a, b)| a + b)
                .collect(),
        )
    }
}

impl Tuple {
    pub fn new(values: Vec<Value>) -> Tuple {
        Tuple(values)
    }
}

impl<'a> std::ops::Index<&'a Index> for Tuple {
    type Output = Value;

    fn index(&self, index: &'a Index) -> &Value {
        &self.0[index.data]
    }
}
