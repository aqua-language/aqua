use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;
use crate::try_pair;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn split(mut self, ctx: &mut Context) -> (Self, Self) {
        ctx.keyed_co_operator(|tx1, tx2| async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k1, v1) => {
                        let k2 = k1.deep_clone();
                        let v2 = v1.deep_clone();
                        try_pair!(tokio::join!(
                            tx1.send(KeyedEvent::Data(t, k2, v2)),
                            tx2.send(KeyedEvent::Data(t, k1, v1)),
                        ));
                    }
                    KeyedEvent::Watermark(t) => {
                        try_pair!(tokio::join!(
                            tx1.send(KeyedEvent::Watermark(t)),
                            tx2.send(KeyedEvent::Watermark(t))
                        ));
                    }
                    KeyedEvent::Snapshot(i) => {
                        try_pair!(tokio::join!(
                            tx1.send(KeyedEvent::Snapshot(i)),
                            tx2.send(KeyedEvent::Snapshot(i))
                        ));
                    }
                    KeyedEvent::Sentinel => {
                        try_pair!(tokio::join!(
                            tx1.send(KeyedEvent::Sentinel),
                            tx2.send(KeyedEvent::Sentinel)
                        ));
                        break;
                    }
                };
            }
            Ok(())
        })
    }
}
