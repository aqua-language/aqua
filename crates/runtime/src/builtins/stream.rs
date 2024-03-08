use crate::builtins::time::Time;
use crate::traits::Data;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub mod assert;
pub mod batch;
pub mod drain;
pub mod filter;
pub mod filter_map;
pub mod flat_map;
pub mod fork;
pub mod join;
pub mod keyby;
pub mod map;
pub mod merge;
pub mod operator;
pub mod scan;
pub mod sink;
pub mod source;
pub mod window;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event<T> {
    Data(Time, T),
    Watermark(Time),
    Snapshot(usize),
    Sentinel,
}

#[must_use]
pub struct Stream<T>(pub(crate) Receiver<Event<T>>);

#[must_use]
pub struct Collector<T>(pub(crate) Sender<Event<T>>);

impl<T: Data> Stream<T> {
    pub async fn recv(&mut self) -> Event<T> {
        self.0.recv().await.unwrap_or(Event::Sentinel)
    }
}

impl<T: Data> Collector<T> {
    pub async fn send(&self, event: Event<T>) {
        self.0.send(event).await.ok();
    }
}

impl<T> Stream<T> {
    pub fn new() -> (Collector<T>, Stream<T>) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        (Collector(tx), Stream(rx))
    }
}

#[allow(unused)]
mod wip_stream {
    use crate::ArrayVec;

    use crate::traits::Data;
    use crate::traits::DeepClone;

    use super::Event;
    use crate::builtins::channel::spsc;
    use crate::builtins::channel::spsc::Receiver;
    use crate::builtins::channel::spsc::Sender;

    #[must_use]
    pub struct Stream<T>(pub(crate) Receiver<Event<T>>);

    #[must_use]
    pub struct Collector<T>(pub(crate) Sender<Event<T>>);

    impl<T: Data> Stream<T> {
        pub async fn recv(&mut self) -> Event<T> {
            self.0.recv().await
        }
        pub async fn recv_batch<const N: usize>(&mut self) -> ArrayVec<Event<T>, N> {
            self.0.recv_batch::<N>().await
        }
    }

    impl<T: Data> Collector<T> {
        pub async fn send(&self, event: Event<T>) {
            self.0.send(event).await;
        }
        pub async fn send_batch<const N: usize>(&self, events: ArrayVec<Event<T>, N>) {
            self.0.send_batch::<N>(events).await;
        }
    }

    impl<T> Stream<T> {
        pub fn new() -> (Collector<T>, Stream<T>) {
            let (tx, rx) = spsc::channel(100);
            (Collector(tx), Stream(rx))
        }
    }
}

#[allow(unused)]
mod wip_stream_trait {
    // trait AbstractStream {
    //     type Item;
    //     async fn recv(&mut self) -> Event<Self::Item>;
    // }
    //
    // trait AbstractCollector {
    //     type Item;
    //     async fn send(&self, item: Event<Self::Item>);
    // }
    //
    // impl<T> AbstractStream for Receiver<Event<T>> {
    //     type Item = T;
    //     async fn recv(&mut self) -> Event<T> {
    //         self.recv().await.unwrap_or(Event::Sentinel)
    //     }
    // }
    //
    // impl<T: DeepClone> AbstractCollector for Sender<Event<T>> {
    //     type Item = T;
    //     async fn send(&self, event: Event<T>) {
    //         self.send(event.deep_clone()).await.ok();
    //     }
    // }
    //
    // impl<T: DeepClone> AbstractStream for spsc::Receiver<Event<T>> {
    //     type Item = T;
    //     async fn recv(&mut self) -> Event<T> {
    //         spsc::Receiver::recv(self).await
    //     }
    // }
    //
    // impl<T: DeepClone> AbstractCollector for spsc::Sender<Event<T>> {
    //     type Item = T;
    //     async fn send(&self, event: Event<T>) {
    //         spsc::Sender::send(self, event.deep_clone()).await;
    //     }
    // }
}
