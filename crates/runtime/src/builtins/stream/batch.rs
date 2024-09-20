use crate::builtins::duration::Duration;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn from_iter<I>(
        ctx: &mut Context,
        iter: I,
        f: impl Fn(&T) -> Time + Send + 'static,
        watermark_frequency: usize,
        slack: Duration,
    ) -> Stream<T>
    where
        I: IntoIterator<Item = T> + Send + 'static,
        <I as IntoIterator>::IntoIter: Send + 'static,
    {
        ctx.operator(move |tx| async move {
            let mut latest_time = Time::zero();
            let mut watermark = Time::zero();
            for (i, v) in iter.into_iter().enumerate() {
                let time = f(&v);
                if time < watermark {
                    continue;
                }
                if time > latest_time {
                    latest_time = time;
                }
                if i % watermark_frequency == 0 {
                    watermark = latest_time - slack;
                    tx.send(Event::Watermark(watermark)).await?;
                }
                tx.send(Event::Data(time, v)).await?;
            }
            tx.send(Event::Sentinel).await
        })
    }
}
