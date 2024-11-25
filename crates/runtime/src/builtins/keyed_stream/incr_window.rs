use crate::HashMap;
use crate::BTreeMap;

use std::collections::hash_map::Entry;
use std::ops::Range;

use crate::builtins::window::Window;
use crate::builtins::duration::Duration;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Properties {
    pub associative: bool,
    pub commutative: bool,
}

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn incr_window<P, O>(
        self,
        ctx: &mut Context,
        assigner: Window,
        lift: impl Fn(&T) -> P + Send + 'static,
        combine: impl Fn(&P, P) -> P + Send + 'static,
        lower: impl Fn(K, &P, Range<Time>) -> O + Send + 'static,
        properties: Properties,
    ) -> KeyedStream<K, O>
    where
        P: Data,
        O: Data,
    {
        match assigner {
            Window::Tumbling { length } => {
                if properties.commutative {
                    self.commutative_tumbling_window(ctx, length, lift, combine, lower)
                } else {
                    todo!()
                }
            }
            Window::Sliding { .. } => todo!(),
            Window::Session { .. } => todo!(),
            Window::Counting { .. } => todo!(),
            Window::Moving { .. } => todo!(),
        }
    }

    fn commutative_tumbling_window<P, O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        lift: impl Fn(&T) -> P + Send + 'static,
        combine: impl Fn(&P, P) -> P + Send + 'static,
        lower: impl Fn(K, &P, Range<Time>) -> O + Send + 'static,
    ) -> KeyedStream<K, O>
    where
        P: Data,
        O: Data,
    {
        ctx.keyed_operator(move |tx1| async move {
            let mut aggs: BTreeMap<Time, HashMap<K, P>> = BTreeMap::new();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(time, key, data) => {
                        let t0 = time.div_floor(duration) * duration;
                        let data = lift(&data);
                        match aggs.entry(t0).or_default().entry(key) {
                            Entry::Occupied(mut entry) => {
                                *entry.get_mut() = combine(entry.get(), data);
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(data);
                            }
                        }
                    }
                    KeyedEvent::Watermark(time) => {
                        while let Some(entry) = aggs.first_entry() {
                            let t0 = *entry.key();
                            let t1 = t0 + duration;
                            if t1 < time {
                                for (key, p) in entry.remove() {
                                    let data = lower(key.clone(), &p, t0..t1);
                                    tx1.send(KeyedEvent::Data(t1, key, data)).await?;
                                }
                                tx1.send(KeyedEvent::Watermark(time)).await?;
                            } else {
                                tx1.send(KeyedEvent::Watermark(time)).await?;
                                break;
                            }
                        }
                    }
                    KeyedEvent::Snapshot(i) => {
                        tx1.send(KeyedEvent::Snapshot(i)).await?;
                    }
                    KeyedEvent::Sentinel => {
                        tx1.send(KeyedEvent::Sentinel).await?;
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}
