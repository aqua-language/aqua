use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn take(mut self, ctx: &mut Context, mut i: i32) -> Stream<T> {
        ctx.operator(move |tx| async move {
            loop {
                if i == 0 {
                    tx.send(Event::Sentinel).await?;
                    break;
                }
                match self.recv().await {
                    Event::Data(t, v) => {
                        tx.send(Event::Data(t, v)).await?;
                        i -= 1;
                    }
                    Event::Watermark(t) => tx.send(Event::Watermark(t)).await?,

                    Event::Snapshot(i) => tx.send(Event::Snapshot(i)).await?,
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
