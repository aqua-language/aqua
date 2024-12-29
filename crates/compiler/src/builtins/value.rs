use runtime::builtins::url::Url;
use runtime::prelude::Blob;
use runtime::prelude::Dict;
use runtime::prelude::Duration;
use runtime::prelude::File;
use runtime::prelude::Format;
use runtime::prelude::Path;
use runtime::prelude::Reader;
use runtime::prelude::Send;
use runtime::prelude::Set;
use runtime::prelude::SocketAddr;
use runtime::prelude::Sync;
use runtime::prelude::Time;
use runtime::prelude::Window;
use runtime::prelude::Writer;

pub use super::types::array::Array;
pub use super::types::backend::Backend;
pub use super::types::dataflow::Dataflow;
pub use super::types::function::Function;
pub use super::types::instance::Instance;
pub use super::types::iterator::Adaptor;
use super::types::keyed_stream::KeyedStream;
pub use super::types::record::Record;
pub use super::types::stream::Stream;
pub use super::types::tuple::Tuple;
use super::types::unit::Unit;
pub use super::types::variant::Variant;

use std::rc::Rc;

#[derive(Clone, Debug, Send, Sync)]
pub enum Value {
    Array(Array),
    Blob(Blob),
    Bool(bool),
    Char(char),
    Dict(Dict<Value, Value>),
    Window(Window),
    Duration(Duration),
    Format(Format),
    F32(f32),
    F64(f64),
    File(File),
    Fun(Function),
    I128(i128),
    I16(i16),
    I32(i32),
    I64(i64),
    I8(i8),
    Bag(runtime::prelude::Bag<Value>),
    Option(runtime::builtins::option::Option<Rc<Value>>),
    Path(Path),
    Reader(Reader),
    Record(Record),
    Result(runtime::builtins::result::Result<Rc<Value>>),
    Set(Set<Value>),
    SocketAddr(SocketAddr),
    Stream(Stream),
    KeyedStream(KeyedStream),
    Dataflow(Dataflow),
    String(runtime::builtins::im_string::String),
    Time(Time),
    Tuple(Tuple),
    U128(u128),
    U16(u16),
    U32(u32),
    U64(u64),
    U8(u8),
    Usize(usize),
    Url(Url),
    Variant(Variant),
    Vec(runtime::builtins::vec::Vec<Value>),
    Writer(Writer),
    Instance(Instance),
    Ordering(std::cmp::Ordering),
    Backend(Backend),
    Range(runtime::builtins::range::Range<Rc<Value>>),
    Iterator(Adaptor),
    Storage(super::types::storage::Storage),
    Unit(Unit),
}

impl Value {
    pub fn rc(&self) -> Rc<Self> {
        Rc::new(self.clone())
    }
}
