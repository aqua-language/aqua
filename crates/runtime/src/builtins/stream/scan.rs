use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn scan<A: Data>(
        mut self,
        ctx: &mut Context,
        init: A,
        fun: impl Fn(T, A) -> A + Send + 'static,
    ) -> Stream<A> {
        ctx.operator(|tx| async move {
            let mut acc = init;
            loop {
                match self.recv().await {
                    Event::Data(t, v) => {
                        acc = fun(v, acc);
                        tx.send(Event::Data(t, acc.deep_clone())).await;
                    }
                    Event::Watermark(t) => {
                        tx.send(Event::Watermark(t)).await;
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
