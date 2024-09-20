mod drain;
mod filter;
mod flat_map;
mod fork;
pub mod incr_window;
mod join;
mod keyby;
mod map;
mod merge;
mod scan;
mod sink;
mod unkey;
mod window;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::builtins::time::Time;
use crate::traits::Data;
use crate::traits::Key;
use serde::Deserialize;
use serde::Serialize;

use super::stream::SendError;

#[derive(Debug, Serialize, Deserialize)]
pub enum KeyedEvent<K, T> {
    Data(Time, K, T),
    Watermark(Time),
    Snapshot(usize),
    Sentinel,
}

pub struct KeyedStream<K: Data, T: Data>(pub(crate) Receiver<KeyedEvent<K, T>>);

pub struct KeyedCollector<K: Data, T: Data>(pub(crate) Sender<KeyedEvent<K, T>>);

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub async fn recv(&mut self) -> KeyedEvent<K, T> {
        self.0.recv().await.unwrap_or(KeyedEvent::Sentinel)
    }
}

impl<K: Data, T: Data> KeyedCollector<K, T> {
    pub async fn send(&self, event: KeyedEvent<K, T>) -> Result<(), SendError> {
        self.0.send(event).await.map_err(|_| SendError::Closed)
    }
}

impl<K: Data, T: Data> KeyedStream<K, T> {
    pub(crate) fn new() -> (KeyedCollector<K, T>, KeyedStream<K, T>) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        (KeyedCollector(tx), KeyedStream(rx))
    }
}
