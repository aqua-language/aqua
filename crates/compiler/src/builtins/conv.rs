use super::types::backend::Backend;
use super::types::iterator::Adaptor;
use super::types::keyed_stream::KeyedStream;
use super::types::storage::Storage;
use super::types::unit::Unit;
use super::value::Value;
use crate::builtins::types::array::Array;
use crate::builtins::types::dataflow::Dataflow;
use crate::builtins::types::function::Function;
use crate::builtins::types::instance::Instance;
use crate::builtins::types::record::Record;
use crate::builtins::types::stream::Stream;
use crate::builtins::types::tuple::Tuple;
use crate::builtins::types::variant::Variant;
use runtime::builtins::blob::Blob;
use runtime::builtins::dict::Dict;
use runtime::builtins::duration::Duration;
use runtime::builtins::file::File;
use runtime::builtins::format::Format;
use runtime::builtins::path::Path;
use runtime::builtins::reader::Reader;
use runtime::builtins::set::Set;
use runtime::builtins::socket::SocketAddr;
use runtime::builtins::time::Time;
use runtime::builtins::url::Url;
use runtime::builtins::window::Window;
use runtime::builtins::writer::Writer;
use runtime::prelude::Bag;
use std::cmp::Ordering;
use std::rc::Rc;

macro_rules! conv {
    {
        $type:ty, $variant:ident, $as:ident
    } => {
        impl Value {
            #[track_caller]
            pub fn $as(&self) -> $type {
                let Value::$variant(v) = self else {
                    unreachable!("{}{:?}", std::panic::Location::caller(), self)
                };
                v.clone()
            }
        }
        impl From<$type> for Value {
            fn from(v: $type) -> Self {
                Value::$variant(v)
            }
        }
    }
}

conv!(Array, Array, as_array);
conv!(Tuple, Tuple, as_tuple);
conv!(Function, Fun, as_function);
conv!(Record, Record, as_record);
conv!(Stream, Stream, as_stream);
conv!(KeyedStream, KeyedStream, as_keyed_stream);
conv!(Storage, Storage, as_storage);
conv!(Variant, Variant, as_variant);
conv!(bool, Bool, as_bool);
conv!(Blob, Blob, as_blob);
conv!(Dict<Value, Value>, Dict, as_dict);
conv!(Window, Window, as_window);
conv!(Duration, Duration, as_duration);
conv!(Dataflow, Dataflow, as_dataflow);
conv!(Format, Format, as_format);
conv!(File, File, as_file);
conv!(Bag<Value>, Bag, as_bag);
conv!(
    runtime::builtins::option::Option<Rc<Value>>,
    Option,
    as_option
);
conv!(Path, Path, as_path);
conv!(Reader, Reader, as_reader);
conv!(
    runtime::builtins::result::Result<Rc<Value>>,
    Result,
    as_result
);
conv!(Set<Value>, Set, as_set);
conv!(SocketAddr, SocketAddr, as_socket_addr);
conv!(runtime::builtins::im_string::String, String, as_string);
conv!(Time, Time, as_time);
conv!(runtime::builtins::vec::Vec<Value>, Vec, as_vec);
conv!(Adaptor, Iterator, as_iterator);
conv!(Writer, Writer, as_writer);
conv!(char, Char, as_char);
conv!(f32, F32, as_f32);
conv!(f64, F64, as_f64);
conv!(i128, I128, as_i128);
conv!(i16, I16, as_i16);
conv!(i32, I32, as_i32);
conv!(i64, I64, as_i64);
conv!(i8, I8, as_i8);
conv!(u128, U128, as_u128);
conv!(u16, U16, as_u16);
conv!(u32, U32, as_u32);
conv!(u64, U64, as_u64);
conv!(u8, U8, as_u8);
conv!(usize, Usize, as_usize);
conv!(Instance, Instance, as_instance);
conv!(Ordering, Ordering, as_ordering);
conv!(Backend, Backend, as_backend);
conv!(Url, Url, as_url);
conv!(Unit, Unit, as_unit);
