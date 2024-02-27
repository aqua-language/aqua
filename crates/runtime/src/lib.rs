pub mod builtins;
pub mod formats;
pub mod runner;
// pub mod state;
pub mod traits;
// pub mod logging;

#[cfg(feature = "opt")]
type Hasher = std::hash::BuildHasherDefault<rustc_hash::FxHasher>;

#[cfg(not(feature = "opt"))]
type Hasher = std::collections::hash_map::RandomState;

// #[cfg(feature = "opt")]
// pub type BTreeMap<K, V> = btree_slab::BTreeMap<K, V>;
//
// #[cfg(not(feature = "opt"))]
pub type BTreeMap<K, V> = std::collections::BTreeMap<K, V>;

pub type HashMap<K, V> = std::collections::HashMap<K, V, Hasher>;

pub type SmolHashMap<K, V> = halfbrown::HashMap<K, V, Hasher>;
pub type SmolVec<T> = smallvec::SmallVec<T>;

#[cfg(all(not(target_env = "msvc"), feature = "opt"))]
#[global_allocator]
static GLOBAL: jemalloc::Jemalloc = jemalloc::Jemalloc;

pub mod prelude {
    pub use macros::data;
    pub use macros::unwrap;
    pub use macros::DeepClone;
    pub use macros::New;
    pub use macros::Send;
    pub use macros::Sync;
    pub use macros::Timestamp;
    pub use macros::Unpin;

    pub use crate::builtins::aggregator::Aggregator;
    pub use crate::builtins::array::Array;
    pub use crate::builtins::assigner::Assigner;
    pub use crate::builtins::blob::Blob;
    pub use crate::builtins::dict::Dict;
    pub use crate::builtins::duration::Duration;
    pub use crate::builtins::encoding::Encoding;
    pub use crate::builtins::file::File;
    // pub use crate::builtins::image::Image;
    pub use crate::builtins::keyed_stream::KeyedStream;
    // pub use crate::builtins::matrix::Matrix;
    pub use crate::builtins::im_string::String;
    // pub use crate::builtins::option::Option;
    pub use crate::builtins::path::Path;
    pub use crate::builtins::reader::Reader;
    pub use crate::builtins::result::Result;
    pub use crate::builtins::set::Set;
    pub use crate::builtins::socket::SocketAddr;
    pub use crate::builtins::stream::Stream;
    pub use crate::builtins::time::Time;
    pub use crate::builtins::time_source::TimeSource;
    // pub use crate::builtins::url::Url;
    pub use crate::builtins::vec::Vec;
    pub use crate::builtins::writer::Writer;
    pub use crate::traits::Data;
    pub use crate::traits::DeepClone;

    // #[cfg(feature = "model")]
    // pub use crate::builtins::model::Model;

    pub use crate::runner::context::Context;
    pub use crate::runner::current_thread::CurrentThreadRunner;
    pub use crate::runner::data_parallel::DataParallelRunner;
    pub use crate::runner::task_parallel::TaskParallelRunner;

    // pub use crate::state::Database;
    // pub use crate::state::State;

    pub use serde;
    pub use tokio;

    pub use crate::builtins::stream;
    pub use crate::formats;
}
