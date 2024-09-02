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

pub use super::types::array::Array;
use super::types::backend::Backend;
pub use super::types::dataflow::Dataflow;
pub use super::types::function::Fun;
pub use super::types::instance::Instance;
pub use super::types::record::Record;
pub use super::types::stream::Stream;
pub use super::types::tuple::Tuple;
pub use super::types::variant::Variant;

use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Aggregator(Aggregator<Rc<Value>, Rc<Value>, Rc<Value>, Rc<Value>>),
    Array(Array),
    Blob(Blob),
    Bool(bool),
    Char(char),
    Dict(Dict<Value, Value>),
    Assigner(Assigner),
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
    Usize(usize),
    Variant(Variant),
    Vec(runtime::builtins::vec::Vec<Value>),
    Writer(Writer),
    Instance(Instance),
    Ordering(std::cmp::Ordering),
    Backend(Backend),
    Range(runtime::builtins::range::Range<Rc<Value>>)
}

impl Value {
    pub fn rc(&self) -> Rc<Self> {
        Rc::new(self.clone())
    }
}
