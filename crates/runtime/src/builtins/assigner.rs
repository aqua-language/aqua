use serde::Deserialize;
use serde::Serialize;

use crate::builtins::duration::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C)]
pub enum Assigner {
    Tumbling { length: Duration },
    Sliding { duration: Duration, step: Duration },
    Session { gap: Duration },
    Counting { length: i32 },
    Moving { length: i32, step: i32 },
}

impl std::fmt::Display for Assigner {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Assigner::Tumbling { length } => write!(f, "Tumbling({})", length),
            Assigner::Sliding { duration, step } => write!(f, "Sliding({}, {})", duration, step),
            Assigner::Session { gap } => write!(f, "Session({})", gap),
            Assigner::Counting { length } => write!(f, "Counting({})", length),
            Assigner::Moving { length, step } => write!(f, "Moving({}, {})", length, step),
        }
    }
}

impl Assigner {
    pub fn tumbling(length: Duration) -> Self {
        Self::Tumbling { length }
    }

    pub fn sliding(length: Duration, step: Duration) -> Self {
        Self::Sliding {
            duration: length,
            step,
        }
    }

    pub fn session(gap: Duration) -> Self {
        Self::Session { gap }
    }

    pub fn counting(length: i32) -> Self {
        Self::Counting { length }
    }

    pub fn moving(length: i32, step: i32) -> Self {
        Self::Moving { length, step }
    }
}
