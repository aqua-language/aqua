use crate::runner::context::Context;
use crate::traits::Data;

use super::Stream;

impl<T: Data> Stream<T> {
    pub fn collect_vec(mut self, ctx: &mut Context, out: tokio::sync::mpsc::Sender<Vec<T>>) {
        ctx.sink(|| async move {
            let mut vec = Vec::new();
            loop {
                match self.recv().await {
                    super::Event::Data(_, v) => vec.push(v),
                    super::Event::Watermark(_) => {}
                    super::Event::Snapshot(_) => {}
                    super::Event::Sentinel => {
                        out.send(vec).await.unwrap();
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}
