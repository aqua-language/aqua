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
    /// each slice is a collection of (Time, Data) pairs. When a watermark is received, the window
    /// iterates over the slices before the watermark and sends the result of the compute function
    pub fn time_sliding_aligned_holistic_vec_window<O>(
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
            let mut buffer: Vec<(Time, Slice<T>)> = Vec::new();
            // Slices before this time are sorted
            let mut t_sorted: Time = Time::zero();
            loop {
                match self.recv().await {
                    Event::Data(time, data) => {
                        let t0 = align(time, step);
                        match buffer.binary_search_by_key(&t0, |(t, _)| *t) {
                            Ok(i) => {
                                (buffer[i].1).0.push((time, data));
                            }
                            Err(i) => {
                                buffer.insert(i, (t0, Slice::new()));
                                (buffer[i].1).0.push((time, data));
                            }
                        }
                    }
                    Event::Watermark(time) => {
                        let t_safe = align(time, step);
                        for (t, slice) in &mut buffer {
                            if *t < t_sorted {
                                continue;
                            } else if *t < t_safe {
                                slice.0.sort_by_key(|(t, _)| *t);
                            } else {
                                break;
                            }
                        }
                        t_sorted = t_safe;
                        for (t0, _) in &buffer {
                            let wr = WindowRange::new(*t0, *t0 + duration);
                            if wr.t1 > time {
                                break;
                            }
                            let data = compute(Window::new(&buffer), wr);
                            tx.send(Event::Data(wr.t1, data.deep_clone())).await;
                        }
                        buffer.retain(|(t, _)| *t >= t_safe);
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
    buffer: &'a [(Time, Slice<T>)],
}

impl<'a, T> Window<'a, T> {
    fn new(buffer: &'a [(Time, Slice<T>)]) -> Self {
        Self { buffer }
    }
    pub fn iter(&self) -> WindowIter<T> {
        WindowIter::new(self.buffer.iter())
    }
}

impl<'a, T> IntoIterator for &'a Window<'a, T> {
    type Item = &'a T;
    type IntoIter = WindowIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

struct Slice<T>(Vec<(Time, T)>);

impl<T> Slice<T> {
    fn new() -> Self {
        Self(Vec::new())
    }
}

pub struct WindowIter<'a, T> {
    iter: std::slice::Iter<'a, (Time, Slice<T>)>,
    current: Option<std::slice::Iter<'a, (Time, T)>>,
}

impl<'a, T> WindowIter<'a, T> {
    fn new(iter: std::slice::Iter<'a, (Time, Slice<T>)>) -> Self {
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
