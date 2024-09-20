use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct Range<T> {
    pub start: Option<T>,
    pub end: Option<T>,
}

impl<T: DeepClone> DeepClone for Range<T> {
    fn deep_clone(&self) -> Self {
        Range {
            start: self.start.deep_clone(),
            end: self.end.deep_clone(),
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Range<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (&self.start, &self.end) {
            (Some(start), Some(end)) => write!(f, "{start}..{end}"),
            (Some(start), None) => write!(f, "{start}.."),
            (None, Some(end)) => write!(f, "..{end}"),
            (None, None) => write!(f, ".."),
        }
    }
}
