#![allow(unused)]

mod conv;
pub mod de;
pub mod decls;
mod eq;
mod hash;
mod ord;
mod ser;

pub use decls::types::array::Array;
pub use decls::types::dataflow::Dataflow;
pub use decls::types::function::Fun;
pub use decls::types::instance::Instance;
pub use decls::types::record::Record;
pub use decls::types::stream::Stream;
pub use decls::types::tuple::Tuple;
pub use decls::types::variant::Variant;

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
    Aggregator(Aggregator<Rc<Value>, Rc<Value>, Rc<Value>, Rc<Value>>),
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
    TimeSource(TimeSource<Rc<Value>>),
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
    Ordering(std::cmp::Ordering),
}

impl Value {
    fn rc(&self) -> Rc<Self> {
        Rc::new(self.clone())
    }
}
