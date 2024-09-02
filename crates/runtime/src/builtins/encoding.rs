use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C)]
pub enum Encoding {
    Csv { sep: char },
    Json,
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Encoding::Csv { sep } => write!(f, "Csv({})", sep),
            Encoding::Json => write!(f, "Json"),
        }
    }
}

impl Encoding {
    pub fn csv(sep: char) -> Self {
        Self::Csv { sep }
    }
    pub fn json() -> Self {
        Self::Json
    }
}
