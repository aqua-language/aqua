#![allow(unused)]

use serde::Deserialize;
use serde::Serialize;

use crate::builtins::vec::Vec;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub enum Aggregator<F0, F1, F2, F3> {
    Incremental { lift: F0, combine: F1, lower: F2 },
    Holistic { compute: F3 },
}

impl<
        F0: std::fmt::Display,
        F1: std::fmt::Display,
        F2: std::fmt::Display,
        F3: std::fmt::Display,
    > std::fmt::Display for Aggregator<F0, F1, F2, F3>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Aggregator::Incremental {
                lift,
                combine,
                lower,
            } => write!(
                f,
                "Incremental(lift={}, combine={}, lower={})",
                lift, combine, lower
            ),
            Aggregator::Holistic { compute } => write!(f, "Holistic(compute={})", compute),
        }
    }
}

impl<I, P, O> Aggregator<fn(I) -> P, fn(P, P) -> P, fn(P) -> O, fn(Vec<I>) -> O> {
    pub fn incremental(lift: fn(I) -> P, combine: fn(P, P) -> P, lower: fn(P) -> O) -> Self {
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
