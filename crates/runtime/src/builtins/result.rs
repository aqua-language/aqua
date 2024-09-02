use macros::DeepClone;

use serde::Deserialize;
use serde::Serialize;

use crate::builtins::string::String;
use crate::traits::DeepClone;

#[derive(Debug, DeepClone, Clone, Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Result<T>(pub std::result::Result<T, String>);

impl<T:std::fmt::Display> std::fmt::Display for Result<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            std::result::Result::Ok(x) => write!(f, "Ok({})", x),
            std::result::Result::Err(x) => write!(f, "Error({})", x),
        }
    }
}

impl<T> Result<T> {
    pub fn ok(x: T) -> Self {
        Self(std::result::Result::Ok(x))
    }

    pub fn error(x: String) -> Self {
        Self(std::result::Result::Err(x))
    }

    pub fn is_ok(self) -> bool {
        matches!(self.0, std::result::Result::Ok(_))
    }

    pub fn is_error(self) -> bool {
        matches!(self.0, std::result::Result::Err(_))
    }

    pub fn unwrap_ok(self) -> T {
        match self.0 {
            std::result::Result::Ok(x) => x,
            std::result::Result::Err(_) => unreachable!(),
        }
    }

    pub fn unwrap_error(self) -> String {
        match self.0 {
            std::result::Result::Ok(_) => unreachable!(),
            std::result::Result::Err(x) => x,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Result<U> {
        Result(self.0.map(f))
    }
}

impl<T, E: std::error::Error> From<std::result::Result<T, E>> for Result<T> {
    fn from(x: std::result::Result<T, E>) -> Self {
        Self(x.map_err(|x| String::from(x.to_string())))
    }
}
