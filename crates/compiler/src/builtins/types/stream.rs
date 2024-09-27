use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;

use std::rc::Rc;

use super::dataflow::Dataflow;
use super::function::Fun;
use runtime::builtins::duration::Duration;
use runtime::prelude::Assigner;
use runtime::prelude::Encoding;
use runtime::prelude::Reader;
use runtime::prelude::Send;
use runtime::prelude::Sync;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Stream[T];",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Stream[T]",
        decls: &[
            ImplDecl::Def {
                aqua: indoc::indoc! {
                    "def source(
                         reader: Reader,
                         encoding: Encoding,
                         extractor: (T, Time) => Time,
                         slack: Duration,
                         watermark_interval: Duration
                     ): Stream[T];"
                },
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_reader();
                    let v1 = v[1].as_encoding();
                    let v2 = v[2].as_function();
                    let v3 = v[3].as_duration();
                    let v4 = v[4].as_duration();
                    Stream(Rc::new(Operator::Source(v0, v1, v2, v3, v4))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def sink(s: Stream[T], w: Writer, e: Encoding): Dataflow;",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_writer();
                    let v2 = v[2].as_encoding();
                    Dataflow::Sink(v0, v1, v2).into()
                },
            },
            ImplDecl::Def {
                aqua: "def take(s: Stream[T], n: i32): Stream[T];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_i32();
                    Stream(Rc::new(Operator::Take(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def map[U](s: Stream[T], f: T => U): Stream[U];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream(Rc::new(Operator::Map(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def filter(s: Stream[T], f: T => bool): Stream[T];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream(Rc::new(Operator::Filter(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def flatmap[U](s: Stream[T], f: T => Vec[U]): Stream[U];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream(Rc::new(Operator::FlatMap(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def flatten(s: Stream[Vec[T]]): Stream[T];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    Stream(Rc::new(Operator::Flatten(v0))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def window[U](s: Stream[T], a: Assigner, f: Vec[T] => U): Stream[U];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_assigner();
                    let v2 = v[2].as_function();
                    Stream(Rc::new(Operator::Window(v0, v1, v2))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def incrWindow[P,U](s: Stream[T], a: Assigner, f1: T=>P, f2: (P,P)=>P, f2: P=>U): Stream[U];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_assigner();
                    let v2 = v[2].as_function();
                    let v3 = v[3].as_function();
                    let v4 = v[4].as_function();
                    Stream(Rc::new(Operator::IncrWindow(v0, v1, v2, v3, v4))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def keyby[K](s: Stream[T], f: T => K): KeyedStream[K, T];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream(Rc::new(Operator::Keyby(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def merge(s1: Stream[T], s2: Stream[T]): Stream[T];",
                codegen: None,
                fun: |_, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_stream();
                    Stream(Rc::new(Operator::Merge(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def collect(s: Stream[T]): Vec[T];",
                codegen: None,
                fun: |ctx, v| {
                    let v0 = v[0].as_stream();
                    v0.collect(ctx).into()
                },
            },
        ],
    });
}

#[derive(Debug, Clone, Eq, PartialEq, Send, Sync)]
pub struct Stream(pub Rc<Operator>);

#[derive(Debug, Clone, Eq, PartialEq, Send, Sync)]
pub enum Operator {
    Source(Reader, Encoding, Fun, Duration, Duration),
    Take(Stream, i32),
    Map(Stream, Fun),
    Filter(Stream, Fun),
    Flatten(Stream),
    FlatMap(Stream, Fun),
    Keyby(Stream, Fun),
    Window(Stream, Assigner, Fun),
    IncrWindow(Stream, Assigner, Fun, Fun, Fun),
    Merge(Stream, Stream),
}

impl Operator {
    pub fn to_stream(self) -> Stream {
        Stream(Rc::new(self.clone()))
    }
}

impl Stream {
    /// Used to determine if two streams are equivalent.
    pub fn id(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }

    pub fn kind(&self) -> &Operator {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0.as_ref() {
            Operator::Source(r, e, t, interval, slack) => {
                write!(f, "source({r}, {e}, {t}, {interval}, {slack})")
            }
            Operator::Map(s, t) => write!(f, "map({s}, {t})"),
            Operator::Filter(s, t) => write!(f, "filter({s}, {t})"),
            Operator::Flatten(s) => write!(f, "flatten({s})"),
            Operator::FlatMap(s, t) => write!(f, "flatmap({s}, {t})"),
            Operator::Keyby(s, t) => write!(f, "keyby({s}, {t})"),
            Operator::Window(s, d, a) => write!(f, "window({s}, {d}, {a})"),
            Operator::Merge(s1, s2) => write!(f, "merge({s1}, {s2})"),
            Operator::IncrWindow(s, a, f0, f1, f2) => {
                write!(f, "incr_window({s}, {a}, {f0}, {f1}, {f2})")
            }
            Operator::Take(s, n) => write!(f, "take({s}, {n})"),
        }
    }
}

use crate::builtins::DECLS;
use linkme::distributed_slice;

impl Stream {
    fn collect(&self, ctx0: &crate::interpret::Context) -> runtime::builtins::vec::Vec<Value> {
        let this = self.clone();
        let ctx0 = ctx0.clone();
        let (tx, mut rx) = runtime::prelude::tokio::sync::mpsc::channel::<Vec<Value>>(1);
        runtime::runner::current_thread::CurrentThreadRunner::run(|ctx1| {
            this.spawn(&ctx0, ctx1).collect_vec(ctx1, tx);
        });
        let v: Vec<Value> = rx.blocking_recv().unwrap();
        v.into()
    }

    fn spawn(
        &self,
        ctx0: &crate::interpret::Context,
        ctx1: &mut runtime::runner::context::Context,
    ) -> runtime::prelude::Stream<Value> {
        match self.kind().clone() {
            Operator::Source(r, e, f, d0, d1) => {
                let t = f.params.values().next().unwrap().clone();
                let seed = crate::builtins::traits::serde::Seed::new(t, ctx0.decls.clone());
                let mut ctx0 = ctx0.clone();
                runtime::prelude::Stream::dyn_source(
                    ctx1,
                    r.clone(),
                    e.clone(),
                    move |v, ts| {
                        let a0: Value = v;
                        let a1: Value = ts.into();
                        f.call(&mut ctx0, &[a0, a1]).as_time()
                    },
                    d0,
                    d1,
                    seed,
                )
            }
            Operator::Take(s, n) => {
                let s = s.spawn(ctx0, ctx1);
                runtime::prelude::Stream::take(s, ctx1, n)
            }
            Operator::Map(s, f) => {
                let s = s.spawn(ctx0, ctx1);
                let ctx0 = ctx0.clone();
                runtime::prelude::Stream::map(s, ctx1, move |v| {
                    let a0: Value = v;
                    let mut ctx0 = ctx0.clone();
                    f.call(&mut ctx0, &[a0])
                })
            }
            Operator::Filter(s, f) => {
                let s = s.spawn(ctx0, ctx1);
                let ctx0 = ctx0.clone();
                runtime::prelude::Stream::filter(s, ctx1, move |v| {
                    let a0: Value = v.clone();
                    let mut ctx0 = ctx0.clone();
                    f.call(&mut ctx0, &[a0]).as_bool()
                })
            }
            Operator::Flatten(_) => todo!(),
            Operator::FlatMap(_, _) => todo!(),
            Operator::Keyby(_, _) => todo!(),
            Operator::Window(_, _, _) => todo!(),
            Operator::IncrWindow(_, _, _, _, _) => todo!(),
            Operator::Merge(_, _) => todo!(),
        }
    }
}
