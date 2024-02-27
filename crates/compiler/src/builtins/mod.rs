pub mod dynamic {
    pub mod array;
    pub mod dataflow;
    pub mod function;
    pub mod instance;
    // pub mod matrix;
    pub mod record;
    pub mod stream;
    pub mod tuple;
    pub mod variant;
}

mod conv;
pub mod de;
mod eq;
mod hash;
mod ord;
mod ser;

pub use dynamic::array::Array;
pub use dynamic::dataflow::Dataflow;
pub use dynamic::dataflow::Sink;
pub use dynamic::function::Fun;
pub use dynamic::instance::Instance;
// pub use dynamic::matrix::Matrix;
pub use dynamic::record::Record;
pub use dynamic::stream::Stream;
pub use dynamic::tuple::Tuple;
pub use dynamic::variant::Variant;

use runtime::prelude::Aggregator;
use runtime::prelude::Assigner;
use runtime::prelude::Blob;
use runtime::prelude::Dict;
use runtime::prelude::Duration;
use runtime::prelude::Encoding;
use runtime::prelude::File;
// use runtime::prelude::Image;
// use runtime::prelude::Model;
use runtime::prelude::Path;
use runtime::prelude::Reader;
use runtime::prelude::Set;
use runtime::prelude::SocketAddr;
use runtime::prelude::Time;
use runtime::prelude::TimeSource;
// use runtime::prelude::Url;
use runtime::prelude::Writer;

use std::rc::Rc;

#[derive(Clone)]
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

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Aggregator(v) => v.fmt(f),
            Value::Array(v) => v.fmt(f),
            Value::Blob(v) => v.fmt(f),
            Value::Bool(v) => v.fmt(f),
            Value::Char(v) => v.fmt(f),
            Value::Dict(v) => v.fmt(f),
            Value::Discretizer(v) => v.fmt(f),
            Value::Duration(v) => v.fmt(f),
            Value::Encoding(v) => v.fmt(f),
            Value::F32(v) => v.fmt(f),
            Value::F64(v) => v.fmt(f),
            Value::File(v) => v.fmt(f),
            Value::Fun(v) => v.fmt(f),
            Value::I128(v) => v.fmt(f),
            Value::I16(v) => v.fmt(f),
            Value::I32(v) => v.fmt(f),
            Value::I64(v) => v.fmt(f),
            Value::I8(v) => v.fmt(f),
            // Value::Matrix(v) => v.fmt(f),
            // Value::Model(v) => v.fmt(f),
            Value::Option(v) => v.fmt(f),
            Value::Path(v) => v.fmt(f),
            Value::Reader(v) => v.fmt(f),
            Value::Record(v) => v.fmt(f),
            Value::Result(v) => v.fmt(f),
            Value::Set(v) => v.fmt(f),
            Value::SocketAddr(v) => v.fmt(f),
            Value::Stream(v) => v.fmt(f),
            Value::Dataflow(v) => v.fmt(f),
            Value::String(v) => v.fmt(f),
            Value::Time(v) => v.fmt(f),
            Value::TimeSource(v) => v.fmt(f),
            Value::Tuple(v) => v.fmt(f),
            Value::U128(v) => v.fmt(f),
            Value::U16(v) => v.fmt(f),
            Value::U32(v) => v.fmt(f),
            Value::U64(v) => v.fmt(f),
            Value::U8(v) => v.fmt(f),
            // Value::Url(v) => v.fmt(f),
            Value::Usize(v) => v.fmt(f),
            Value::Variant(v) => v.fmt(f),
            Value::Vec(v) => v.fmt(f),
            Value::Writer(v) => v.fmt(f),
            Value::Instance(v) => v.fmt(f),
            // Value::Image(v) => v.fmt(f),
        }
    }
}
