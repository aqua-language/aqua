use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn filter(
        mut self,
        ctx: &mut Context,
        f: impl Fn(&T) -> bool + Send + 'static,
    ) -> KeyedStream<K, T> {
        ctx.keyed_operator(move |tx| async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k, v) => {
                        if f(&v) {
                            tx.send(KeyedEvent::Data(t, k, v)).await?;
                        }
                    }
                    KeyedEvent::Watermark(t) => tx.send(KeyedEvent::Watermark(t)).await?,
                    KeyedEvent::Snapshot(i) => tx.send(KeyedEvent::Snapshot(i)).await?,
                    KeyedEvent::Sentinel => {
                        tx.send(KeyedEvent::Sentinel).await?;
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}
