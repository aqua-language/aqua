use std::future::Future;

use crate::builtins::keyed_stream::KeyedCollector;
use crate::builtins::keyed_stream::KeyedStream;
use crate::builtins::stream::Collector;
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

    pub fn spawn<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut rx = self.rx.resubscribe();
        self.join_set.spawn(async move {
            rx.recv().await.expect("Channel should not be closed.");
            f.await
        });
    }

    pub fn operator<T, F>(&mut self, f: impl FnOnce(Collector<T>) -> F) -> Stream<T>
    where
        F: Future<Output = ()> + Send + 'static,
        T: Data,
    {
        let (tx, rx) = Stream::new();
        self.spawn(f(tx));
        rx
    }

    pub fn keyed_operator<K, T, F>(
        &mut self,
        f: impl FnOnce(KeyedCollector<K, T>) -> F,
    ) -> KeyedStream<K, T>
    where
        F: Future<Output = ()> + Send + 'static,
        K: Data,
        T: Data,
    {
        let (tx, rx) = KeyedStream::new();
        self.spawn(f(tx));
        rx
    }

    pub fn co_operator<T0, T1, F>(
        &mut self,
        f: impl FnOnce(Collector<T0>, Collector<T1>) -> F,
    ) -> (Stream<T0>, Stream<T1>)
    where
        F: Future<Output = ()> + Send + 'static,
        T0: Data,
        T1: Data,
    {
        let (tx0, rx0) = Stream::new();
        let (tx1, rx1) = Stream::new();
        self.spawn(f(tx0, tx1));
        (rx0, rx1)
    }

    pub fn keyed_co_operator<K0, K1, T0, T1, F>(
        &mut self,
        f: impl FnOnce(KeyedCollector<K0, T0>, KeyedCollector<K1, T1>) -> F,
    ) -> (KeyedStream<K0, T0>, KeyedStream<K1, T1>)
    where
        F: Future<Output = ()> + Send + 'static,
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

    pub fn sink<F>(&mut self, f: impl FnOnce() -> F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(f());
    }
}

pub fn array_unzip<const N: usize, A, B>(f: impl Fn() -> (A, B)) -> ([A; N], [B; N]) {
    let mut a0: [std::mem::MaybeUninit<A>; N] =
        unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    let mut a1: [std::mem::MaybeUninit<B>; N] =
        unsafe { std::mem::MaybeUninit::uninit().assume_init() };

    for (l, r) in a0[..].iter_mut().zip(a1.iter_mut()) {
        let (tx, rx) = f();
        unsafe {
            std::ptr::write(l.as_mut_ptr(), tx);
            std::ptr::write(r.as_mut_ptr(), rx);
        }
    }

    let a0 = unsafe { std::mem::transmute_copy::<[std::mem::MaybeUninit<A>; N], [A; N]>(&a0) };
    let a1 = unsafe { std::mem::transmute_copy::<[std::mem::MaybeUninit<B>; N], [B; N]>(&a1) };
    (a0, a1)
}
