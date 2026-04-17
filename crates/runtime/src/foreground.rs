use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub struct ForegroundExecutor {
    queue: Arc<Mutex<VecDeque<async_task::Runnable>>>,
}

impl ForegroundExecutor {
    pub fn new() -> Self {
        ForegroundExecutor {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn spawn<Fut>(&self, future: Fut) -> async_task::Task<Fut::Output>
    where
        Fut: Future + 'static,
        Fut::Output: 'static,
    {
        let queue = self.queue.clone();

        let (runnable, task) = async_task::spawn_local(future, move |runnable| {
            queue.lock().unwrap().push_back(runnable);
        });
        runnable.schedule();

        task
    }

    pub fn tick(&self) {
        let runnables: Vec<_> = self.queue.lock().unwrap().drain(..).collect();
        for runnable in runnables {
            runnable.run();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_spawn_and_tick() {
        let exec = ForegroundExecutor::new();
        let task = exec.spawn(async { 42 });
        exec.tick();

        let result = smol::block_on(task);
        assert_eq!(result, 42);
    }

    #[test]
    fn foreground_handles_async_continuations() {
        let exec = ForegroundExecutor::new();

        let task = exec.spawn(async {
            futures_lite::future::yield_now().await;
            42
        });

        while !task.is_finished() {
            exec.tick();
        }

        let result = smol::block_on(task);
        assert_eq!(result, 42);
    }

    #[test]
    fn detached_task_runs_to_completion_without_await() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };

        let mut exec = ForegroundExecutor::new();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let task = exec.spawn(async move {
            flag_clone.store(true, Ordering::SeqCst);
        });

        task.detach();
        exec.tick();

        assert!(flag.load(Ordering::SeqCst));
    }
}
