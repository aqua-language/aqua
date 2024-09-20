use crate::builtins::keyed_stream::KeyedEvent;
use crate::builtins::keyed_stream::KeyedStream;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn keyby<K: Data>(
        mut self,
        ctx: &mut Context,
        fun: impl Fn(&T) -> K + Send + 'static,
    ) -> KeyedStream<K, T> {
        ctx.keyed_operator(|tx1| async move {
            loop {
                match self.recv().await {
                    Event::Data(t, v) => {
                        let k = fun(&v);
                        tx1.send(KeyedEvent::Data(t, k, v)).await?;
                    }
                    Event::Watermark(t) => {
                        tx1.send(KeyedEvent::Watermark(t)).await?;
                    }
                    Event::Snapshot(i) => {
                        tx1.send(KeyedEvent::Snapshot(i)).await?;
                    }
                    Event::Sentinel => {
                        tx1.send(KeyedEvent::Sentinel).await?;
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}
