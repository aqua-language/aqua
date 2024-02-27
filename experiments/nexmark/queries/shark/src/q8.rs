use runtime::prelude::*;

use crate::data::Auction;
use crate::data::Person;

#[data]
struct Output {
    person: u64,
    name: String,
    reserve: u64,
}

#[data]
struct PrunedAuction {
    seller: u64,
    reserve: u64,
}

#[data]
struct PrunedPerson {
    id: u64,
    name: String,
}

const LOWER_BOUND: Duration = Duration::from_minutes(0);
const UPPER_BOUND: Duration = Duration::from_minutes(1);

pub fn run(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    persons
        .interval_join(
            ctx,
            auctions,
            |p| p.id,
            |a| a.seller,
            LOWER_BOUND,
            UPPER_BOUND,
            |p, a| Output::new(p.id, p.name.clone(), a.reserve),
        )
        .drain(ctx);
}

// Opts:
// * Data pruning
// * Interval join => interval join forward
pub fn run_opt(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    let auctions = auctions.map(ctx, |a| PrunedAuction::new(a.seller, a.reserve));
    let persons = persons.map(ctx, |p| PrunedPerson::new(p.id, p.name));
    persons
        .interval_join_forward(
            ctx,
            auctions,
            |p| p.id,
            |a| a.seller,
            UPPER_BOUND,
            |p, a| Output::new(p.id, p.name.clone(), a.reserve),
        )
        .drain(ctx);
}
