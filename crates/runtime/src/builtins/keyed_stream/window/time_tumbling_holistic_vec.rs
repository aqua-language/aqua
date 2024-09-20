use std::collections::VecDeque;

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
    pub fn time_tumbling_holistic_vec_window<O>(
        mut self,
        ctx: &mut Context,
        _duration: Duration,
        _compute: impl for<'a> Fn(&K, &'a [T], WindowRange) -> O + Send + 'static,
    ) -> KeyedStream<K, O>
    where
        O: Data,
    {
        ctx.keyed_operator(|tx| async move {
            let mut _aggs: VecDeque<(Time, HashMap<K, Vec<T>>)> = VecDeque::new();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(_time, _key, _data) => {
                        todo!()
                    }
                    KeyedEvent::Watermark(_time) => {
                        todo!();
                        // tx.send(KeyedEvent::Watermark(time)).await;
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
