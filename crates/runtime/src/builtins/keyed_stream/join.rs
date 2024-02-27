use crate::builtins::dict::Dict;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn join<R, O>(
        mut self,
        ctx: &mut Context,
        index: Dict<K, R>,
        merge: impl Fn(&T, &R) -> O + Send + 'static,
    ) -> KeyedStream<K, O>
    where
        R: Data,
        O: Data,
    {
        ctx.keyed_operator(|tx| async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k, v0) => {
                        if let Some(v1) = index.0.get(&k) {
                            let v2 = merge(&v0, v1);
                            tx.send(KeyedEvent::Data(t, k, v2)).await;
                        }
                    }
                    KeyedEvent::Watermark(t) => {
                        tx.send(KeyedEvent::Watermark(t)).await;
                    }
                    KeyedEvent::Snapshot(i) => {
                        tx.send(KeyedEvent::Snapshot(i)).await;
                    }
                    KeyedEvent::Sentinel => {
                        tx.send(KeyedEvent::Sentinel).await;
                        break;
                    }
                }
            }
        })
    }
}
