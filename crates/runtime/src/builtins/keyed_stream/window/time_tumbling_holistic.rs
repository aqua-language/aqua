use crate::builtins::duration::Duration;
use crate::builtins::keyed_stream::KeyedEvent;
use crate::builtins::keyed_stream::KeyedStream;
use crate::builtins::stream::window::align;
use crate::builtins::stream::window::WindowRange;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;
use crate::BTreeMap;
use crate::HashMap;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn time_tumbling_holistic_window<O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        compute: impl for<'a> Fn(&K, &'a [T], WindowRange) -> O + Send + 'static,
    ) -> KeyedStream<K, O>
    where
        O: Data,
    {
        ctx.keyed_operator(move |tx| async move {
            let mut aggs: BTreeMap<Time, HashMap<K, Vec<T>>> = BTreeMap::new();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(time, key, data) => {
                        let t0 = align(time, duration);
                        aggs.entry(t0)
                            .or_default()
                            .entry(key)
                            .or_default()
                            .push(data);
                    }
                    KeyedEvent::Watermark(time) => {
                        while let Some(entry) = aggs.first_entry() {
                            let t0 = *entry.key();
                            let wr = WindowRange::new(t0, t0 + duration);
                            if wr.t1 < time {
                                let kvs = entry.remove();
                                for (key, vs) in kvs {
                                    let data = compute(&key, &vs, wr);
                                    tx.send(KeyedEvent::Data(wr.t1, key, data)).await?;
                                }
                            } else {
                                break;
                            }
                        }
                        tx.send(KeyedEvent::Watermark(time)).await?;
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
