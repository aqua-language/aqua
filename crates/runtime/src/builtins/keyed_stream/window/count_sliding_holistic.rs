use std::collections::VecDeque;

use crate::builtins::keyed_stream::KeyedEvent;
use crate::builtins::keyed_stream::KeyedStream;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;
use crate::HashMap;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn count_sliding_holistic_window<O>(
        mut self,
        ctx: &mut Context,
        size: usize,
        step: usize,
        compute: impl Fn(&K, Window<T>) -> O + Send + 'static,
    ) -> KeyedStream<K, O>
    where
        O: Data,
    {
        ctx.keyed_operator(move |tx| async move {
            let mut aggs: HashMap<K, VecDeque<T>> = HashMap::default();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(time, key, data) => {
                        let agg = aggs.entry(key.clone()).or_default();
                        agg.push_back(data);
                        if agg.len() == size {
                            let result = compute(&key, Window::new(agg));
                            for _ in 0..step {
                                agg.pop_front();
                            }
                            tx.send(KeyedEvent::Data(time, key, result)).await?;
                        }
                    }
                    KeyedEvent::Watermark(time) => {
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

pub struct Window<'a, T> {
    data: &'a VecDeque<T>,
}

impl<'a, T> Window<'a, T> {
    fn new(data: &'a VecDeque<T>) -> Self {
        Self { data }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'a, T> {
        self.data.iter()
    }
}
