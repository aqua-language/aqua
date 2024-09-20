use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::sync::OnceLock;

use append_only_vec::AppendOnlyVec;

pub struct ConcurrentMap<K, V> {
    readonly_vec: AppendOnlyVec<V>,
    writeonly_map: OnceLock<Mutex<HashMap<K, Uid>>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uid(u16);

impl<K: Eq + Hash, V> ConcurrentMap<K, V> {
    pub const fn new() -> ConcurrentMap<K, V> {
        ConcurrentMap {
            readonly_vec: AppendOnlyVec::new(),
            writeonly_map: OnceLock::new(),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Uid {
        let mut state = self
            .writeonly_map
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();
        state.get(&key).copied().unwrap_or_else(|| {
            let uid = Uid(self.readonly_vec.push(value) as u16);
            let prev = state.insert(key, uid);
            debug_assert!(prev.is_none());
            uid
        })
    }

    pub fn get(&self, key: Uid) -> &V {
        &self.readonly_vec[key.0 as usize]
    }
}
