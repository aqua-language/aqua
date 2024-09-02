use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_keyed_stream(&mut self) {
        self.declare_type(
            "type KeyedStream[K, T];",
            BuiltinType {
                rust: "KeyedStream",
            },
        );
        self.declare_def(
            "def keyed_source[K, T](r: Reader, e: Encoding, t: TimeSource[T]): KeyedStream[K, T];",
            BuiltinDef {
                rust: "KeyedStream::source",
                fun: |_ctx, _v| {
                    // let v0 = v[0].as_reader();
                    // let v1 = v[1].as_encoding();
                    // let v2 = v[2].as_time_source();
                    todo!()
                },
            },
        );

        self.declare_def(
            "def keyed_sink[K, T](s: KeyedStream[K, T], w: Writer, e: Encoding): Dataflow;",
            BuiltinDef {
                rust: "KeyedStream::sink",
                fun: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_writer();
                    // let v2 = v[2].as_encoding();
                    todo!()
                },
            },
        );

        self.declare_def(
            "def keyed_map[K, A, B](s: KeyedStream[K, A], f: fun(A):B): KeyedStream[K, B];",
            BuiltinDef {
                rust: "KeyedStream::map",
                fun: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
        );

        self.declare_def(
            "def keyed_filter[K, T](s: KeyedStream[K, T], f: fun(T): bool): KeyedStream[K, T];",
            BuiltinDef {
                rust: "KeyedStream::filter",
                fun: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
        );

        self.declare_def(
            "def keyed_flatmap[K, A, B, I](s: KeyedStream[K, A], f: fun(A):I): KeyedStream[K, B]
             where Iterator[I, Item=B];",
            BuiltinDef {
                rust: "KeyedStream::flatmap",
                fun: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
        );

        self.declare_def(
            "def keyed_flatten[K, T, I](s: KeyedStream[K, I]): KeyedStream[K, T]
             where IntoIterator[I, Item=T];",
            BuiltinDef {
                rust: "KeyedStream::flatten",
                fun: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    todo!()
                },
            },
        );

        self.declare_def(
            "def keyed_window[K, I, P, O](s: KeyedStream[K, I], d: Discretizer, a: Aggregator[I, P, O]): KeyedStream[K, O];",
            BuiltinDef {
                rust: "KeyedStream::window",
                fun: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_discretizer();
                    // let v2 = v[2].as_aggregator();
                    todo!()
                },
            },
        );

        self.declare_def(
            "def keyed_keyby[K0, K1, T](s: Stream[T], f: fun(T):K0): KeyedStream[K1, T];",
            BuiltinDef {
                rust: "KeyedStream::keyby",
                fun: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
        );

        self.declare_def(
            "def unkey[K, T](s: KeyedStream[K, T]): Stream[T];",
            BuiltinDef {
                rust: "KeyedStream::unkey",
                fun: |_ctx, v| {
                    let _v0 = v[0].as_stream();
                    todo!()
                },
            },
        );
    }
}
