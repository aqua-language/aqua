use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn unkey(mut self, ctx: &mut Context) -> Stream<T> {
        ctx.operator(|tx| async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, _, v) => {
                        tx.send(Event::Data(t, v)).await?;
                    }
                    KeyedEvent::Watermark(t) => {
                        tx.send(Event::Watermark(t)).await?;
                    }
                    KeyedEvent::Snapshot(i) => {
                        tx.send(Event::Snapshot(i)).await?;
                    }
                    KeyedEvent::Sentinel => {
                        tx.send(Event::Sentinel).await?;
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}
