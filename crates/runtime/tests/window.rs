use runtime::prelude::stream::Event;
use runtime::prelude::*;

#[data]
struct Data<T> {
    value: T,
    time: Time,
}

#[ignore]
#[test]
fn test_tumbling_holistic0() {
    CurrentThreadRunner::run(|ctx| {
        let events = (0..1000).map(|i| Data::new(i, Time::zero()));
        Stream::from_iter(ctx, events, |e| e.time, 100, Duration::zero())
            .time_tumbling_holistic_window(ctx, Duration::from_seconds(100), |data, _| {
                let sum = data.iter().map(|d| d.value).sum::<i64>();
                let min = data.iter().map(|d| d.value).min().unwrap();
                let max = data.iter().map(|d| d.value).max().unwrap();
                (sum, min, max)
            })
            .assert(
                ctx,
                [
                    Event::Watermark(Time::zero()),
                    Event::Data(Time::zero(), (0, 0, 0)),
                    Event::Data(Time::zero(), (0, 0, 0)),
                    Event::Data(Time::zero(), (0, 0, 0)),
                ],
            );
    });
}
