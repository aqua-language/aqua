use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn fork(mut self, ctx: &mut Context) -> (Self, Self) {
        ctx.co_operator(|tx1, tx2| async move {
            loop {
                match self.recv().await {
                    Event::Data(t, v1) => {
                        let v2 = v1.deep_clone();
                        tokio::join!(tx1.send(Event::Data(t, v2)), tx2.send(Event::Data(t, v1)))
                    }
                    Event::Watermark(t) => {
                        tokio::join!(tx1.send(Event::Watermark(t)), tx2.send(Event::Watermark(t)))
                    }
                    Event::Snapshot(i) => {
                        tokio::join!(tx1.send(Event::Snapshot(i)), tx2.send(Event::Snapshot(i)))
                    }
                    Event::Sentinel => {
                        tokio::join!(tx1.send(Event::Sentinel), tx2.send(Event::Sentinel));
                        break;
                    }
                };
            }
        })
    }
}
