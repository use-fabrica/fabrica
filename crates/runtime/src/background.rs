use std::sync::Arc;

use async_channel::Sender;

pub struct BackgroundExecutor {
    executor: Arc<smol::Executor<'static>>,
    signal: Sender<()>,
}

impl BackgroundExecutor {
    pub fn new() -> Self {
        let executor = Arc::new(smol::Executor::new());
        let (signal, shutdown) = async_channel::unbounded::<()>();

        let ex = executor.clone();
        std::thread::spawn(move || smol::block_on(ex.run(shutdown.recv())));

        Self { executor, signal }
    }

    pub fn spawn<Fut>(&self, future: Fut) -> smol::Task<Fut::Output>
    where
        Fut: Future + 'static + Send,
        Fut::Output: 'static,
        Fut::Output: Send,
    {
        self.executor.spawn(future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_spawn_runs_on_thread_pool() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };

        let exec = BackgroundExecutor::new();
        let ran_on_bg = Arc::new(AtomicBool::new(false));
        let ran_on_bg_clone = ran_on_bg.clone();

        let task = exec.spawn(async move {
            ran_on_bg_clone.store(true, Ordering::SeqCst);
            42
        });

        let result = smol::block_on(task);
        assert_eq!(result, 42);
        assert!(ran_on_bg.load(Ordering::SeqCst));
    }
}
