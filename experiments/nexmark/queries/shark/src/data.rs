use runtime::prelude::*;

#[data]
pub struct Auction {
    pub id: u64,
    pub item_name: String,
    pub description: String,
    pub initial_bid: u64,
    pub reserve: u64,
    #[timestamp]
    pub date_time: u64,
    pub expires: u64,
    pub seller: u64,
    pub category: u64,
    pub extra: String,
}

#[data]
pub struct Person {
    pub id: u64,
    pub name: String,
    pub email_address: String,
    pub credit_card: String,
    pub city: String,
    pub state: String,
    #[timestamp]
    pub date_time: u64,
    pub extra: String,
}

#[data]
pub struct Bid {
    pub auction: u64,
    pub bidder: u64,
    pub price: u64,
    pub channel: String,
    pub url: String,
    #[timestamp]
    pub date_time: u64,
    pub extra: String,
}
