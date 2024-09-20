use std::collections::VecDeque;

use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::runner::context::Context;
use crate::traits::Data;

impl<T: Data> Stream<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn count_sliding_aligned_commutative_associative_window<P, O>(
        mut self,
        ctx: &mut Context,
        size: usize,
        step: usize,
        _init: P,
        lift: impl Fn(&T) -> P + Send + 'static,
        combine: impl Fn(&P, &P) -> P + Send + 'static,
        lower: impl Fn(&P) -> O + Send + 'static,
    ) -> Stream<O>
    where
        P: Data,
        O: Data,
    {
        assert!(size % step == 0);
        ctx.operator(move |tx| async move {
            let mut s: VecDeque<P> = VecDeque::new();
            let mut n = 0;
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        let data = lift(&data);
                        if n % step == 0 {
                            s.push_back(data);
                        } else {
                            let agg = s.back_mut().unwrap();
                            *agg = combine(agg, &data);
                        }
                        if n == size {
                            let agg = s.pop_front().unwrap();
                            let agg = s
                                .range(0..(size / step) - 1)
                                .fold(agg, |agg, data| combine(&agg, data));
                            let data = lower(&agg);
                            tx.send(Event::Data(time, data)).await?;
                            n -= step;
                        } else {
                            n += 1;
                        }
                    }
                    Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
                    Event::Snapshot(i) => tx.send(Event::Snapshot(i)).await?,
                    Event::Sentinel => {
                        tx.send(Event::Sentinel).await?;
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}
