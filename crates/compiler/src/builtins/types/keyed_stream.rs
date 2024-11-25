use std::rc::Rc;

use linkme::distributed_slice;
use runtime::prelude::Duration;
use runtime::prelude::Format;
use runtime::prelude::Reader;
use runtime::prelude::Send;
use runtime::prelude::Sync;
use runtime::prelude::Window;

use crate::aqua;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

use super::dataflow::Dataflow;
use super::function::Function;
use super::stream::Stream;

#[derive(Debug, Clone, Eq, PartialEq, Send, Sync)]
pub struct KeyedStream(pub Rc<KeyedOperator>);

#[derive(Debug, Clone, Eq, PartialEq, Send, Sync)]
pub enum KeyedOperator {
    Source(Reader, Format, Function, Duration, Duration),
    Take(KeyedStream, i32),
    Map(KeyedStream, Function),
    Filter(KeyedStream, Function),
    Flatten(KeyedStream),
    FlatMap(KeyedStream, Function),
    Keyby(Stream, Function),
    Unkey(KeyedStream),
    Window(KeyedStream, Window, Function),
    IncrWindow(KeyedStream, Window, Function, Function, Function),
    Merge(KeyedStream, KeyedStream),
}

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type KeyedStream[K, T];",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[K, T] KeyedStream[K, T]",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: aqua! {
                    "def source[K, T](
                        r: Reader,
                        f: Format,
                        t: (T, Time) => Time,
                        slack: Duration,
                        watermark_interval: Duration
                     ): KeyedStream[K, T];"
                },
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_reader();
                    let v1 = v[1].as_format();
                    let v2 = v[2].as_function();
                    let v3 = v[3].as_duration();
                    let v4 = v[4].as_duration();
                    KeyedStream(Rc::new(KeyedOperator::Source(v0, v1, v2, v3, v4))).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def sink(s: KeyedStream[K, T], w: Writer, f: Format): Dataflow;",
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_keyed_stream();
                    let v1 = v[1].as_writer();
                    let v2 = v[2].as_format();
                    Dataflow::KeyedSink(v0, v1, v2).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def map[U](s: KeyedStream[K, T], f: T=>U): KeyedStream[K, U];",
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def filter(s: KeyedStream[K, T], f: T=>bool): KeyedStream[K, T];",
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
            // ImplDecl::Def {
            //     docs: "",
            //     aqua: indoc::indoc! {
            //         "def flatmap[B, I](s: KeyedStream[K, A], f: A=>I): KeyedStream[K, B]
            //          where Iterator[I, Item=B];"
            //     },
            //     codegen: None,
            //     eval: |_ctx, _v| {
            //         // let v0 = v[0].as_stream();
            //         // let v1 = v[1].as_function();
            //         todo!()
            //     },
            // },
            // ImplDecl::Def {
            //     docs: "",
            //     aqua: aqua! {
            //         "def flatten[K, T, I](s: KeyedStream[K, I]): KeyedStream[K, T]
            //              where IntoIterator[I, Item=T];"
            //     },
            //     codegen: None,
            //     eval: |_ctx, v| {
            //         let v0 = v[0].as_keyed_stream();
            //         KeyedStream(Rc::new(KeyedOperator::Flatten(v0))).into()
            //     },
            // },
            ImplDecl::Def {
                docs: "",
                aqua: aqua! {
                    "def window[O](
                        stream: KeyedStream[K, T],
                        win: Window,
                        agg: (K, Vec[T]) => O,
                    ): KeyedStream[K, O];"
                },
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_keyed_stream();
                    let v1 = v[1].as_window();
                    let v2 = v[2].as_function();
                    KeyedStream(Rc::new(KeyedOperator::Window(v0, v1, v2))).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: aqua! {
                    "def incrWindow[P, O](
                        stream: KeyedStream[K, T],
                        win: Window,
                        lift: T=>P,
                        combine: (P,P)=>P,
                        lower: P=>O
                    ): KeyedStream[K, O];"
                },
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_keyed_stream();
                    let v1 = v[1].as_window();
                    let v2 = v[2].as_function();
                    let v3 = v[3].as_function();
                    let v4 = v[4].as_function();
                    KeyedStream(Rc::new(KeyedOperator::IncrWindow(v0, v1, v2, v3, v4))).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def unkey[K, T](s: KeyedStream[K, T]): Stream[T];",
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_keyed_stream();
                    KeyedStream(Rc::new(KeyedOperator::Unkey(v0))).into()
                },
            },
        ],
    });
}

impl std::fmt::Display for KeyedStream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0.as_ref() {
            KeyedOperator::Source(r, e, t, interval, slack) => {
                write!(f, "source({r}, {e}, {t}, {interval}, {slack})")
            }
            KeyedOperator::Map(s, t) => write!(f, "map({s}, {t})"),
            KeyedOperator::Filter(s, t) => write!(f, "filter({s}, {t})"),
            KeyedOperator::Flatten(s) => write!(f, "flatten({s})"),
            KeyedOperator::FlatMap(s, t) => write!(f, "flatmap({s}, {t})"),
            KeyedOperator::Keyby(s, t) => write!(f, "keyby({s}, {t})"),
            KeyedOperator::Window(s, d, a) => write!(f, "window({s}, {d}, {a})"),
            KeyedOperator::Merge(s1, s2) => write!(f, "merge({s1}, {s2})"),
            KeyedOperator::IncrWindow(s, a, f0, f1, f2) => {
                write!(f, "incr_window({s}, {a}, {f0}, {f1}, {f2})")
            }
            KeyedOperator::Take(s, n) => write!(f, "take({s}, {n})"),
            KeyedOperator::Unkey(s) => write!(f, "unkey({s})"),
        }
    }
}
