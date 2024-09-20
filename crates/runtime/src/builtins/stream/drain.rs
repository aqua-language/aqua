use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn drain(mut self, ctx: &mut Context) {
        ctx.sink(|| async move {
            loop {
                if let Event::Sentinel = self.recv().await {
                    break;
                }
            }
            Ok(())
        });
    }
}
