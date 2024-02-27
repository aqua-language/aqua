use runtime::prelude::*;

use crate::data::Bid;

#[data]
pub struct Output {
    pub mean: f64,
    pub stddev: f64,
    pub min: u64,
    pub max: u64,
}

#[data]
pub struct Partial {
    pub sum: u64,
    pub count: u64,
    pub max: u64,
    pub min: u64,
    pub sum_sq: u64,
}

impl Partial {
    pub fn identity() -> Self {
        Self::new(0, 0, u64::MIN, u64::MAX, 0)
    }

    pub fn lift(bid: &Bid) -> Self {
        Self::new(bid.price, 1, bid.price, bid.price, bid.price.pow(2))
    }

    pub fn combine(&self, other: &Self) -> Self {
        Self::new(
            self.sum + other.sum,
            self.count + other.count,
            self.max.max(other.max),
            self.min.min(other.min),
            self.sum_sq + other.sum_sq,
        )
    }

    pub fn lower(&self) -> Output {
        let mean = self.sum as f64 / self.count as f64;
        let variance = (self.sum_sq as f64 / self.count as f64) - (mean * mean);
        let stddev = variance.sqrt();
        Output::new(mean, stddev, self.max, self.min)
    }
}

pub fn run(bids: Stream<Bid>, size: usize, step: usize, ctx: &mut Context) {
    bids.count_sliding_holistic_window(ctx, size, step, |data| {
        let mut sum = 0;
        let mut count = 0;
        let mut min = u64::MAX;
        let mut max = u64::MIN;
        for bid in data.iter() {
            sum += bid.price;
            count += 1;
            min = min.min(bid.price);
            max = max.max(bid.price);
        }
        let mean = sum as f64 / count as f64;

        let mut sum_sq_diff = 0.0;
        for bid in data.iter() {
            let diff = bid.price as f64 - mean;
            sum_sq_diff += diff * diff;
        }
        let variance = sum_sq_diff / count as f64;
        let stddev = variance.sqrt();
        Output::new(mean, stddev, max, min)
    })
    .drain(ctx);
}

pub fn run_opt(bids: Stream<Bid>, size: usize, step: usize, ctx: &mut Context) {
    bids.count_sliding_aligned_commutative_associative_window(
        ctx,
        size,
        step,
        Partial::identity(),
        Partial::lift,
        Partial::combine,
        Partial::lower,
    )
    .drain(ctx);
}
