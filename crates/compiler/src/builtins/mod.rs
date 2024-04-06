#![allow(unused)]

mod conv;
pub mod de;
pub mod decls;
mod eq;
mod hash;
mod ord;
mod ser;

pub use decls::array::Array;
pub use decls::dataflow::Dataflow;
pub use decls::function::Fun;
pub use decls::instance::Instance;
pub use decls::record::Record;
pub use decls::stream::Stream;
pub use decls::tuple::Tuple;
pub use decls::variant::Variant;

use runtime::prelude::Aggregator;
use runtime::prelude::Assigner;
use runtime::prelude::Blob;
use runtime::prelude::Dict;
use runtime::prelude::Duration;
use runtime::prelude::Encoding;
use runtime::prelude::File;
use runtime::prelude::Path;
use runtime::prelude::Reader;
use runtime::prelude::Set;
use runtime::prelude::SocketAddr;
use runtime::prelude::Time;
use runtime::prelude::TimeSource;
use runtime::prelude::Writer;

use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Aggregator(Aggregator<Fun, Fun, Fun, Fun>),
    Array(Array),
    Blob(Blob),
    Bool(bool),
    Char(char),
    Dict(Dict<Value, Value>),
    Discretizer(Assigner),
    Duration(Duration),
    Encoding(Encoding),
    F32(f32),
    F64(f64),
    File(File),
    Fun(Fun),
    I128(i128),
    I16(i16),
    I32(i32),
    I64(i64),
    I8(i8),
    // Image(Image),
    // Matrix(Matrix),
    // Model(Model),
    Option(runtime::builtins::option::Option<Rc<Value>>),
    Path(Path),
    Reader(Reader),
    Record(Record),
    Result(runtime::builtins::result::Result<Rc<Value>>),
    Set(Set<Value>),
    SocketAddr(SocketAddr),
    Stream(Stream),
    Dataflow(Dataflow),
    String(runtime::builtins::im_string::String),
    Time(Time),
    TimeSource(TimeSource<Fun>),
    Tuple(Tuple),
    U128(u128),
    U16(u16),
    U32(u32),
    U64(u64),
    U8(u8),
    // Url(Url),
    Usize(usize),
    Variant(Variant),
    Vec(runtime::builtins::vec::Vec<Value>),
    Writer(Writer),
    Instance(Instance),
}
