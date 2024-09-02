use macros::Send;
use macros::Sync;
use macros::Unpin;
use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;

#[derive(
    Default, Send, Sync, Unpin, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
#[repr(C)]
pub struct UncheckedCell<T: ?Sized>(pub std::rc::Rc<T>);

impl<T: DeepClone> DeepClone for UncheckedCell<T> {
    fn deep_clone(&self) -> Self {
        Self(self.0.deep_clone())
    }
}

impl<T> Clone for UncheckedCell<T> {
    fn clone(&self) -> Self {
        UncheckedCell(self.0.clone())
    }
}

impl<T> std::ops::Deref for UncheckedCell<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for UncheckedCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

impl<T> UncheckedCell<T> {
    pub(crate) fn new(value: T) -> UncheckedCell<T> {
        UncheckedCell(std::rc::Rc::new(value))
    }

    // # Safety
    //
    // This method can only be used if the caller can guarantee that
    // the value is not read or borrowed during the lifetime of `&mut T`.
    //
    // Borrow checking guarantees that the value is not dropped while
    // the reference is still in use.
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn as_mut_unchecked(&self) -> &mut T {
        let v = std::rc::Rc::into_raw(self.0.clone()) as *mut T;
        unsafe {
            std::rc::Rc::decrement_strong_count(v);
            &mut *v
        }
    }
}
