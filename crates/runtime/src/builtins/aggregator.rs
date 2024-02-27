#![allow(unused)]

use serde::Deserialize;
use serde::Serialize;

use crate::builtins::vec::Vec;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub enum Aggregator<F0, F1, F2, F3> {
    Incremental {
        lift: F0,
        combine: F1,
        lower: F2,
    },
    Holistic {
        compute: F3,
    },
}

impl<I, P, O> Aggregator<fn(I) -> P, fn(P, P) -> P, fn(P) -> O, fn(Vec<I>) -> O> {
    pub fn incremental(
        lift: fn(I) -> P,
        combine: fn(P, P) -> P,
        lower: fn(P) -> O,
    ) -> Self {
        Self::Incremental {
            lift,
            combine,
            lower,
        }
    }
}

impl<I, O> Aggregator<fn(I) -> (), fn((), ()) -> (), fn() -> (), fn(Vec<I>) -> O> {
    pub fn holistic(compute: fn(Vec<I>) -> O) -> Self {
        Self::Holistic { compute }
    }
}
