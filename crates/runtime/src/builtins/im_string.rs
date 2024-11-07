use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;
use crate::SmolStr;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct String(pub(crate) SmolStr);

impl std::fmt::Display for String {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for String {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<&'a str> for String {
    #[inline(always)]
    fn eq(&self, other: &&'a str) -> bool {
        self.0 == *other
    }
}

impl DeepClone for String {
    #[inline(always)]
    fn deep_clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl String {
    #[inline(always)]
    pub fn new<T: AsRef<str>>(s: T) -> Self {
        Self(SmolStr::from(s.as_ref()))
    }
}

impl<'a> From<&'a str> for String {
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        Self(SmolStr::from(s))
    }
}

impl From<std::string::String> for String {
    #[inline(always)]
    fn from(s: std::string::String) -> Self {
        Self(SmolStr::from(s))
    }
}

impl String {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&self) -> Self {
        String::from("")
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut s = std::string::String::with_capacity(self.0.len() + other.0.len());
        s.push_str(&self.0);
        s.push_str(&other.0);
        Self::from(s)
    }
}
