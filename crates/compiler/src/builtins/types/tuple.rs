use crate::ast::Index;
use crate::Compiler;

use serde::Serialize;

use crate::builtins::value::Value;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Tuple(pub Vec<Value>);

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

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_tuple(&mut self) {}
}
