use runtime::prelude::*;

use crate::data::Auction;
use crate::data::Bid;

#[data]
struct Output {
    category: u64,
    avg_price: u64,
}

#[data]
struct PrunedAuction {
    id: u64,
    category: u64,
    expires: u64,
    date_time: u64,
}

#[data]
struct PrunedBid {
    auction: u64,
    price: u64,
    date_time: u64,
}

const SIZE: Duration = Duration::from_seconds(10);

pub fn run(auctions: Stream<Auction>, bids: Stream<Bid>, ctx: &mut Context) {
    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |auction| auction.id,
            |bid| bid.auction,
            SIZE,
            |auction, bid| (auction.clone(), bid.clone()),
        )
        .filter(ctx, |(a, b)| {
            a.date_time <= b.date_time && b.date_time <= a.expires
        })
        .keyby(ctx, |(a, _)| (a.id, a.category))
        .time_tumbling_holistic_window(ctx, SIZE, |(_, category), items, _| {
            let max = items.iter().map(|(_, b)| b.price).max().unwrap();
            (*category, max)
        })
        .keyby(ctx, |(_, category)| *category)
        .time_tumbling_holistic_window(ctx, SIZE, |category, items, _| {
            let sum = items.iter().map(|(_, max)| max).sum::<u64>();
            let count = items.len() as u64;
            Output::new(*category, sum / count)
        })
        .drain(ctx);
}

// Opts:
// * Data pruning
pub fn run_opt(auctions: Stream<Auction>, bids: Stream<Bid>, ctx: &mut Context) {
    let auctions = auctions.map(ctx, |a| {
        PrunedAuction::new(a.id, a.category, a.expires, a.date_time)
    });
    let bids = bids.map(ctx, |b| PrunedBid::new(b.auction, b.price, b.date_time));

    auctions
        .tumbling_window_join(
            ctx,
            bids,
            |auction| auction.id,
            |bid| bid.auction,
            SIZE,
            |auction, bid| (auction.clone(), bid.clone()),
        )
        .filter(ctx, |(a, b)| {
            a.date_time <= b.date_time && b.date_time <= a.expires
        })
        .keyby(ctx, |(a, _)| (a.id, a.category))
        .time_tumbling_holistic_window(ctx, SIZE, |(_, category), items, _| {
            let max = items.iter().map(|(_, b)| b.price).max().unwrap();
            (*category, max)
        })
        .keyby(ctx, |(_, category)| *category)
        .time_tumbling_holistic_window(ctx, SIZE, |category, items, _| {
            let sum = items.iter().map(|(_, max)| max).sum::<u64>();
            let count = items.len() as u64;
            Output::new(*category, sum / count)
        })
        .drain(ctx);
}
