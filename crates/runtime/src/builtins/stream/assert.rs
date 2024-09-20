use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn assert<I>(mut self, ctx: &mut Context, iter: I)
    where
        I: IntoIterator<Item = Event<T>> + Send + 'static,
        <I as IntoIterator>::IntoIter: Send + 'static,
        T: PartialEq,
    {
        ctx.sink(|| async move {
            let mut iter = iter.into_iter();
            loop {
                let next = self.recv().await;
                assert_eq!(iter.next().as_ref(), Some(&next));
                if let Event::Sentinel = next {
                    break;
                }
            }
            Ok(())
        });
    }
}
