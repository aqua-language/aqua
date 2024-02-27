use std::collections::BTreeMap;

use crate::builtins::duration::Duration;
use crate::builtins::keyed_stream::KeyedEvent;
use crate::builtins::keyed_stream::KeyedStream;
use crate::builtins::stream::window::WindowRange;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;
use crate::HashMap;

impl<K: Key, T: Data> KeyedStream<K, T> {
    // Requires that duration % step == 0
    #[allow(clippy::too_many_arguments)]
    pub fn time_sliding_aligned_commutative_associative_window<P, O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        step: Duration,
        _init: P,
        lift: impl Fn(&T) -> P + Send + 'static,
        combine: impl Fn(&P, &P) -> P + Send + 'static,
        lower: impl Fn(&K, &P, WindowRange) -> O + Send + 'static,
    ) -> KeyedStream<K, O>
    where
        O: Data,
        P: Data,
    {
        assert!(duration % step == Duration::from_seconds(0));
        ctx.keyed_operator(|tx| async move {
            let mut slices: BTreeMap<Time, HashMap<K, P>> = BTreeMap::new();
            let mut output: HashMap<K, P> = HashMap::default();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(time, key, data) => {
                        let data = lift(&data);
                        let wr = WindowRange::of(time, step, step);
                        slices
                            .entry(wr.t0)
                            .or_default()
                            .entry(key)
                            .and_modify(|e| *e = combine(e, &data))
                            .or_insert(data);
                    }
                    KeyedEvent::Watermark(time) => {
                        while let Some(entry) = slices.first_entry() {
                            let t0 = *entry.key();
                            let t1 = t0 + duration;
                            let wr = WindowRange::new(t0, t1);
                            if wr.t1 <= time {
                                for (_, kvs) in slices.range(..t1) {
                                    for (k, v) in kvs {
                                        output
                                            .entry(k.clone())
                                            .and_modify(|e| *e = combine(e, v))
                                            .or_insert_with(|| v.clone());
                                    }
                                }
                                for (k, v) in output.drain() {
                                    let output = lower(&k, &v, wr);
                                    tx.send(KeyedEvent::Data(time, k, output.deep_clone()))
                                        .await;
                                }
                                slices.pop_first();
                            } else {
                                break;
                            }
                        }
                        tx.send(KeyedEvent::Watermark(time)).await;
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
