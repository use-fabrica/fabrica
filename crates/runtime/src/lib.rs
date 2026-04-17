mod background;
mod foreground;
mod priority;

use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub use background::BackgroundExecutor;
pub use foreground::ForegroundExecutor;
pub use priority::Priority;

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

#[test]
fn background_to_foreground_bridge() {
    let fg = ForegroundExecutor::new();
    let bg = BackgroundExecutor::new();

    let task = fg.spawn(async move {
        // This runs on foreground
        let bg_result = bg
            .spawn(async {
                // This runs on background thread
                42
            })
            .await;
        // After .await, we should be back on foreground
        bg_result + 1
    });

    // Drive foreground until complete
    while !task.is_finished() {
        fg.tick();
    }

    let result = smol::block_on(task);
    assert_eq!(result, 43);
}
