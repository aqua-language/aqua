use crate::ArrayVec;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use macros::Send;
use macros::Sync;
use macros::Unpin;

pub fn channel<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let channel = Channel(Rc::new(RefCell::new(ChannelInner {
        buffer: VecDeque::with_capacity(cap),
        waker: None,
    })));
    (Sender(channel.clone()), Receiver(channel))
}

pub struct ChannelInner<T> {
    buffer: VecDeque<T>,
    waker: Option<Waker>,
}

pub struct Channel<T>(pub(crate) Rc<RefCell<ChannelInner<T>>>);

impl<T> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Channel(self.0.clone())
    }
}

#[derive(Send, Sync)]
pub struct Sender<T>(pub(crate) Channel<T>);

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender(self.0.clone())
    }
}

#[derive(Send, Sync)]
pub struct Receiver<T>(pub(crate) Channel<T>);

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver(self.0.clone())
    }
}

#[derive(Unpin)]
pub struct SenderFuture<T> {
    chan: Channel<T>,
    data: Option<T>,
}

#[derive(Unpin)]
pub struct BatchSenderFuture<const N: usize, T> {
    chan: Channel<T>,
    data: ArrayVec<T, N>,
}

#[derive(Unpin)]
pub struct ReceiverFuture<T> {
    chan: Channel<T>,
}

#[derive(Unpin)]
pub struct BatchReceiverFuture<const N: usize, T> {
    chan: Channel<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, data: T) -> SenderFuture<T> {
        SenderFuture {
            chan: self.0.clone(),
            data: Some(data),
        }
    }

    pub fn send_batch<const N: usize>(&self, mut data: ArrayVec<T, N>) -> BatchSenderFuture<N, T> {
        data.reverse();
        BatchSenderFuture {
            chan: self.0.clone(),
            data,
        }
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> ReceiverFuture<T> {
        ReceiverFuture {
            chan: self.0.clone(),
        }
    }
    pub fn recv_batch<const N: usize>(&self) -> BatchReceiverFuture<N, T> {
        BatchReceiverFuture {
            chan: self.0.clone(),
        }
    }
}

impl<T> Future for ReceiverFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut chan = self.chan.0.borrow_mut();
        if let Some(data) = chan.buffer.pop_back() {
            if let Some(waker) = chan.waker.take() {
                waker.wake();
            }
            Poll::Ready(data)
        } else {
            chan.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<T> Future for SenderFuture<T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        let mut chan = this.chan.0.borrow_mut();
        if chan.buffer.len() < chan.buffer.capacity() {
            let data = this.data.take().unwrap();
            if let Some(waker) = chan.waker.take() {
                waker.wake();
            }
            chan.buffer.push_front(data);
            Poll::Ready(())
        } else {
            chan.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<const N: usize, T> Future for BatchReceiverFuture<N, T> {
    type Output = ArrayVec<T, N>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        let mut chan = this.chan.0.borrow_mut();
        if !chan.buffer.is_empty() {
            let mut data = ArrayVec::new();
            for _ in 0..N.min(chan.buffer.len()) {
                data.push(chan.buffer.pop_back().unwrap());
            }
            if let Some(waker) = chan.waker.take() {
                waker.wake();
            }
            Poll::Ready(data)
        } else {
            chan.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<const N: usize, T> Future for BatchSenderFuture<N, T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        let mut chan = this.chan.0.borrow_mut();
        let data = &mut this.data;
        let (len, cap) = { (chan.buffer.len(), chan.buffer.capacity()) };
        if len < cap {
            for _ in 0..data.len().min(len) {
                chan.buffer.push_front(data.pop().unwrap());
            }
            if let Some(waker) = chan.waker.take() {
                waker.wake();
            }
            if data.is_empty() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        } else {
            chan.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
