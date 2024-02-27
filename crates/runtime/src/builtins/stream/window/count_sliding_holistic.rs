use std::collections::VecDeque;

use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::runner::context::Context;
use crate::traits::Data;

impl<T: Data> Stream<T> {
    pub fn count_sliding_holistic_window<O>(
        mut self,
        ctx: &mut Context,
        size: usize,
        step: usize,
        compute: impl Fn(Window<T>) -> O + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
    {
        ctx.operator(|tx| async move {
            let mut s: VecDeque<T> = VecDeque::with_capacity(size);
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        s.push_back(data);
                        if s.len() == size {
                            let result = compute(Window::new(&s));
                            for _ in 0..step {
                                s.pop_front();
                            }
                            tx.send(Event::Data(time, result.deep_clone())).await;
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
    data: &'a VecDeque<T>,
}

impl<'a, T> Window<'a, T> {
    fn new(data: &'a VecDeque<T>) -> Self {
        Self { data }
    }
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'a, T> {
        self.data.iter()
    }
}
