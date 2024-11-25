use runtime::builtins::time::Time;
use runtime::builtins::writer::Writer;
use runtime::prelude::data;
use runtime::prelude::CurrentThreadRunner;
use runtime::prelude::DeepClone;
use runtime::prelude::Duration;
use runtime::prelude::Format;
use runtime::prelude::New;
use runtime::prelude::Path;
use runtime::prelude::Reader;
use runtime::prelude::Send;
use runtime::prelude::Stream;
use runtime::prelude::Timestamp;

#[data]
struct Data {
    x: i32,
    y: i32,
    z: i32,
}

const INPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data.csv");
const OUTPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/output.csv");

#[test]
fn test() {
    CurrentThreadRunner::run(|ctx| {
        let s0 = Stream::<Data>::source(
            ctx,
            Reader::file(Path::new(INPUT), false),
            Format::csv(','),
            |_: Data, t: Time| t,
            Duration::from_seconds(1),
            Duration::from_seconds(1),
        );
        let s1 = Stream::<Data>::filter(s0, ctx, |data: &Data| data.y > 5);
        let _ = Stream::<Data>::sink(s1, ctx, Writer::file(Path::new(OUTPUT)), Format::csv(','));
    });
}
