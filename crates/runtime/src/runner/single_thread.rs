use crate::context::Context;

pub struct CurrentThreadRunner {}

impl CurrentThreadRunner {
    pub fn run(f: impl Fn(&mut Context)) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime")
            .block_on(async {
                let (tx, rx) = tokio::sync::broadcast::channel(1);
                let mut ctx = Context::new(rx);
                let local_set = tokio::task::LocalSet::new();
                local_set.run_until(async { f(&mut ctx) }).await;
                tx.send(()).unwrap();
                while let Some(_) = ctx.join_set.join_next().await {}
                local_set.await;
            });
    }
}
