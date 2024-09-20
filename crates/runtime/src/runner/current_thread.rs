use crate::runner::context::Context;

pub struct CurrentThreadRunner {}

impl CurrentThreadRunner {
    pub fn run(f: impl FnOnce(&mut Context)) {
        let future = async {
            let ctx = Context::run_local(f).await;
            ctx.await_termination().await;
        };
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime")
            .block_on(future)
    }
}
