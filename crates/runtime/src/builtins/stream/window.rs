use crate::builtins::duration::Duration;
use crate::builtins::time::Time;

pub mod count_sliding_aligned_commutative_associative;
pub mod count_sliding_holistic;
pub mod count_sliding_invertible;
pub mod count_tumbling_holistic;

pub mod time_sliding_aligned_commutative_associative;
pub mod time_sliding_aligned_holistic;
pub mod time_sliding_commutative_invertible;
pub mod time_sliding_invertible;
pub mod time_tumbling_holistic;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowRange {
    pub t0: Time,
    pub t1: Time,
}

impl WindowRange {
    pub fn new(t0: Time, t1: Time) -> Self {
        Self { t0, t1 }
    }

    pub fn of(time: Time, duration: Duration, step: Duration) -> Self {
        let t0 = align(time, step);
        let t1 = t0 + duration;
        Self { t0, t1 }
    }
}

pub fn align(time: Time, step: Duration) -> Time {
    time.div_floor(step) * step
}
