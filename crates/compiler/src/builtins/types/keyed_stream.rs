use linkme::distributed_slice;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type KeyedStream[K, T];",
        codegen: None,
    });

    ctx.declare(
        Decl::Impl {
            aqua: "impl[K, T] KeyedStream[K, T]",
            decls: &[
            ImplDecl::Def {
                aqua: "def source[K, T](r: Reader, e: Encoding, t: (T, Time) => Time): KeyedStream[K, T];",
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_reader();
                    // let v1 = v[1].as_encoding();
                    // let v2 = v[2].as_time_source();
                    todo!()
                },
            },
            ImplDecl::Def {
                aqua: "def sink[K, T](s: KeyedStream[K, T], w: Writer, e: Encoding): Dataflow;",
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_writer();
                    // let v2 = v[2].as_encoding();
                    todo!()
                },
            },
            ImplDecl::Def {
                aqua: "def map[K, A, B](s: KeyedStream[K, A], f: A=>B): KeyedStream[K, B];",
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
            ImplDecl::Def {
                aqua: "def filter[K, T](s: KeyedStream[K, T], f: T=>bool): KeyedStream[K, T];",
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
            ImplDecl::Def {
                aqua: indoc::indoc! {
                    "def flatmap[K, A, B, I](s: KeyedStream[K, A], f: A=>I): KeyedStream[K, B]
                     where Iterator[I, Item=B];"
                },
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
            ImplDecl::Def {
                aqua:"def flatten[K, T, I](s: KeyedStream[K, I]): KeyedStream[K, T]
                    where IntoIterator[I, Item=T];",
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    todo!()
                },
            },
            ImplDecl::Def {
                aqua:indoc::indoc! {
                    "def incrWindow[K, I, P, O](
                        stream: KeyedStream[K, I],
                        assigner: Assigner,
                        lift: I=>P,
                        combine: (P,P)=>P,
                        lower: (P)=>O
                    ): KeyedStream[K, O];"
                },
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_discretizer();
                    // let v2 = v[2].as_aggregator();
                    todo!()
                },
            },
            ImplDecl::Def {
                aqua: "def keyby[K0, K1, T](s: Stream[T], f: (T)=>K0): KeyedStream[K1, T];",
                codegen: None,
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_stream();
                    // let v1 = v[1].as_function();
                    todo!()
                },
            },
            ImplDecl::Def {
                aqua: "def unkey[K, T](s: KeyedStream[K, T]): Stream[T];",
                codegen: None,
                eval: |_ctx, v| {
                    let _v0 = v[0].as_stream();
                    todo!()
                },
            }]
        }
    );
}
