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
    pub fn time_sliding_aligned_holistic_window<O>(
        mut self,
        ctx: &mut Context,
        size: Duration,
        slide: Duration,
        compute: impl for<'a, 'b> Fn(&K, Window<'a, 'b, T>, WindowRange) -> O + Send + 'static,
    ) -> KeyedStream<K, O>
    where
        O: Data,
    {
        ctx.keyed_operator(|tx| async move {
            let mut slices: BTreeMap<Time, HashMap<K, Vec<T>>> = BTreeMap::new();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(time, key, data) => {
                        slices
                            .entry(align(time, slide))
                            .or_default()
                            .entry(key)
                            .or_default()
                            .push(data);
                    }
                    KeyedEvent::Watermark(time) => {
                        while let Some((t0, _)) = slices.first_key_value() {
                            let t1 = *t0 + size;
                            let wr = WindowRange::new(*t0, t1);
                            if wr.t1 < time {
                                let mut output: HashMap<K, Vec<&[T]>> = HashMap::default();
                                for (_, kvs) in slices.range(..wr.t1) {
                                    for (k, vs) in kvs {
                                        output.entry(k.clone()).or_default().push(vs);
                                    }
                                }
                                for (k, vs) in output.drain() {
                                    let output = compute(&k, Window::new(vs.as_slice()), wr);
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

pub struct Window<'a, 'b, T> {
    slices: &'a [&'b [T]],
}

impl<'a, 'b, T> Window<'a, 'b, T> {
    fn new(slices: &'a [&'b [T]]) -> Self {
        Self { slices }
    }
}

pub struct WindowIter<'a, 'b, T> {
    slices: &'a [&'b [T]],
    idx: usize,
}

impl<'a, 'b, T> Iterator for WindowIter<'a, 'b, T> {
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.slices.len() {
            let slice = self.slices[self.idx];
            self.idx += 1;
            slice.first()
        } else {
            None
        }
    }
}

impl<'a, 'b, T> IntoIterator for Window<'a, 'b, T> {
    type Item = &'b T;
    type IntoIter = WindowIter<'a, 'b, T>;

    fn into_iter(self) -> Self::IntoIter {
        WindowIter {
            slices: self.slices,
            idx: 0,
        }
    }
}
