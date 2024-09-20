use serde::Deserialize;
use serde::Serialize;

use crate::builtins::im_string::String;
use crate::builtins::result::Result;
use crate::traits::DeepClone;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct Url(pub url::Url);

impl DeepClone for Url {
    fn deep_clone(&self) -> Self {
        Url(self.0.clone())
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Url {
    pub fn parse(s: String) -> Result<Self> {
        match url::Url::parse(s.0.as_str()) {
            Ok(v) => Result::ok(Url(v)),
            Err(s) => Result::error(s.to_string().into()),
        }
    }

    pub fn to_string(self) -> String {
        String::from(self.0.to_string())
    }
}
