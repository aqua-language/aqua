use serde::Deserialize;
use serde::Serialize;

use crate::builtins::unchecked_cell::UncheckedCell;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Blob(pub(crate) UncheckedCell<std::vec::Vec<u8>>);

impl std::fmt::Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Blob({:?})", self.0)
    }
}

impl Blob {
    pub fn new(bytes: std::vec::Vec<u8>) -> Self {
        Self(UncheckedCell::new(bytes))
    }
}
