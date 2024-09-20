#![allow(unused)]
use crate::HashMap;

use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn scan<P, O>(
        mut self,
        ctx: &mut Context,
        lift: impl Fn(&T) -> P + Send + 'static,
        combine: impl Fn(&P, P) -> P + Send + 'static,
        lower: impl Fn(K, &P) -> O + Send + 'static,
    ) -> KeyedStream<K, O>
    where
        P: Data,
        O: Data,
    {
        ctx.keyed_operator(|tx| async move {
            let state: HashMap<K, P> = HashMap::default();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k, v) => {
                        todo!()
                        // let p = state.entry(k.clone()).or_insert_with(identity);
                        // *p = combine(p.clone(), lift(v));
                        // tx1.send(KeyedEvent::Data(t, k, lower(p.clone())))
                        //     .await
                        //     .unwrap();
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
