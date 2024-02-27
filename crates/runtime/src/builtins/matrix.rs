#![allow(unused)]
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use macros::Share;
use ndarray::ArrayBase;
use ndarray::Axis;
use ndarray::Dim;
use ndarray::IxDyn;
use ndarray::IxDynImpl;
use ndarray::OwnedRepr;
use ndarray::SliceInfo;
use ndarray::SliceInfoElem;
use num::Num;
use num::Zero;
use serde::Deserialize;
use serde::Serialize;

use crate::builtins::array::Array;
use crate::builtins::cell::Cell;
use crate::builtins::iterator::Iter;
use crate::builtins::vec::Vec;
use crate::traits::Data;
use crate::traits::Share;

#[derive(Debug, Share, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub struct Matrix<T>(pub Cell<Inner<T>>);

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub struct Inner<T>(pub ArrayBase<OwnedRepr<T>, Dim<IxDynImpl>>);

impl<T> std::ops::Deref for Inner<T> {
    type Target = ArrayBase<OwnedRepr<T>, Dim<IxDynImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Inner<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Clone> Share for Inner<T> {
    fn deep_clone(&self) -> Self {
        Inner(self.0.clone())
    }
}

impl<T> Matrix<T> {
    pub fn zeros(shape: Vec<usize>) -> Self
    where
        T: Clone + Zero,
    {
        Matrix::from(ArrayBase::zeros(shape.0.take()))
    }

    pub fn insert_axis(mut self, axis: usize)
    where
        T: Clone,
    {
        self.0.insert_axis_inplace(Axis(axis));
    }

    pub fn remove_axis(mut self, axis: usize)
    where
        T: Clone,
    {
        self.0.index_axis_inplace(Axis(axis), 0);
    }

    pub fn into_vec(self) -> Vec<T>
    where
        T: Clone,
    {
        Vec::from(self.0.take().0.into_raw_vec())
    }

    pub fn iter(&self) -> Iter<impl Iterator<Item = T> + '_>
    where
        T: Clone,
    {
        Iter::new(self.0.iter().cloned())
    }

    pub fn transpose(self) -> Self
    where
        T: Clone,
    {
        Matrix::from(self.0.t().into_owned())
    }

    // pub fn slice<const N: usize>(self, indices: Array<Index, N>) -> Self
    // where
    //     T: Clone,
    // {
    //     Matrix::from(self.0.map(|this| {
    //         let x: SliceInfo<std::vec::Vec<SliceInfoElem>, IxDyn, IxDyn> =
    //             SliceInfo::try_from(vec![SliceInfoElem::from(1..)]).unwrap();
    //         this.0.slice(x).into_owned()
    //     }))
    // }
}

impl<T> From<ArrayBase<OwnedRepr<T>, Dim<IxDynImpl>>> for Matrix<T> {
    fn from(array: ArrayBase<OwnedRepr<T>, Dim<IxDynImpl>>) -> Self {
        Matrix(Cell::new(Inner(array)))
    }
}

/// [a]
/// [:]
/// [a:b:2]
/// [a::-1]
/// [np.newaxis]
#[derive(Debug, Share, Clone, Serialize, Deserialize)]
pub enum Index {
    Range(usize, usize),
    Index(usize),
}
