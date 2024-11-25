use serde::Deserialize;
use serde::Serialize;

use crate::builtins::duration::Duration;
use crate::traits::DeepClone;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C)]
pub enum Window {
    Tumbling { length: Duration },
    Sliding { duration: Duration, step: Duration },
    Session { gap: Duration },
    Counting { length: i32 },
    Moving { length: i32, step: i32 },
}

impl DeepClone for Window {
    fn deep_clone(&self) -> Self {
        self.clone()
    }
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Window::Tumbling { length } => write!(f, "Tumbling({})", length),
            Window::Sliding { duration, step } => write!(f, "Sliding({}, {})", duration, step),
            Window::Session { gap } => write!(f, "Session({})", gap),
            Window::Counting { length } => write!(f, "Counting({})", length),
            Window::Moving { length, step } => write!(f, "Moving({}, {})", length, step),
        }
    }
}

impl Window {
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
