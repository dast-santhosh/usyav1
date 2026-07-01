/// A basic task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A rudimentary cooperative Task structure
pub struct Task {
    id: TaskId,
    // In the future this will hold a Box<dyn Future<Output = ()>>
}

impl Task {
    pub fn new() -> Self {
        Task { id: TaskId::new() }
    }
}

/// A basic task executor skeleton
pub struct Executor {
    // In the future:
    // tasks: BTreeMap<TaskId, Task>,
    // task_queue: Arc<ArrayQueue<TaskId>>,
    // waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {}
    }

    pub fn spawn(&mut self, _task: Task) {
        // Will insert into tasks map and task_queue
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn run_ready_tasks(&mut self) {
        // Pop tasks from the queue and poll them
    }

    fn sleep_if_idle(&self) {
        // Halt CPU to save power if task queue is empty
        use x86_64::instructions::interrupts::{self, enable_and_hlt};
        interrupts::disable();
        // if self.task_queue.is_empty() {
            enable_and_hlt();
        // } else {
        //     interrupts::enable();
        // }
    }
}
