pub mod data;

pub mod q1;
pub mod q2;
pub mod q3;
pub mod q4;
pub mod q5;
pub mod q6;
pub mod q7;
pub mod q8;
pub mod qw;

use std::fs::File;
use std::io::BufRead;

use runtime::prelude::formats::csv;
use runtime::prelude::*;
use runtime::traits::Timestamp;

use crate::data::Auction;
use crate::data::Bid;
use crate::data::Person;

const WATERMARK_FREQUENCY: usize = 1000;
const SLACK: Duration = Duration::from_milliseconds(100);

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("No `dir` specified");
    let query = args.next().expect("No `query` specified");

    let bids = std::fs::File::open(&format!("{dir}/bids.csv")).map(iter::<Bid>);
    let auctions = std::fs::File::open(&format!("{dir}/auctions.csv")).map(iter::<Auction>);
    let persons = std::fs::File::open(&format!("{dir}/persons.csv")).map(iter::<Person>);

    fn timed(f: impl FnOnce(&mut Context) + Send + 'static) {
        let time = std::time::Instant::now();
        CurrentThreadRunner::run(f);
        eprintln!("{}", time.elapsed().as_millis());
    }

    match query.as_str() {
        // Un-optimised
        "q1" => timed(move |ctx| q1::run(stream(ctx, bids), ctx)),
        "q2" => timed(move |ctx| q2::run(stream(ctx, bids), ctx)),
        "q3" => timed(move |ctx| q3::run(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "q4" => timed(move |ctx| q4::run(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q5" => timed(move |ctx| q5::run(stream(ctx, bids), ctx)),
        "q6" => timed(move |ctx| q6::run(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q7" => timed(move |ctx| q7::run(stream(ctx, bids), ctx)),
        "q8" => timed(move |ctx| q8::run(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "qw" => {
            let size = args.next().unwrap().parse().unwrap();
            let step = args.next().unwrap().parse().unwrap();
            timed(move |ctx| qw::run(stream(ctx, bids), size, step, ctx))
        }
        // Optimised
        "q1-opt" => timed(move |ctx| q1::run_opt(stream(ctx, bids), ctx)),
        "q2-opt" => timed(move |ctx| q2::run_opt(stream(ctx, bids), ctx)),
        "q3-opt" => timed(move |ctx| q3::run_opt(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "q4-opt" => timed(move |ctx| q4::run_opt(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q5-opt" => timed(move |ctx| q5::run_opt(stream(ctx, bids), ctx)),
        "q6-opt" => timed(move |ctx| q6::run_opt(stream(ctx, auctions), stream(ctx, bids), ctx)),
        "q7-opt" => timed(move |ctx| q7::run_opt(stream(ctx, bids), ctx)),
        "q8-opt" => timed(move |ctx| q8::run_opt(stream(ctx, auctions), stream(ctx, persons), ctx)),
        "qw-opt" => {
            let size = args.next().unwrap().parse().unwrap();
            let step = args.next().unwrap().parse().unwrap();
            timed(move |ctx| qw::run_opt(stream(ctx, bids), size, step, ctx))
        }
        "io" => {
            timed(move |ctx| {
                if bids.is_ok() {
                    stream(ctx, bids).drain(ctx);
                }
                if persons.is_ok() {
                    stream(ctx, persons).drain(ctx);
                }
                if auctions.is_ok() {
                    stream(ctx, auctions).drain(ctx);
                }
            });
        }
        _ => panic!("unknown query"),
    }
}

// Memory-mapped CSV reader
#[allow(unused)]
fn iter_mmap<T: Data>(file: File) -> impl Iterator<Item = T> {
    let mmap = unsafe {
        memmap2::MmapOptions::new()
            .map(&file)
            .expect("Unable to map file")
    };
    mmap.advise(memmap2::Advice::Sequential).unwrap();
    let mut reader = csv::de::Reader::<1024>::new(',');
    let mut i = 0;
    std::iter::from_fn(move || {
        if i >= mmap.len() {
            return None;
        }
        let mut deserializer = csv::de::Deserializer::new(&mut reader, &mmap[i..]);
        let bid = T::deserialize(&mut deserializer).unwrap();
        i += deserializer.nread;
        Some(bid)
    })
}

// Buffered CSV reader
fn iter<T: Data>(file: File) -> impl Iterator<Item = T> {
    let mut bufreader = std::io::BufReader::new(file);
    let mut buf = std::vec::Vec::new();
    let mut reader = csv::de::Reader::<1024>::new(',');
    std::iter::from_fn(move || match bufreader.read_until(b'\n', &mut buf) {
        Ok(0) => None,
        Ok(n) => {
            let mut de = csv::de::Deserializer::new(&mut reader, &buf[0..n]);
            match T::deserialize(&mut de) {
                Ok(data) => {
                    buf.clear();
                    Some(data)
                }
                Err(e) => panic!("Failed to deserialize: {}", e),
            }
        }
        Err(e) => panic!("Failed to read from stdin: {}", e),
    })
}

// Stream from iterator
fn stream<T: Data + Timestamp>(
    ctx: &mut Context,
    iter: std::io::Result<impl Iterator<Item = T> + Send + 'static>,
) -> Stream<T> {
    Stream::from_iter(ctx, iter.unwrap(), T::timestamp, WATERMARK_FREQUENCY, SLACK)
}
