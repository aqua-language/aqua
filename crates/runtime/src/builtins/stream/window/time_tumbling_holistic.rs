use crate::builtins::duration::Duration;
use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::BTreeMap;

use super::align;
use super::WindowRange;

impl<T: Data> Stream<T> {
    pub fn time_tumbling_holistic_window<O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        compute: impl Fn(&[T], WindowRange) -> O + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
    {
        ctx.operator(|tx| async move {
            let mut buffer: BTreeMap<Time, Vec<T>> = BTreeMap::new();
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        let t0 = align(time, duration);
                        buffer.entry(t0).or_default().push(data);
                    }
                    Event::Watermark(time) => {
                        while let Some(entry) = buffer.first_entry() {
                            let t0 = *entry.key();
                            let t1 = t0 + duration;
                            let wr = WindowRange::new(t0, t1);
                            if wr.t1 > time {
                                break;
                            }
                            let vs = entry.remove();
                            let data = compute(&vs, wr);
                            tx.send(Event::Data(wr.t1, data.deep_clone())).await;
                        }
                        tx.send(Event::Watermark(time)).await;
                    }
                    Event::Snapshot(i) => {
                        tx.send(Event::Snapshot(i)).await;
                    }
                    Event::Sentinel => {
                        tx.send(Event::Sentinel).await;
                        break;
                    }
                }
            }
        })
    }
}
