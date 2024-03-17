use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Task structure.
///
/// This is a wrapper around a pinned, heap-allocated, and
/// dynamically dispatched future with the empty type ()
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// This function takes an arbitrary future with an output type of () and pins it in memory
    /// through the Box::pin function.
    /// Then it wraps the boxed future in the Task struct and returns it.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()>{
        self.future.as_mut().poll(context)
    }
}
