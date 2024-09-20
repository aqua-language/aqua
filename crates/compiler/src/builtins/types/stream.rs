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
                    "def source[T](
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
                    Stream(Rc::new(StreamKind::Source(v0, v1, v2, v3, v4))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def sink[T](s: Stream[T], w: Writer, e: Encoding): Dataflow;",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_writer();
                    let v2 = v[2].as_encoding();
                    Dataflow::Sink(v0, v1, v2).into()
                },
            },
            ImplDecl::Def {
                aqua: "def take[T](s: Stream[T], n: i32): Stream[T];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_i32();
                    Stream(Rc::new(StreamKind::Take(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def map[A, B](s: Stream[A], f: A => B): Stream[B];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream(Rc::new(StreamKind::Map(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def filter[T](s: Stream[T], f: T => bool): Stream[T];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream(Rc::new(StreamKind::Filter(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def flatmap[A, B](s: Stream[A], f: A => Vec[B]): Stream[B];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream(Rc::new(StreamKind::FlatMap(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua: "def flatten[T](s: Stream[Vec[T]]): Stream[T];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    Stream(Rc::new(StreamKind::Flatten(v0))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def window[I, O](s: Stream[I], a: Assigner, f: Vec[I] => O): Stream[O];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_assigner();
                    let v2 = v[2].as_function();
                    Stream(Rc::new(StreamKind::Window(v0, v1, v2))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def incrWindow[I,P,O](s: Stream[I], a: Assigner, f1: I=>P, f2: (P,P)=>P, f2: P=>O): Stream[O];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_assigner();
                    let v2 = v[2].as_function();
                    let v3 = v[3].as_function();
                    let v4 = v[4].as_function();
                    Stream(Rc::new(StreamKind::IncrWindow(v0, v1, v2, v3, v4))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def keyby[K, T](s: Stream[T], f: T => K): Stream[(K, T)];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream(Rc::new(StreamKind::Keyby(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def merge[T](s1: Stream[T], s2: Stream[T]): Stream[T];",
                codegen: None,
                fun: |_, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_stream();
                    Stream(Rc::new(StreamKind::Merge(v0, v1))).into()
                },
            },
            ImplDecl::Def {
                aqua:"def collect[T](s: Stream[T]): Vec[T];",
                codegen: None,
                fun: |ctx, v| {
                    let v0 = v[0].as_stream();
                    v0.collect(ctx).into()
                },
            },
            ImplDecl::Def {
                aqua:"def unkey[K, T](s: Stream[(K, T)]): Stream[T];",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    Stream(Rc::new(StreamKind::Unkey(v0))).into()
                },
            },
        ],
    });
}

#[derive(Debug, Clone, Eq, PartialEq, Send, Sync)]
pub struct Stream(pub Rc<StreamKind>);

#[derive(Debug, Clone, Eq, PartialEq, Send, Sync)]
pub enum StreamKind {
    Source(Reader, Encoding, Fun, Duration, Duration),
    Take(Stream, i32),
    Map(Stream, Fun),
    Filter(Stream, Fun),
    Flatten(Stream),
    FlatMap(Stream, Fun),
    Keyby(Stream, Fun),
    Unkey(Stream),
    Window(Stream, Assigner, Fun),
    IncrWindow(Stream, Assigner, Fun, Fun, Fun),
    Merge(Stream, Stream),
}

impl StreamKind {
    pub fn to_stream(self) -> Stream {
        Stream(Rc::new(self.clone()))
    }
}

impl Stream {
    /// Used to determine if two streams are equivalent.
    pub fn id(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }

    pub fn kind(&self) -> &StreamKind {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0.as_ref() {
            StreamKind::Source(r, e, t, interval, slack) => {
                write!(f, "source({r}, {e}, {t}, {interval}, {slack})")
            }
            StreamKind::Map(s, t) => write!(f, "map({s}, {t})"),
            StreamKind::Filter(s, t) => write!(f, "filter({s}, {t})"),
            StreamKind::Flatten(s) => write!(f, "flatten({s})"),
            StreamKind::FlatMap(s, t) => write!(f, "flatmap({s}, {t})"),
            StreamKind::Keyby(s, t) => write!(f, "keyby({s}, {t})"),
            StreamKind::Unkey(s) => write!(f, "unkey({s})"),
            StreamKind::Window(s, d, a) => write!(f, "window({s}, {d}, {a})"),
            StreamKind::Merge(s1, s2) => write!(f, "merge({s1}, {s2})"),
            StreamKind::IncrWindow(s, a, f0, f1, f2) => {
                write!(f, "incr_window({s}, {a}, {f0}, {f1}, {f2})")
            }
            StreamKind::Take(s, n) => write!(f, "take({s}, {n})"),
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
            StreamKind::Source(r, e, f, d0, d1) => {
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
            StreamKind::Take(s, n) => {
                let s = s.spawn(ctx0, ctx1);
                runtime::prelude::Stream::take(s, ctx1, n)
            }
            StreamKind::Map(s, f) => {
                let s = s.spawn(ctx0, ctx1);
                let ctx0 = ctx0.clone();
                runtime::prelude::Stream::map(s, ctx1, move |v| {
                    let a0: Value = v;
                    let mut ctx0 = ctx0.clone();
                    f.call(&mut ctx0, &[a0])
                })
            }
            StreamKind::Filter(s, f) => {
                let s = s.spawn(ctx0, ctx1);
                let ctx0 = ctx0.clone();
                runtime::prelude::Stream::filter(s, ctx1, move |v| {
                    let a0: Value = v.clone();
                    let mut ctx0 = ctx0.clone();
                    f.call(&mut ctx0, &[a0]).as_bool()
                })
            }
            StreamKind::Flatten(_) => todo!(),
            StreamKind::FlatMap(_, _) => todo!(),
            StreamKind::Keyby(_, _) => todo!(),
            StreamKind::Unkey(_) => todo!(),
            StreamKind::Window(_, _, _) => todo!(),
            StreamKind::IncrWindow(_, _, _, _, _) => todo!(),
            StreamKind::Merge(_, _) => todo!(),
        }
    }
}
