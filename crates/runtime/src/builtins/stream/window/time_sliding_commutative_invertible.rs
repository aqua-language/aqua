use std::collections::BTreeMap;

use crate::builtins::duration::Duration;
use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;

use super::WindowRange;

impl<T: Data> Stream<T> {
    // Properties:
    // * Inverse: We can undo aggregations.
    #[allow(clippy::too_many_arguments)]
    pub fn time_sliding_commutative_invertible_window<P, O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        step: Duration,
        init: P,
        lift: impl Fn(&T) -> P + Send + 'static,
        combine: impl Fn(&P, &P) -> P + Send + 'static,
        lower: impl Fn(&P, WindowRange) -> O + Send + 'static,
        inverse: impl Fn(&P, &P) -> P + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
        P: Data,
    {
        ctx.operator(|tx| async move {
            let mut buffer: BTreeMap<Time, P> = BTreeMap::new();
            let mut first: Option<WindowRange> = None;
            let mut agg: P = init;
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        // With commutativity, we can pre-aggregate data for the first window
                        let data = lift(&data);
                        if let Some(first) = first.as_mut() {
                            if time < first.t1 {
                                agg = combine(&agg, &data);
                            }
                        } else {
                            let wr = WindowRange::of(time, duration, step);
                            first = Some(wr);
                            agg = combine(&agg, &data);
                        }
                        buffer.insert(time, data);
                    }
                    Event::Watermark(time) => {
                        // Process the first window
                        if let Some(wr) = first {
                            if wr.t1 <= time {
                                let data = lower(&agg, wr);
                                tx.send(Event::Data(wr.t1, data.deep_clone())).await;
                                let after = buffer.split_off(&(wr.t0 + step));
                                for (_, p) in std::mem::replace(&mut buffer, after) {
                                    agg = inverse(&agg, &p);
                                }
                                first = None;
                            }
                        }
                        while let Some(entry) = buffer.first_entry() {
                            let wr = WindowRange::of(*entry.key(), duration, step);
                            if wr.t1 <= time {
                                for (_, p) in buffer.range(..wr.t1) {
                                    agg = combine(&agg, p);
                                }
                                let data = lower(&agg, wr);
                                tx.send(Event::Data(wr.t1, data.deep_clone())).await;
                                // Evict the part of the oldest window that is no longer needed.
                                let after = buffer.split_off(&(wr.t0 + step));
                                for (_, p) in std::mem::replace(&mut buffer, after) {
                                    agg = inverse(&agg, &p);
                                }
                            }
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
