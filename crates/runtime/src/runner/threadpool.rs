#![allow(unused)]

use std::future::Future;
use std::path::Path;

use logging::Logger;

pub struct TaskParallelRunner {
    #[cfg(feature = "thread-pinning")]
    topology: std::sync::Arc<std::sync::Mutex<hwloc::Topology>>,
    threads: Vec<std::thread::JoinHandle<()>>,
    logger: Logger,
}

impl TaskParallelRunner {
    pub fn new(logger: Logger) -> Self {
        Self {
            #[cfg(feature = "thread-pinning")]
            topology: std::sync::Arc::new(std::sync::Mutex::new(hwloc::Topology::new())),
            threads: Vec::new(),
            logger,
        }
    }

    pub fn spawn(&mut self, f: impl Fn() + Send + 'static) -> &mut Self {
        self.threads.push(std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build runtime")
                .block_on(async {
                    let set = tokio::task::LocalSet::new();
                    set.run_until(async { f() }).await;
                    set.await;
                });
        }));
        self
    }

    pub fn spawn_pinned(&mut self, f: impl Fn() + Send + 'static, _cpu_id: usize) -> &mut Self {
        #[cfg(feature = "thread-pinning")]
        let topology = self.topology.clone();
        self.threads.push(std::thread::spawn(move || {
            #[cfg(feature = "thread-pinning")]
            bind_thread_to_cpu(_cpu_id, topology);
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let set = tokio::task::LocalSet::new();
                    set.run_until(async { f() }).await;
                    set.await;
                });
        }));
        self
    }

    pub fn await_termination(self) {
        drop(self)
    }
}

#[cfg(feature = "thread-pinning")]
fn bind_thread_to_cpu(
    _cpu_id: usize,
    _topology: std::sync::Arc<std::sync::Mutex<hwloc::Topology>>,
) {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            let thread_id = unsafe { libc::pthread_self() };
            let mut topology = topology.lock().unwrap();
            let cpus = topology.objects_with_type(&ObjectType::Core).unwrap();
            let cpuset = cpus.get(_cpu_id).expect("Core not found").cpuset().unwrap();
            _topology
                .set_cpubind_for_thread(thread_id, cpuset, CPUBIND_THREAD)
                .unwrap();
        }
    }
}

impl Drop for TaskParallelRunner {
    fn drop(&mut self) {
        for thread in self.threads.drain(..) {
            thread.join().expect("Failed to join thread");
        }
    }
}
