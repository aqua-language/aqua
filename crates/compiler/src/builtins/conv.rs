use std::rc::Rc;

use runtime::builtins::aggregator::Aggregator;
use runtime::builtins::assigner::Assigner;
use runtime::builtins::blob::Blob;
use runtime::builtins::dict::Dict;
use runtime::builtins::duration::Duration;
use runtime::builtins::encoding::Encoding;
use runtime::builtins::file::File;
// use runtime::builtins::image::Image;
// use runtime::builtins::model::Model;
use runtime::builtins::path::Path;
use runtime::builtins::reader::Reader;
use runtime::builtins::set::Set;
use runtime::builtins::socket::SocketAddr;
use runtime::builtins::time::Time;
use runtime::builtins::time_source::TimeSource;
// use runtime::builtins::url::Url;
use runtime::builtins::writer::Writer;

use super::Value;
use crate::builtins::decls::array::Array;
use crate::builtins::decls::dataflow::Dataflow;
use crate::builtins::decls::function::Fun;
use crate::builtins::decls::instance::Instance;
use crate::builtins::decls::record::Record;
use crate::builtins::decls::stream::Stream;
use crate::builtins::decls::tuple::Tuple;
use crate::builtins::decls::variant::Variant;

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
conv!(Fun, Fun, as_function);
// conv!(Matrix, Matrix, as_matrix);
conv!(Record, Record, as_record);
conv!(Stream, Stream, as_stream);
conv!(Variant, Variant, as_variant);
conv!(bool, Bool, as_bool);
conv!(Aggregator<Fun, Fun, Fun, Fun>, Aggregator, as_aggregator);
conv!(Blob, Blob, as_blob);
conv!(Dict<Value, Value>, Dict, as_dict);
conv!(Assigner, Discretizer, as_discretizer);
conv!(Duration, Duration, as_duration);
conv!(Dataflow, Dataflow, as_dataflow);
conv!(Encoding, Encoding, as_encoding);
conv!(File, File, as_file);
#[cfg(feature = "model")]
conv!(Model, Model, as_model);
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
conv!(TimeSource<Fun>, TimeSource, as_time_source);
// conv!(Url, Url, as_url);
conv!(runtime::builtins::vec::Vec<Value>, Vec, as_vec);

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
// conv!(Image, Image, as_image);
