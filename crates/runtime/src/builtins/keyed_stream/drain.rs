use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn drain(mut self, ctx: &mut Context) {
        ctx.sink(|| async move {
            loop {
                if let KeyedEvent::Sentinel = self.recv().await {
                    break;
                }
            }
            Ok(())
        });
    }
}
