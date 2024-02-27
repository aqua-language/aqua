use macros::DeepClone;
use macros::Send;
use macros::Sync;
use macros::Unpin;
use serde::Deserialize;
use serde::Serialize;

use crate::traits::DeepClone;

#[derive(DeepClone, Send, Sync, Unpin, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub struct Cell<T: ?Sized>(pub std::rc::Rc<std::cell::RefCell<T>>);

impl<T> Clone for Cell<T> {
    fn clone(&self) -> Self {
        Cell(self.0.clone())
    }
}

impl<T> std::hash::Hash for Cell<T>
where
    T: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Cell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

impl<T> Cell<T> {
    pub fn new(value: T) -> Cell<T> {
        Cell(std::rc::Rc::new(std::cell::RefCell::new(value)))
    }

    pub fn as_mut(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }

    pub fn as_ref(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }

    pub fn share(&self) -> Self {
        self.clone()
    }
}
