use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use clap::Parser;
use csv::WriterBuilder;
use nexmark::config::NexmarkConfig;
use nexmark::event::Event;
use nexmark::event::EventType;

#[derive(Parser, Clone, Debug)]
struct Args {
    /// Number of events to generate.
    #[clap(long, default_value = "2000000")]
    num_events: usize,
    #[clap(long, default_value_t = false)]
    persons: bool,
    #[clap(long, default_value_t = false)]
    auctions: bool,
    #[clap(long, default_value_t = false)]
    bids: bool,
    #[clap(long, default_value = ".")]
    dir: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let conf = NexmarkConfig {
        // Set to 1700000000 (Tue Nov 14 2023 22:13:20 GMT+0000) to make events reproducible.
        base_time: 1700000000,
        ..Default::default()
    };
    let mut total = 0;
    if args.persons {
        total += conf.person_proportion;
    }
    if args.auctions {
        total += conf.auction_proportion;
    }
    if args.bids {
        total += conf.bid_proportion;
    }
    if total == 0 {
        return Err("At least one of --bids, --auctions, --persons must be set".into());
    }
    std::fs::create_dir_all(&args.dir)?;

    for (name, ty, flag, proportion) in [
        ("persons", EventType::Person, args.persons, conf.person_proportion),
        ("auctions", EventType::Auction, args.auctions, conf.auction_proportion),
        ("bids", EventType::Bid, args.bids, conf.bid_proportion),
    ] {
        if flag == false {
            continue;
        }
        let n = args.num_events * proportion / total;
        println!("Generating {}*{}/{} = {} events", args.num_events, proportion, total, n);

        let file = File::create(args.dir.join(name).with_extension("csv"))?;
        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(BufWriter::new(file));
        nexmark::EventGenerator::new(conf.clone())
            .with_type_filter(ty)
            .take(n)
            .enumerate()
            .inspect(|(i, _)| {
                let m = i + 1;
                let p = n / 100;
                if m % p == 10 {
                    let progress = m / p;
                    println!("{name}: {progress}%");
                }
            })
            .try_for_each(|(_, event)| match event {
                Event::Person(row) if ty == EventType::Person => writer.serialize(&row),
                Event::Auction(row) if ty == EventType::Auction => writer.serialize(&row),
                Event::Bid(row) if ty == EventType::Bid => writer.serialize(&row),
                _ => unreachable!(),
            })?;
    }

    Ok(())
}
