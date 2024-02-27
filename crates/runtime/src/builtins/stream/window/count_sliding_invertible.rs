use std::collections::VecDeque;

use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::runner::context::Context;
use crate::traits::Data;

impl<T: Data> Stream<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn count_sliding_invertible_window<P, O>(
        mut self,
        ctx: &mut Context,
        size: usize,
        step: usize,
        init: P,
        lift: impl Fn(&T) -> P + Send + 'static,
        combine: impl Fn(&P, &P) -> P + Send + 'static,
        lower: impl Fn(&P) -> O + Send + 'static,
        inverse: impl Fn(&P, &P) -> P + Send + 'static,
    ) -> Stream<O>
    where
        P: Data,
        O: Data,
    {
        ctx.operator(|tx| async move {
            let mut s: P = init;
            let mut vec: VecDeque<P> = VecDeque::new();
            let mut n = 0;
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        let data = lift(&data);
                        s = combine(&s, &data);
                        vec.push_back(data);
                        if n == size {
                            tx.send(Event::Data(time, lower(&s).deep_clone())).await;
                            for _ in 0..step {
                                let data = vec.pop_front().unwrap();
                                s = inverse(&s, &data);
                            }
                            n -= step;
                        } else {
                            n += 1;
                        }
                    }
                    Event::Watermark(time) => {
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
