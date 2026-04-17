mod background;
mod foreground;

use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub use background::BackgroundExecutor;
pub use foreground::ForegroundExecutor;

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
