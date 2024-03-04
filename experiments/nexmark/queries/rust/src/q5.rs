use runtime::prelude::*;

use crate::data::Bid;

#[data]
struct Output {
    auction: u64,
}

#[data]
struct PrunedBid {
    auction: u64,
    bidder: u64,
}

const SIZE: Duration = Duration::from_minutes(5);
const SLIDE: Duration = Duration::from_minutes(1);

pub fn run(bids: Stream<Bid>, ctx: &mut Context) {
    bids.keyby(ctx, |b| b.auction)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, |auction, bids, _| {
            (*auction, bids.into_iter().count())
        })
        .unkey(ctx)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, |items, _| {
            let auction = items.iter().max_by_key(|(_, a)| a).unwrap().0;
            Output::new(auction)
        })
        .drain(ctx);
}

// Opts:
// * Data pruning
pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    let bids = bids.map(ctx, |b| PrunedBid::new(b.auction, b.bidder));
    bids.keyby(ctx, |b| b.auction)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, |auction, bids, _| {
            (*auction, bids.into_iter().count())
        })
        .unkey(ctx)
        .time_sliding_aligned_holistic_window(ctx, SIZE, SLIDE, |items, _| {
            let auction = items.iter().max_by_key(|(_, a)| a).unwrap().0;
            Output::new(auction)
        })
        .drain(ctx);
}
