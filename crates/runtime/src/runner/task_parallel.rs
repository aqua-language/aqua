use crate::runner::context::Context;

pub struct TaskParallelRunner {
    tx: std::sync::mpsc::SyncSender<()>,
}

impl TaskParallelRunner {
    #[cfg(feature = "multi-thread")]
    pub fn new(f: impl FnOnce(&mut Context) + Send + 'static) -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<()>(1);
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime")
            .block_on(async {
                let ctx = Context::run(f);
                rx.recv().unwrap();
                ctx.await_termination().await;
            });
        Self { tx }
    }

    #[cfg(not(feature = "multi-thread"))]
    pub fn new(f: impl FnOnce(&mut Context) + Send + 'static) -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<()>(1);
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime")
            .block_on(async {
                let ctx = Context::run(f);
                rx.recv().unwrap();
                ctx.await_termination().await;
            });
        Self { tx }
    }

    pub fn run(self) {
        self.tx.send(()).unwrap();
    }
}
