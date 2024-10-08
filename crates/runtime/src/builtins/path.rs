use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct Path(pub std::path::PathBuf);

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl Path {
    pub fn new(path: impl AsRef<str>) -> Self {
        Self(std::path::PathBuf::from(path.as_ref()))
    }

    pub fn join(&self, path: impl AsRef<str>) -> Self {
        Self(self.0.join(path.as_ref()))
    }
}

impl<'a> From<&'a str> for Path {
    fn from(path: &'a str) -> Self {
        Self::new(path)
    }
}

impl From<std::path::PathBuf> for Path {
    fn from(path: std::path::PathBuf) -> Self {
        Self(path)
    }
}
