use tokio::task::JoinSet;

pub struct PinnedDataParallelRunner {
    topology: std::sync::Arc<std::sync::Mutex<hwloc::Topology>>,
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl PinnedDataParallelRunner {
    pub fn new() -> Self {
        Self {
            topology: std::sync::Arc::new(std::sync::Mutex::new(hwloc::Topology::new())),
            threads: Vec::new(),
        }
    }

    pub fn spawn_current_thread(&mut self, f: impl Fn(&mut JoinSet<()>)) -> &mut Self {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime")
            .block_on(async {
                let mut join_set = JoinSet::<()>::new();
                let local_set = tokio::task::LocalSet::new();
                local_set.run_until(async { f(&mut join_set) }).await;
                while let Some(_) = join_set.join_next().await {}
                local_set.await;
            });
        self
    }

    pub fn spawn_pinned(
        &mut self,
        f: impl Fn(&mut JoinSet<()>) + Send + 'static,
        _cpu_id: usize,
    ) -> &mut Self {
        let topology = self.topology.clone();
        self.threads.push(std::thread::spawn(move || {
            bind_thread_to_cpu(_cpu_id, topology);
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let mut join_set = JoinSet::<()>::new();
                    let local_set = tokio::task::LocalSet::new();
                    local_set.run_until(async { f(&mut join_set) }).await;
                    while let Some(_) = join_set.join_next().await {}
                    local_set.await;
                });
        }));
        self
    }

    pub fn await_termination(self) {
        drop(self)
    }
}

fn bind_thread_to_cpu(
    _cpu_id: usize,
    _topology: std::sync::Arc<std::sync::Mutex<hwloc::Topology>>,
) {
    let thread_id = unsafe { libc::pthread_self() };
    let mut topology = topology.lock().unwrap();
    let cpus = topology.objects_with_type(&ObjectType::Core).unwrap();
    let cpuset = cpus.get(_cpu_id).expect("Core not found").cpuset().unwrap();
    _topology
        .set_cpubind_for_thread(thread_id, cpuset, CPUBIND_THREAD)
        .unwrap();
}

impl Drop for PinnedDataParallelRunner {
    fn drop(&mut self) {
        for thread in self.threads.drain(..) {
            thread.join().expect("Failed to join thread");
        }
    }
}
