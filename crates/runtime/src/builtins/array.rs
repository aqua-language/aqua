use macros::Send;
use macros::Sync;
use macros::Unpin;

use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::ser::SerializeTuple;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::HashMap;
use crate::builtins::dict::Dict;
use crate::builtins::set::Set;
use crate::builtins::vec::Vec;
use crate::traits::DeepClone;

#[derive(Clone, Debug, Send, Sync, Unpin, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct Array<T, const N: usize>(pub [T; N]);

impl<T: DeepClone, const N: usize> DeepClone for Array<T, N> {
    fn deep_clone(&self) -> Self {
        let data = {
            let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
            for (new, old) in data.iter_mut().zip(self.0.iter()) {
                new.write(old.deep_clone());
            }
            unsafe { std::mem::transmute_copy::<[MaybeUninit<T>; N], [T; N]>(&data) }
        };
        Array(data)
    }
}

impl<T, const N: usize> Array<T, N> {
    pub fn new(data: [T; N]) -> Self {
        Array(data)
    }

    pub fn set(mut self, index: usize, value: T) -> Self {
        self.0[index] = value;
        self
    }

    pub fn get(self, index: usize) -> T
    where
        T: Clone,
    {
        self.0[index].clone()
    }

    pub fn into_vec(self) -> Vec<T>
    where
        T: Clone,
    {
        Vec::from(self.0.to_vec())
    }

    pub fn into_set(self) -> Set<T>
    where
        T: Clone + Hash + Eq,
    {
        Set::from(self.0.into_iter().collect::<std::collections::HashSet<_>>())
    }
}

impl<K, V, const N: usize> Array<(K, V), N>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub fn into_dict(self) -> Dict<K, V>
    where
        V: Clone,
    {
        Dict::from(self.0.into_iter().collect::<HashMap<_, _>>())
    }
}

impl<T, const N: usize> From<[T; N]> for Array<T, N> {
    fn from(data: [T; N]) -> Self {
        Array(data)
    }
}

impl<T: Serialize, const N: usize> Serialize for Array<T, N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut s = s.serialize_tuple(N)?;
        for item in &self.0 {
            s.serialize_element(item)?;
        }
        s.end()
    }
}

impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for Array<T, N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Array<T, N>, D::Error> {
        Ok(Array(d.deserialize_tuple(N, ArrayVisitor(PhantomData))?))
    }
}

struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for ArrayVisitor<T, N> {
    type Value = [T; N];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&format!("an array of length {}", N))
    }

    #[inline]
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut data = std::vec::Vec::with_capacity(N);
        for _ in 0..N {
            match (seq.next_element())? {
                Some(val) => data.push(val),
                None => return Err(serde::de::Error::invalid_length(N, &self)),
            }
        }
        match data.try_into() {
            Ok(arr) => Ok(arr),
            Err(_) => unreachable!(),
        }
    }
}
