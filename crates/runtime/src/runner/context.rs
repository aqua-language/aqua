use std::future::Future;

use crate::builtins::keyed_stream::KeyedCollector;
use crate::builtins::keyed_stream::KeyedStream;
use crate::builtins::stream::Collector;
use crate::builtins::stream::SendError;
use crate::builtins::stream::Stream;
use crate::traits::Data;

pub struct Context {
    join_set: tokio::task::JoinSet<()>,
    local_set: Option<tokio::task::LocalSet>,
    tx: tokio::sync::broadcast::Sender<()>,
    rx: tokio::sync::broadcast::Receiver<()>,
}

impl Default for Context {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        Self {
            join_set: tokio::task::JoinSet::new(),
            local_set: None,
            tx,
            rx,
        }
    }
}

#[macro_export]
macro_rules! try_pair {
    ($e:expr) => {
        let (a, b) = $e;
        (a?, b?)
    };
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run_local(f: impl FnOnce(&mut Context)) -> Self {
        let mut ctx = Self::new();
        let local_set = tokio::task::LocalSet::new();
        local_set.run_until(async { f(&mut ctx) }).await;
        ctx.local_set = Some(local_set);
        ctx
    }

    pub fn run(f: impl FnOnce(&mut Context) + Send + 'static) -> Self {
        let mut ctx = Self::new();
        f(&mut ctx);
        ctx
    }

    pub async fn await_termination(mut self) {
        self.tx.send(()).unwrap();
        while let Some(result) = self.join_set.join_next().await {
            result.expect("Task should not panic.");
        }
        if let Some(local_set) = self.local_set {
            local_set.await;
        }
    }

    pub fn spawn<Fut>(&mut self, f: Fut)
    where
        Fut: Future<Output = Result<(), SendError>> + Send + 'static,
    {
        let mut rx = self.rx.resubscribe();
        self.join_set.spawn(async move {
            rx.recv().await.expect("Channel should not be closed.");
            f.await.ok();
        });
    }

    /// An operator with one input and one output.
    pub fn operator<T, F, Fut>(&mut self, f: F) -> Stream<T>
    where
        F: FnOnce(Collector<T>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), SendError>> + Send + 'static,
        T: Data,
    {
        let (tx, rx) = Stream::new();
        self.spawn(f(tx));
        rx
    }

    /// A keyed operator with one input and one output.
    pub fn keyed_operator<K, T, F, Fut>(&mut self, f: F) -> KeyedStream<K, T>
    where
        F: FnOnce(KeyedCollector<K, T>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), SendError>> + Send + 'static,
        K: Data,
        T: Data,
    {
        let (tx, rx) = KeyedStream::new();
        self.spawn(f(tx));
        rx
    }

    /// An operator with two inputs and one output.
    pub fn co_operator<T0, T1, F, Fut>(&mut self, f: F) -> (Stream<T0>, Stream<T1>)
    where
        F: FnOnce(Collector<T0>, Collector<T1>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), SendError>> + Send + 'static,
        T0: Data,
        T1: Data,
    {
        let (tx0, rx0) = Stream::new();
        let (tx1, rx1) = Stream::new();
        self.spawn(f(tx0, tx1));
        (rx0, rx1)
    }

    /// A keyed operator with two inputs and two outputs.
    pub fn keyed_co_operator<K0, K1, T0, T1, F>(
        &mut self,
        f: impl FnOnce(KeyedCollector<K0, T0>, KeyedCollector<K1, T1>) -> F,
    ) -> (KeyedStream<K0, T0>, KeyedStream<K1, T1>)
    where
        F: Future<Output = Result<(), SendError>> + Send + 'static,
        T0: Data,
        T1: Data,
        K0: Data,
        K1: Data,
    {
        let (tx0, rx0) = KeyedStream::new();
        let (tx1, rx1) = KeyedStream::new();
        self.spawn(f(tx0, tx1));
        (rx0, rx1)
    }

    /// An operator with one input and zero outputs.
    pub fn sink<F>(&mut self, f: impl FnOnce() -> F)
    where
        F: Future<Output = Result<(), SendError>> + Send + 'static,
    {
        self.spawn(f());
    }
}
