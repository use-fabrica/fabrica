use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Task<T> {
    inner: async_task::Task<T>,
}

impl<T> Task<T> {
    pub fn detach(self) {
        self.inner.detach()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }
}

impl<T> Future for Task<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

pub fn spawn<Fut>(future: Fut) -> (async_task::Runnable, Task<Fut::Output>)
where
    Fut: Future + 'static,
    Fut::Output: 'static + Send,
{
    let (runnable, task) = async_task::spawn_local(future, |runnable: async_task::Runnable| {
        runnable.schedule();
    });

    (runnable, Task { inner: task })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_await_returns_future_result() {
        let (_runnable, task) = spawn(async { 42 });
        let result = smol::block_on(task);
        assert_eq!(result, 42);
    }
}
