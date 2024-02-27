use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::runner::context::Context;
use crate::traits::Data;

impl<T: Data> Stream<T> {
    pub fn count_tumbling_holistic_window<O>(
        mut self,
        ctx: &mut Context,
        size: usize,
        compute: impl Fn(Window<T>) -> O + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
    {
        ctx.operator(|tx| async move {
            let mut agg: Vec<T> = Vec::with_capacity(size);
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        agg.push(data);
                        if agg.len() == size {
                            let result = compute(Window::new(&agg));
                            tx.send(Event::Data(time, result.deep_clone())).await;
                            agg.clear();
                        }
                    }
                    Event::Watermark(time) => {
                        tx.send(Event::Watermark(time)).await;
                    }
                    Event::Snapshot(i) => {
                        tx.send(Event::Snapshot(i)).await;
                    }
                    Event::Sentinel => {
                        tx.send(Event::Sentinel).await;
                        break;
                    }
                }
            }
        })
    }
}

pub struct Window<'a, T> {
    data: &'a Vec<T>,
}

impl<'a, T> Window<'a, T> {
    fn new(data: &'a Vec<T>) -> Self {
        Self { data }
    }
    pub fn iter(&self) -> std::slice::Iter<'a, T> {
        self.data.iter()
    }
}
