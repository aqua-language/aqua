use crate::builtins::stream::Event;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Stream;

impl<T: Data> Stream<T> {
    pub fn merge(mut self, ctx: &mut Context, mut other: Self) -> Self {
        ctx.operator(|tx| async move {
            let mut l_done = false;
            let mut r_done = false;
            let mut l_watermark = Time::zero();
            let mut r_watermark = Time::zero();
            loop {
                tokio::select! {
                    event = self.recv(), if !l_done => match event {
                        Event::Data(t, v) => tx.send(Event::Data(t, v)).await?,
                        Event::Watermark(t) => {
                            if t < r_watermark {
                                tx.send(Event::Watermark(t)).await?;
                            } else if l_watermark < r_watermark && r_watermark < t {
                                tx.send(Event::Watermark(r_watermark)).await?;
                            }
                            l_watermark = t;
                        },
                        Event::Sentinel => {
                            l_done = true;
                            if r_done {
                                tx.send(Event::Sentinel).await?;
                                break;
                            }
                        }
                        Event::Snapshot(i) => tx.send(Event::Snapshot(i)).await?
                    },
                    event = other.recv(), if !r_done => match event {
                        Event::Data(t, v) => tx.send(Event::Data(t, v)).await?,
                        Event::Watermark(t) => {
                            if t < l_watermark {
                                tx.send(Event::Watermark(t)).await?;
                            } else if r_watermark < l_watermark && l_watermark < t {
                                tx.send(Event::Watermark(l_watermark)).await?;
                            }
                            r_watermark = t;
                        },
                        Event::Sentinel => {
                            r_done = true;
                            if l_done {
                                tx.send(Event::Sentinel).await?;
                                break;
                            }
                        }
                        Event::Snapshot(i) => tx.send(Event::Snapshot(i)).await?
                    },
                };
            }
            Ok(())
        })
    }
}
