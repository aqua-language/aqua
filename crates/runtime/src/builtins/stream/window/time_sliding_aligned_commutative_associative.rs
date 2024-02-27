use std::collections::BTreeMap;

use crate::builtins::duration::Duration;
use crate::builtins::stream::window::align;
use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;

use super::WindowRange;

impl<T: Data> Stream<T> {
    // Requires that duration % step == 0
    #[allow(clippy::too_many_arguments)]
    pub fn time_sliding_aligned_commutative_associative_window<P, O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        step: Duration,
        _init: P,
        lift: impl Fn(&T) -> P + Send + 'static,
        combine: impl Fn(&P, &P) -> P + Send + 'static,
        lower: impl Fn(&P, WindowRange) -> O + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
        P: Data,
    {
        assert!(duration % step == Duration::from_seconds(0));
        ctx.operator(|tx| async move {
            let mut slices: BTreeMap<Time, P> = BTreeMap::new();
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        let data = lift(&data);
                        slices
                            .entry(align(time, step))
                            .and_modify(|agg| *agg = combine(agg, &data))
                            .or_insert(data);
                    }
                    Event::Watermark(time) => {
                        while let Some(entry) = slices.first_entry() {
                            let t0 = *entry.key();
                            let t1 = t0 + duration;
                            let wr = WindowRange::new(t0, t1);
                            if wr.t1 <= time {
                                let mut agg = entry.remove();
                                for (_, v) in slices.range(..t1) {
                                    agg = combine(&agg, v);
                                }
                                let output = lower(&agg, wr);
                                tx.send(Event::Data(time, output.deep_clone())).await;
                            } else {
                                break;
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
