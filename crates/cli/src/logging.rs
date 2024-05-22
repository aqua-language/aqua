use std::str::FromStr;

use tracing_subscriber::prelude::*;

#[cfg(debug_assertions)]
pub const FILTER: &str = "info,librdkafka=off,rdkafka::client=off";

#[cfg(not(debug_assertions))]
pub const FILTER: &str = "warn,librdkafka=off,rdkafka::client=off";

pub fn init() {
    let fmt = tracing_subscriber::fmt::layer()
        .with_file(false)
        .with_line_number(false)
        .compact()
        .with_writer(std::io::stderr)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_str(FILTER).unwrap());
    tracing_subscriber::registry().with(fmt).init();
}
