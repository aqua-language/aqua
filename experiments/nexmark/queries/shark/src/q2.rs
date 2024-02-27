use runtime::prelude::*;

use crate::data::Bid;

#[data]
struct Output {
    auction: u64,
    price: u64,
}

pub fn run(bids: Stream<Bid>, ctx: &mut Context) {
    bids.filter(ctx, |bid| {
        bid.auction == 1007
            || bid.auction == 1020
            || bid.auction == 2001
            || bid.auction == 2019
            || bid.auction == 2087
    })
    .map(ctx, |bid| Output::new(bid.auction, bid.price))
    .drain(ctx);
}

// Opt:
// * Fusion
pub fn run_opt(bids: Stream<Bid>, ctx: &mut Context) {
    bids.filter_map(ctx, |bid| {
        if bid.auction == 1007
            || bid.auction == 1020
            || bid.auction == 2001
            || bid.auction == 2019
            || bid.auction == 2087
        {
            Option::Some(Output::new(bid.auction, bid.price))
        } else {
            Option::None
        }
    })
    .drain(ctx);
}
