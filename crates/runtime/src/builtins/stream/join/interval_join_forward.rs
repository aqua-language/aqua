use std::collections::VecDeque;

use crate::builtins::duration::Duration;
use crate::builtins::stream::Collector;
use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;
use crate::HashMap;

impl<T: Data> Stream<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn interval_join_forward<R, K, O>(
        mut self,
        ctx: &mut Context,
        mut other: Stream<R>,
        left_key: impl Fn(&T) -> K + Send + 'static,
        right_key: impl Fn(&R) -> K + Send + 'static,
        upper_bound: Duration,
        joiner: impl Fn(&T, &R) -> O + Send + Sync + 'static,
    ) -> Stream<O>
    where
        R: Data,
        K: Data + Key,
        O: Data,
    {
        ctx.operator(|tx| async move {
            let mut s: State<K, T, R> = State::new(upper_bound);
            let mut l_watermark = Time::zero();
            let mut r_watermark = Time::zero();
            let mut done_l = false;
            let mut done_r = false;
            loop {
                tokio::select! {
                    event = self.recv(), if !done_l => match event {
                        Event::Data(time, data) => {
                            let key = left_key(&data);
                            s.incremental_join_left(key, time, data, &joiner, &tx).await;
                        }
                        Event::Watermark(t) => {
                            s.add_watermark_left(t);
                            if t < r_watermark {
                                tx.send(Event::Watermark(t)).await;
                            } else if l_watermark < r_watermark && r_watermark < t {
                                tx.send(Event::Watermark(r_watermark)).await
                            }
                            l_watermark = t;
                        }
                        Event::Sentinel => {
                            if done_r {
                                tx.send(Event::Sentinel).await;
                                break;
                            }
                            done_l = true;
                        }
                        Event::Snapshot(_) => unimplemented!()
                    },
                    event = other.recv(), if !done_r => match event {
                        Event::Data(time, data) => {
                            let key = right_key(&data);
                            s.incremental_join_right(key, time, data, &joiner, &tx).await;
                        }
                        Event::Watermark(t) => {
                            s.add_watermark_right(t);
                            if t < l_watermark {
                                tx.send(Event::Watermark(t)).await;
                            } else if r_watermark < l_watermark && l_watermark < t {
                                tx.send(Event::Watermark(l_watermark)).await
                            }
                            r_watermark = t;
                        }
                        Event::Sentinel => {
                            if done_l {
                                tx.send(Event::Sentinel).await;
                                break;
                            }
                            done_r = true;
                        }
                        Event::Snapshot(_) => unimplemented!(),
                    },
                };
            }
        })
    }
}

struct State<K, L, R> {
    lslices: SliceSeq<K, L>,
    rslices: SliceSeq<K, R>,
    upper_bound: Duration,
}

struct SliceSeq<K, T>(VecDeque<Slice<K, T>>);

struct Slice<K, T> {
    latest: Time,
    earliest: Time,
    data: HashMap<K, Vec<(Time, T)>>,
}

impl<K, T> Slice<K, T>
where
    K: Key,
{
    fn new(time: Time) -> Self {
        Self {
            latest: time,
            earliest: time,
            data: HashMap::default(),
        }
    }

    fn insert(&mut self, key: K, time: Time, data: T) {
        self.data.entry(key).or_default().push((time, data));
        self.latest = self.latest.max(time);
        self.earliest = self.earliest.min(time);
    }
}

impl<K, T> SliceSeq<K, T>
where
    K: Data + Key,
    T: Data,
{
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn push_watermark(&mut self, time: Time) {
        self.0.push_back(Slice::new(time));
    }

    fn gc(&mut self, time: Time) {
        while let Some(entry) = self.0.front() {
            let t1 = entry.latest;
            if t1 < time {
                self.0.pop_front();
            } else {
                break;
            }
        }
    }

    fn push_data_or_create(&mut self, time: Time, key: K, data: T) {
        if self.0.is_empty() {
            self.0.push_back(Slice::new(time));
        }
        self.0.back_mut().unwrap().insert(key, time, data);
    }
}

impl<K, L, R> State<K, L, R>
where
    K: Data + Key,
    L: Data,
    R: Data,
{
    fn new(upper_bound: Duration) -> Self {
        Self {
            lslices: SliceSeq::new(),
            rslices: SliceSeq::new(),
            upper_bound,
        }
    }

    fn add_watermark_left(&mut self, time: Time) {
        self.lslices.push_watermark(time);
        self.rslices.gc(time);
    }

    fn add_watermark_right(&mut self, time: Time) {
        self.rslices.push_watermark(time);
        self.lslices.gc(time - self.upper_bound);
    }

    async fn incremental_join_left<O: Data>(
        &mut self,
        key: K,
        time: Time,
        data: L,
        joiner: impl Fn(&L, &R) -> O,
        tx: &Collector<O>,
    ) {
        self.lslices
            .push_data_or_create(time, key.clone(), data.clone());
        let latest_possible = time + self.upper_bound;
        for slice in self.rslices.0.iter() {
            if slice.latest < time {
                // If the slice is before the interval, we can skip it
                continue;
            }
            if latest_possible < slice.earliest {
                // If the slice is after the interval, we can stop
                break;
            }
            let Some(vec) = slice.data.get(&key) else {
                continue;
            };
            if time <= slice.earliest && slice.latest <= latest_possible {
                // If the slice is completely contained in the interval, we can just join everything
                for (other_time, other_data) in vec {
                    let time = time.max(*other_time);
                    let data = joiner(&data, other_data);
                    tx.send(Event::Data(time, data.deep_clone())).await;
                }
            } else {
                for (other_time, other_data) in vec {
                    if time <= *other_time && *other_time <= latest_possible {
                        // If the data is in the interval, we can join it
                        let time = time.max(*other_time);
                        let data = joiner(&data, other_data);
                        tx.send(Event::Data(time, data.deep_clone())).await;
                    }
                }
            }
        }
    }

    async fn incremental_join_right<O: Data>(
        &mut self,
        key: K,
        time: Time,
        data: R,
        joiner: impl Fn(&L, &R) -> O,
        tx: &Collector<O>,
    ) {
        self.rslices
            .push_data_or_create(time, key.clone(), data.clone());
        let earliest_possible = time - self.upper_bound;
        for slice in self.lslices.0.iter() {
            if slice.latest < earliest_possible {
                // If the slice is before the interval, we can skip it
                continue;
            }
            if time < slice.earliest {
                // If the slice is after the interval, we can stop
                break;
            }
            let Some(vec) = slice.data.get(&key) else {
                continue;
            };
            if earliest_possible <= slice.earliest && slice.latest <= time {
                // If the slice is completely contained in the interval, we can just join everything
                for (other_time, other_data) in vec {
                    let time = time.max(*other_time);
                    let data = joiner(other_data, &data);
                    tx.send(Event::Data(time, data.deep_clone())).await;
                }
            } else {
                for (other_time, other_data) in vec {
                    if earliest_possible <= *other_time && *other_time <= time {
                        // If the data is in the interval, we can join it
                        let time = time.max(*other_time);
                        let data = joiner(other_data, &data);
                        tx.send(Event::Data(time, data.deep_clone())).await;
                    }
                }
            }
        }
    }
}
