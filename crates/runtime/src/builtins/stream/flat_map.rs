use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T> Stream<T>
where
    T: Data,
{
    pub fn flat_map<O, I>(
        mut self,
        ctx: &mut Context,
        f: impl Fn(T) -> I + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
        I: IntoIterator<Item = O>,
        <I as IntoIterator>::IntoIter: Send,
    {
        ctx.operator(|tx| async move {
            loop {
                match self.recv().await {
                    Event::Data(t, v) => {
                        for v in f(v).into_iter() {
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
