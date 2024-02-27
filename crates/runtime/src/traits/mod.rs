use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;

use crate::builtins::time::Time;

pub trait Data:
    DeepClone + Clone + Send + Sync + Serialize + for<'a> Deserialize<'a> + Unpin + Debug + 'static
{
}
impl<T> Data for T where
    T: DeepClone
        + Clone
        + Send
        + Sync
        + Serialize
        + for<'a> Deserialize<'a>
        + Unpin
        + Debug
        + 'static
{
}

pub trait Key: Data + Eq + PartialEq + Hash {}
impl<T> Key for T where T: Data + Eq + PartialEq + Hash {}

pub trait DeepClone: Clone {
    fn deep_clone(&self) -> Self;
}

macro_rules! impl_deep_clone_tuple {
    { $h:ident $(, $t:ident)* } => {
        impl<$h: DeepClone, $($t: DeepClone),*> DeepClone for ($h, $($t,)*) {
            #[allow(non_snake_case)]
            fn deep_clone(&self) -> Self {
                let ($h, $($t,)*) = self;
                ($h.deep_clone(), $($t.deep_clone(),)*)
            }
        }
        impl_deep_clone_tuple! { $($t),* }
    };
    {} => {}
}

impl_deep_clone_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10);

impl<T: DeepClone> DeepClone for std::rc::Rc<T> {
    fn deep_clone(&self) -> Self {
        std::rc::Rc::new(self.as_ref().deep_clone())
    }
}

impl<T: DeepClone> DeepClone for std::sync::Arc<T> {
    fn deep_clone(&self) -> Self {
        std::sync::Arc::new(self.as_ref().deep_clone())
    }
}

impl<T: DeepClone> DeepClone for std::vec::Vec<T> {
    fn deep_clone(&self) -> Self {
        self.iter().map(|x| x.deep_clone()).collect()
    }
}

impl DeepClone for std::string::String {
    fn deep_clone(&self) -> Self {
        self.clone()
    }
}

impl<T: DeepClone, E: DeepClone> DeepClone for std::result::Result<T, E> {
    fn deep_clone(&self) -> Self {
        match self {
            std::result::Result::Ok(x) => std::result::Result::Ok(x.deep_clone()),
            std::result::Result::Err(x) => std::result::Result::Err(x.deep_clone()),
        }
    }
}

impl<T: DeepClone> DeepClone for std::option::Option<T> {
    fn deep_clone(&self) -> Self {
        self.as_ref().map(|x| x.deep_clone())
    }
}

impl<T: DeepClone> DeepClone for std::cell::RefCell<T> {
    fn deep_clone(&self) -> Self {
        std::cell::RefCell::new(self.borrow().deep_clone())
    }
}

macro_rules! impl_deep_clone_scalar {
    { $t:ty } => {
        impl DeepClone for $t {
            #[inline(always)]
            fn deep_clone(&self) -> Self {
                *self
            }
        }
    };
}

impl_deep_clone_scalar! { () }
impl_deep_clone_scalar! { bool }
impl_deep_clone_scalar! { i8 }
impl_deep_clone_scalar! { i16 }
impl_deep_clone_scalar! { i32 }
impl_deep_clone_scalar! { i64 }
impl_deep_clone_scalar! { i128 }
impl_deep_clone_scalar! { isize }
impl_deep_clone_scalar! { u8 }
impl_deep_clone_scalar! { u16 }
impl_deep_clone_scalar! { u32 }
impl_deep_clone_scalar! { u64 }
impl_deep_clone_scalar! { u128 }
impl_deep_clone_scalar! { usize }
impl_deep_clone_scalar! { f32 }
impl_deep_clone_scalar! { f64 }
impl_deep_clone_scalar! { char }
impl_deep_clone_scalar! { &'static str }

pub trait Timestamp {
    fn timestamp(&self) -> Time;
}
