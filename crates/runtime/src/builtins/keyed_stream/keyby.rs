use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn keyby<K1: Data>(
        mut self,
        ctx: &mut Context,
        fun: impl Fn(&T) -> K1 + Send + 'static,
    ) -> KeyedStream<K1, T> {
        ctx.keyed_operator(|tx| async move {
            loop {
                match self.0.recv().await.unwrap() {
                    KeyedEvent::Data(t, _, v) => {
                        let k = fun(&v);
                        tx.send(KeyedEvent::Data(t, k, v)).await?;
                    }
                    KeyedEvent::Watermark(t) => {
                        tx.send(KeyedEvent::Watermark(t)).await?;
                    }
                    KeyedEvent::Snapshot(i) => {
                        tx.send(KeyedEvent::Snapshot(i)).await?;
                    }
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
