use std::collections::BTreeMap;

use crate::builtins::duration::Duration;
use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;

use super::align;
use super::WindowRange;

impl<T: Data> Stream<T> {
    /// This window maintains a BTreeMap that maps window starting times to slices of data, where
    /// each slice is a vector of (Time, Data) pairs. When a watermark is received, the window
    /// iterates over the slices before the watermark and sends the result of the compute function
    pub fn time_sliding_aligned_holistic_window<O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        step: Duration,
        compute: impl Fn(Window<T>, WindowRange) -> O + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
    {
        ctx.operator(|tx| async move {
            let mut s: WindowState<T> = WindowState::new();
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        let t0 = align(time, step);
                        s.get_mut(t0).0.push((time, data));
                    }
                    Event::Watermark(time) => {
                        while let Some(entry) = s.0.first_entry() {
                            let t0 = *entry.key();
                            let wr = WindowRange::new(t0, t0 + duration);
                            if wr.t1 > time {
                                break;
                            }
                            let win = Window::new(&s.0, wr.t1);
                            let data = compute(win, wr);
                            s.0.pop_first();
                            tx.send(Event::Data(wr.t1, data.deep_clone())).await;
                        }
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

#[derive(Debug, Default)]
struct WindowState<T>(BTreeMap<Time, Slice<T>>);

impl<T> WindowState<T> {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
    fn get_mut(&mut self, time: Time) -> &mut Slice<T> {
        self.0.entry(time).or_default()
    }
}

#[derive(Debug)]
pub struct Window<'a, T> {
    buffer: &'a BTreeMap<Time, Slice<T>>,
    t1: Time,
}

impl<'a, T> Window<'a, T> {
    fn new(buffer: &'a BTreeMap<Time, Slice<T>>, t1: Time) -> Self {
        Self { buffer, t1 }
    }
    pub fn iter(&self) -> WindowIter<T> {
        WindowIter::new(self.buffer.range(..self.t1))
    }
}

impl<'a, T> IntoIterator for &'a Window<'a, T> {
    type Item = &'a T;
    type IntoIter = WindowIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
struct Slice<T>(Vec<(Time, T)>);

impl<T> Default for Slice<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub struct WindowIter<'a, T> {
    iter: std::collections::btree_map::Range<'a, Time, Slice<T>>,
    current: Option<std::slice::Iter<'a, (Time, T)>>,
}

impl<'a, T> WindowIter<'a, T> {
    fn new(iter: std::collections::btree_map::Range<'a, Time, Slice<T>>) -> Self {
        Self {
            iter,
            current: None,
        }
    }
}

impl<'a, T> Iterator for WindowIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = &mut self.current {
            if let Some(item) = current.next().map(|(_, ref item)| item) {
                return Some(item);
            }
        }
        if let Some((_, slice)) = self.iter.next() {
            let mut current = slice.0.iter();
            let next = current.next().map(|(_, ref item)| item);
            self.current = Some(current);
            return next;
        }
        None
    }
}
