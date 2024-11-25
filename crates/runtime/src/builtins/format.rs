use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C)]
pub enum Format {
    Csv { sep: char },
    Json,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Csv { sep } => write!(f, "Csv({})", sep),
            Format::Json => write!(f, "Json"),
        }
    }
}

impl DeepClone for Format {
    fn deep_clone(&self) -> Self {
        self.clone()
    }
}

impl Format {
    pub fn csv(sep: char) -> Self {
        Self::Csv { sep }
    }
    pub fn json() -> Self {
        Self::Json
    }
}
