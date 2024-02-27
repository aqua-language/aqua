use crate::builtins::duration::Duration;
use crate::builtins::stream::window::align;
use crate::builtins::stream::Event;
use crate::builtins::stream::Stream;
use crate::builtins::time::Time;
use crate::runner::context::Context;
use crate::traits::Data;
use crate::traits::Key;
use crate::BTreeMap;
use crate::HashMap;

impl<T: Data> Stream<T> {
    pub fn tumbling_window_join_distinct<R, K, O>(
        mut self,
        ctx: &mut Context,
        mut other: Stream<R>,
        left_key: impl Fn(&T) -> K + Send + 'static,
        right_key: impl Fn(&R) -> K + Send + 'static,
        duration: Duration,
        joiner: impl Fn(&T, &R) -> O + Send + 'static,
    ) -> Stream<O>
    where
        R: Data,
        K: Data + Key,
        O: Data,
    {
        ctx.operator(|tx| async move {
            let mut s: JoinState<K,T,R> = JoinState::default();
            let mut l_watermark = Time::zero();
            let mut r_watermark = Time::zero();
            let mut done_l = false;
            let mut done_r = false;
            loop {
                tokio::select! {
                    event = self.recv(), if !done_l => match event {
                        Event::Data(time, data) => {
                            let key = left_key(&data);
                            let t0 = align(time, duration);
                            let skey = s.get(t0, key);
                            match skey {
                                State::Left(_) => unreachable!(),
                                State::Right(_) => {
                                    let State::Right(r) = std::mem::take(skey) else { unreachable!() };
                                    tx.send(Event::Data(t0, joiner(&data, &r).deep_clone())).await;
                                }
                                State::Empty => {
                                    *skey = State::Left(data);
                                }
                            }
                        }
                        Event::Watermark(t) => {
                            if t < r_watermark {
                                s.gc(t, duration);
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
                        Event::Snapshot(_) => unimplemented!(),
                    },
                    event = other.recv(), if !done_r => match event {
                        Event::Data(time, data) => {
                            let key = right_key(&data);
                            let t0 = align(time, duration);
                            let skey = s.get(t0, key);
                            match skey {
                                State::Left(_) => {
                                    let State::Left(l) = std::mem::take(skey) else { unreachable!() };
                                    tx.send(Event::Data(t0, joiner(&l, &data).deep_clone())).await;
                                }
                                State::Right(_) => unreachable!(),
                                State::Empty => {
                                    *skey = State::Right(data);
                                }
                            }
                        }
                        Event::Watermark(t) => {
                            if t < l_watermark {
                                s.gc(t, duration);
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

struct JoinState<K, L, R>(BTreeMap<Time, KeyState<K, L, R>>);

struct KeyState<K, L, R>(HashMap<K, State<L, R>>);

impl<K, L, R> Default for JoinState<K, L, R> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<K, L, R> Default for KeyState<K, L, R> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

#[derive(Default)]
enum State<L, R> {
    Left(L),
    Right(R),
    #[default]
    Empty,
}

impl<K, L, R> JoinState<K, L, R>
where
    K: Key,
{
    fn get(&mut self, time: Time, key: K) -> &mut State<L, R> {
        self.0.entry(time).or_default().0.entry(key).or_default()
    }

    fn gc(&mut self, time: Time, duration: Duration) {
        while let Some(entry) = self.0.first_entry() {
            let t1 = *entry.key() + duration;
            if t1 < time {
                entry.remove();
            } else {
                break;
            }
        }
    }
}
