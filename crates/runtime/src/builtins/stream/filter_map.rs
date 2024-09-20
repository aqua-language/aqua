// use crate::builtins::option::Option;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn filter_map<O>(
        mut self,
        ctx: &mut Context,
        f: impl Fn(T) -> Option<O> + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
    {
        ctx.operator(|tx| async move {
            loop {
                match self.recv().await {
                    Event::Data(t, v) => {
                        if let Some(v) = f(v) {
                            tx.send(Event::Data(t, v)).await?;
                        }
                    }
                    Event::Watermark(t) => {
                        tx.send(Event::Watermark(t)).await?;
                    }
                    Event::Snapshot(i) => {
                        tx.send(Event::Snapshot(i)).await?;
                    }
                    Event::Sentinel => {
                        tx.send(Event::Sentinel).await?;
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}
