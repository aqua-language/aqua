use crate::builtins::keyed_stream::KeyedEvent;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn merge(mut self, ctx: &mut Context, mut other: Self) -> Self {
        ctx.keyed_operator(|tx| async move {
            loop {
                let event = tokio::select! {
                    event = self.recv() => {
                        if let KeyedEvent::Sentinel = event {
                            other.recv().await
                        } else {
                            event
                        }
                    },
                    event = other.recv() => {
                        if let KeyedEvent::Sentinel = event {
                            self.recv().await
                        } else {
                            event
                        }
                    },
                };
                match event {
                    KeyedEvent::Data(t, k1, v1) => {
                        tx.send(KeyedEvent::Data(t, k1, v1)).await?;
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
