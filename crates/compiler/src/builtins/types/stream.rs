use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

use std::rc::Rc;

use runtime::prelude::Aggregator;
use runtime::prelude::Assigner;
use runtime::prelude::Encoding;
use runtime::prelude::Reader;
use runtime::prelude::Writer;

use super::dataflow::Dataflow;
use super::function::Fun;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Stream {
    Source(Reader, Encoding, Fun),
    Map(Rc<Stream>, Fun),
    Filter(Rc<Stream>, Fun),
    Flatten(Rc<Stream>),
    FlatMap(Rc<Stream>, Fun),
    Keyby(Rc<Stream>, Fun),
    Unkey(Rc<Stream>),
    Window(Rc<Stream>, Assigner, Aggregator<Fun, Fun, Fun, Fun>),
    Merge(Rc<Stream>, Rc<Stream>),
    Sink(Rc<Stream>, Writer, Encoding),
}

impl std::fmt::Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Stream::Source(r, e, t) => write!(f, "source({r}, {e}, {t})"),
            Stream::Map(s, t) => write!(f, "map({s}, {t})"),
            Stream::Filter(s, t) => write!(f, "filter({s}, {t})"),
            Stream::Flatten(s) => write!(f, "flatten({s})"),
            Stream::FlatMap(s, t) => write!(f, "flatmap({s}, {t})"),
            Stream::Keyby(s, t) => write!(f, "keyby({s}, {t})"),
            Stream::Unkey(s) => write!(f, "unkey({s})"),
            Stream::Window(s, d, a) => write!(f, "window({s}, {d}, {a})"),
            Stream::Merge(s1, s2) => write!(f, "merge({s1}, {s2})"),
            Stream::Sink(s, w, e) => write!(f, "sink({s}, {w}, {e})"),
        }
    }
}

impl Compiler {
    pub(super) fn declare_stream(&mut self) {
        self.declare_type("type Stream[T];", BuiltinType { rust: "Stream" });
        self.declare_def(
            "def source[T](r: Reader, e: Encoding, t: fun(T):Time): Stream[T];",
            BuiltinDef {
                rust: "Stream::source",
                fun: |_ctx, v| {
                    let v0 = v[0].as_reader();
                    let v1 = v[1].as_encoding();
                    let v2 = v[2].as_function();
                    Stream::Source(v0, v1, v2).into()
                },
            },
        );

        self.declare_def(
            "def sink[T](s: Stream[T], w: Writer, e: Encoding): Dataflow;",
            BuiltinDef {
                rust: "Stream::sink",
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_writer();
                    let v2 = v[2].as_encoding();
                    Dataflow::Sink(v0, v1, v2).into()
                },
            },
        );

        self.declare_def(
            "def map[A, B](s: Stream[A], f: fun(A):B): Stream[B];",
            BuiltinDef {
                rust: "Stream::map",
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream::Map(Rc::new(v0), v1).into()
                },
            },
        );

        self.declare_def(
            "def filter[T](s: Stream[T], f: fun(T):bool): Stream[T];",
            BuiltinDef {
                rust: "Stream::filter",
                fun: |_ctx, v| {
                    let v0 = v[0].as_stream();
                    let v1 = v[1].as_function();
                    Stream::Filter(Rc::new(v0), v1).into()
                },
            },
        );

        // self.declare_def(
        //     "def flatmap[A, B](s: Stream[A], f: fun(A):Vec[B]): Stream[B];",
        //     BuiltinDef {
        //         rust: "Stream::flatmap",
        //         fun: |_ctx, _v| {
        //             todo!()
        //             // let v0 = v[0].as_stream();
        //             // let v1 = v[1].as_function();
        //             // let x0 = v0.name.clone();
        //             // let x = ctx.new_stream_name();
        //             // v0.extend(x, StreamKind::DFlatMap(x0, v1).into()).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def flatten[T](s: Stream[Vec[T]]): Stream[T];",
        //     BuiltinDef {
        //         rust: "Stream::flatten",
        //         fun: |_ctx, _v| {
        //             todo!()
        //             // let v0 = v[0].as_stream();
        //             // let x0 = v0.name.clone();
        //             // let x = ctx.new_stream_name();
        //             // v0.extend(x, StreamKind::DFlatten(x0).into()).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def window[I, P, O](s: Stream[I], d: Discretizer, a: Aggregator[I, P, O]): Stream[O];",
        //     BuiltinDef {
        //         rust: "Stream::window",
        //         fun: |_ctx, _v| {
        //             todo!()
        //             // let v0 = v[0].as_stream();
        //             // let v1 = v[1].as_discretizer();
        //             // let v2 = v[2].as_aggregator();
        //             // let x0 = v0.name.clone();
        //             // let x = ctx.new_stream_name();
        //             // v0.extend(x, StreamKind::DWindow(x0, v1, v2).into()).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def keyby[K, T](s: Stream[T], f: fun(T):K): Stream[(K, T)];",
        //     BuiltinDef {
        //         rust: "Stream::keyby",
        //         fun: |_ctx, _v| {
        //             todo!()
        //             // let v0 = v[0].as_stream();
        //             // let v1 = v[1].as_function();
        //             // let x0 = v0.name.clone();
        //             // let x = ctx.new_stream_name();
        //             // v0.extend(x, StreamKind::DKeyby(x0, v1).into()).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def merge[T](s1: Stream[T], s2: Stream[T]): Stream[T];",
        //     BuiltinDef {
        //         rust: "Stream::merge",
        //         fun: |_ctx, _v| {
        //             todo!()
        //             // let v0 = v[0].as_stream();
        //             // let v1 = v[1].as_stream();
        //             // let v2 = v[2].as_vec();
        //             // let x0 = v0.name.clone();
        //             // let x = ctx.new_stream_name();
        //             // v0.extend(x, StreamKind::DMerge(x0, v1, v2).into()).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def unkey[K, T](s: Stream[(K, T)]): Stream[T];",
        //     BuiltinDef {
        //         rust: "Stream::unkey",
        //         fun: |_ctx, _v| {
        //             todo!()
        //             // let v0 = v[0].as_stream();
        //             // let x0 = v0.name.clone();
        //             // let x = ctx.new_stream_name();
        //             // v0.extend(x, StreamKind::DUnkey(x0).into()).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def window[I, P, O](s: Stream[I], d: Discretizer, a: Aggregator[I, P, O]): Stream[O];",
        //     BuiltinDef {
        //         rust: "Stream::window",
        //         fun: |_ctx, _v| {
        //             todo!()
        //             // let v0 = v[0].as_stream();
        //             // let v1 = v[1].as_discretizer();
        //             // let v2 = v[2].as_aggregator();
        //             // let x0 = v0.name.clone();
        //             // let x = ctx.new_stream_name();
        //             // v0.extend(x, StreamKind::DWindow(x0, v1, v2).into()).into()
        //         },
        //     },
        // );
    }
}
