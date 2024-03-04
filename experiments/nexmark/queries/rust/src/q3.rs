use runtime::prelude::*;

use crate::data::Auction;
use crate::data::Person;

#[data]
struct Output {
    name: String,
    city: String,
    state: String,
    id: u64,
}

#[data]
struct PrunedAuction {
    id: u64,
    seller: u64,
}

#[data]
struct PrunedPerson {
    id: u64,
    name: String,
    city: String,
    state: String,
}

const SIZE: Duration = Duration::from_seconds(10);

pub fn run(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    auctions
        .tumbling_window_join(
            ctx,
            persons,
            |auction| auction.seller,
            |person| person.id,
            SIZE,
            |auction, person| (auction.clone(), person.clone()),
        )
        .filter(ctx, |(auction, person)| {
            (person.state == "or" || person.state == "id" || person.state == "ca")
                && auction.category == 10
        })
        .map(ctx, |(auction, person)| {
            Output::new(person.name, person.city, person.state, auction.id)
        })
        .drain(ctx);
}

// Opts:
// * Data pruning
// * Predicate pushdown
// * Operator fusion
pub fn run_opt(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    let persons2 = persons.filter_map(ctx, |p| {
        if p.state == "or" || p.state == "id" || p.state == "ca" {
            Option::Some(PrunedPerson::new(p.id, p.name, p.city, p.state))
        } else {
            Option::None
        }
    });
    let auctions2 = auctions.filter_map(ctx, |a| {
        if a.category == 10 {
            Option::Some(PrunedAuction::new(a.id, a.seller))
        } else {
            Option::None
        }
    });
    auctions2
        .tumbling_window_join(
            ctx,
            persons2,
            |a| a.seller,
            |p| p.id,
            SIZE,
            |a, p| Output::new(p.name.clone(), p.city.clone(), p.state.clone(), a.id),
        )
        .drain(ctx);
}
