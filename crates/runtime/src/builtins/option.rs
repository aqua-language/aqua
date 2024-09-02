use macros::DeepClone;
use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;

#[derive(Clone, DeepClone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct Option<T>(pub std::option::Option<T>);

impl<T: std::fmt::Display> std::fmt::Display for Option<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            Some(x) => write!(f, "Some({})", x),
            None => write!(f, "None"),
        }
    }
}

impl<T> Option<T> {
    #[inline(always)]
    pub fn some(x: T) -> Self {
        Self(Some(x))
    }

    #[inline(always)]
    pub fn none() -> Self {
        Self(None)
    }

    pub fn is_some(self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(self) -> bool {
        self.0.is_none()
    }

    pub fn unwrap(self) -> T {
        self.0.unwrap()
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
        self.0.map(f).into()
    }
}

impl<T> From<std::option::Option<T>> for Option<T> {
    fn from(x: std::option::Option<T>) -> Self {
        Self(x)
    }
}
