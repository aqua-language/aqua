use crate::runner::context::Context;

pub struct DataParallelRunner {
    txs: Vec<std::sync::mpsc::Sender<()>>,
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl DataParallelRunner {
    pub fn new<T: Send + 'static, const N: usize>(
        args: [T; N],
        f: impl Fn(T, &mut Context) + Clone + Send + 'static,
    ) -> Self {
        let mut threads = Vec::with_capacity(args.len());
        let mut txs = Vec::with_capacity(args.len());
        for arg in args {
            let f = f.clone();
            let (runner_tx, runner_rx) = std::sync::mpsc::channel();
            txs.push(runner_tx);
            threads.push(std::thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build runtime")
                    .block_on(async {
                        let ctx = Context::run_local(|ctx| f(arg, ctx)).await;
                        runner_rx.recv().unwrap();
                        ctx.await_termination().await;
                    });
            }));
        }
        Self { txs, threads }
    }

    pub fn run(mut self) {
        for tx in self.txs.iter() {
            tx.send(()).unwrap();
        }
        for thread in self.threads.drain(..) {
            thread.join().expect("Failed to join thread");
        }
    }
}
