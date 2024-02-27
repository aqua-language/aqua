use crate::builtins::vec::Vec;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K, T> KeyedStream<K, T>
where
    K: Key,
    T: Data,
{
    pub fn flat_map<O>(mut self, ctx: &mut Context, f: fn(T) -> Vec<O>) -> KeyedStream<K, O>
    where
        O: Data,
    {
        ctx.keyed_operator(|tx| async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k, v) => {
                        for v in f(v).iter() {
                            tx.send(KeyedEvent::Data(t, k.deep_clone(), v)).await;
                        }
                    }
                    KeyedEvent::Watermark(t) => tx.send(KeyedEvent::Watermark(t)).await,
                    KeyedEvent::Snapshot(i) => tx.send(KeyedEvent::Snapshot(i)).await,
                    KeyedEvent::Sentinel => {
                        tx.send(KeyedEvent::Sentinel).await;
                        break;
                    }
                }
            }
        })
    }
}
